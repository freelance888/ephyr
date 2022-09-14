use std::collections::HashMap;
use std::ops::Deref;
use futures::{StreamExt, TryFutureExt};
use gstreamer::{Element, Pipeline, State};
use gstreamer as gst;
use gstreamer::glib::clone::Downgrade;
use gstreamer::glib::ObjectExt;
use gstreamer::prelude::{ElementExtManual, GObjectExtManualGst, GstBinExtManual};
use gstreamer::traits::{ElementExt, GstBinExt, GstObjectExt};
use libc::pipe;
use tokio::task::JoinHandle;
use url::Url;
use uuid::Uuid;
use crate::state::{Output, OutputId};
use ephyr_log::log;
use crate::gstreamer::output_bin::OutputBin;

#[derive(Debug)]
pub(crate) struct RestreamPipeline {
    pipeline: Pipeline,
    source: Element,
    tee: Element,
    sinks: HashMap<Uuid, OutputBin>,
    bus_handle: Option<JoinHandle<()>>,

    from_url: Option<Url>,
}

impl RestreamPipeline {
    pub(crate) fn new(id: &str) -> Self {
        let pipeline = Pipeline::new(Some(&format!("pipeline_{}", id)));
        // TODO change this to more generic input
        let source =
            gst::ElementFactory::make("rtmp2src", Some("input-src"))
                .expect("Could not create pipeline");
        // source.set_property("idle-timeout", 1u32);
        let sinks = HashMap::new();
        let tee =
            gst::ElementFactory::make("tee", Some("tee"))
                .expect("Could not create Tee for pipeline");

        pipeline.add_many(&[&source, &tee]);
        source.link(&tee).expect("Could not connect source to Tee");

        log::info!("Creating new pipeline from scratch with id: {}", id);

        RestreamPipeline {
            pipeline,
            source,
            tee,
            sinks,
            bus_handle: None,
            from_url: None,
        }
    }

    pub(crate) fn set_source(&mut self, url: &Url) {
        // self.pipeline.set_state(State::Paused).expect("Could not change the pipeline status");
        // TODO change this to more generic input
        self.source.set_property_from_str("location", &url.to_string());
        self.from_url = Some(url.clone());
        log::info!("Setting pipeline input: {}", url.to_string());
    }

    pub(crate) fn set_sinks<'a, I>(&mut self, outputs:I) -> Option<u32>
    where
        I: Iterator<Item = &'a Output>,
    {
        // self.pipeline.set_state(State::Paused).expect("Could not change the pipeline status");

        let mut new_sinks = HashMap::with_capacity(self.sinks.len());

        let mut count = 0u32;
        outputs.for_each(|o| {
            let mut bin = self.sinks.remove(&o.id.into()).unwrap_or_else(|| {
                let mut bin = OutputBin::new(&self.pipeline, &o.id.into(), &o.dst);
                bin.link_src(&self.tee);
                bin
            });
            log::info!("Adding output to pipeline: {}", o.dst.to_string());
            new_sinks.insert(o.id.into(), bin);
            count += 1;
        });


        for (id, bin) in &mut self.sinks {
            bin.unlink_and_remove(&self.tee, &self.pipeline);
        }
        self.sinks = new_sinks;
        Some(count)
    }

    pub fn run(&mut self) {
        if self.bus_handle.is_none() {
            self.bus_handle = Some(tokio::spawn(Self::bus_loop(self.pipeline.bus().unwrap(), self.pipeline.clone())));
        }
        if self.from_url.is_none() {
            panic!("Trying to run pipeline without source URL");
        }
        self.pipeline.set_state(State::Playing).expect("Could not start pipeline");
    }

    pub fn stop(&mut self) {
        // if self.pipeline.current_state() == State::Playing || self.pipeline.current_state() == State::Paused {
        self.pipeline.set_state(State::Ready).expect("Could not stop the pipeline");
        self.from_url = None;
        // }
    }

    async fn bus_loop(bus: gst::Bus, pipeline: Pipeline) {
        let mut bus_msg = bus.stream();
        let mut i = 0;

        while let Some(msg) = bus_msg.next().await {
            use gst::MessageView;
            log::info!("Pipeline {} message received {:?}", pipeline.name(), msg);

            match msg.view() {
                MessageView::Error(err) => {
                    log::error!("Element {:?} thrown error: {}", err.src().map(|e| e.path_string()), err.error());
                    gst::debug_bin_to_dot_file(&pipeline, gst::DebugGraphDetails::ALL, format!("{}_out_{}", pipeline.name(), i));
                    i += 1;
                    if let Some(src) = err.src() {
                        log::info!("{:?}", src);
                        let elem = pipeline.by_name(src.name().as_str());
                    }
                    // pipeline.set_state(State::Null).unwrap();
                    // log::info!("Stopping pipeline with panic");
                    // panic!("error");
                }
                MessageView::Eos(_) => {
                    pipeline.set_state(State::Ready).unwrap();
                    log::info!("Stopping pipeline");
                }
                MessageView::StreamStart(_) => {
                    log::info!("Stream started in {}", pipeline.name());
                    pipeline.set_state(State::Playing).expect("Could not start pipeline after StreamStart");
                    gst::debug_bin_to_dot_file(&pipeline, gst::DebugGraphDetails::ALL, format!("{}_out_{}_ss", pipeline.name(), i));
                    i += 1;
                }
                _ => (),
            }
        }
    }
}

impl Drop for RestreamPipeline {
    fn drop(&mut self) {
        self.pipeline.set_state(State::Null).expect("Could not set state of dropped pipeline to NULL");
        if let Some(handle) = &self.bus_handle {
            handle.abort();
        }
    }
}