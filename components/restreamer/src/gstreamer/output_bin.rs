use gstreamer::{Element, Pipeline, State};
use gstreamer::prelude::{GObjectExtManualGst, GstBinExtManual};
use url::Url;
use uuid::Uuid;
use gstreamer as gst;
use gstreamer::glib::ObjectExt;
use gstreamer::traits::{ElementExt, GstBinExt};
use libc::pipe;

#[derive(Debug)]
pub(crate) struct OutputBin {
    queue: Element,
    sink: Element,

    is_linked: bool,
    to_url: Url,
}

impl OutputBin {
    pub(crate) fn new(pipeline: &Pipeline, id: &Uuid, to_url: &Url) -> Self {
        let id_str = &id.to_string();
        let queue = gst::ElementFactory::make("queue2", Some(&format!("queue_{}", id_str))).expect("Could not create queue");
        let sink = gst::ElementFactory::make("rtmp2sink", Some(&format!("sink_{}", id_str))).expect("Could not create sink");
        // sink.set_property("async-connect", false);
        pipeline.add_many(&[&queue, &sink]).expect("Could not add elements to OutputBin");
        // let queue_for_connect = queue.clone();
        // let sink_for_connect = sink.clone();
        // queue.connect_pad_added(move |a, pad| {
        //     queue_for_connect.link(&sink_for_connect);
        // });
        OutputBin {
            to_url: to_url.clone(),
            queue,
            sink,
            is_linked: false,
        }
    }

    pub(crate) fn link_src(&mut self, elem: &Element) {
        elem.link(&self.queue).expect("Could not link Bin to existing pipeline");
        self.queue.link(&self.sink).expect("Could not connect queue to sink");
        self.is_linked = true;
    }

    pub(crate) fn set_url(&mut self, url: &str) {
        self.sink.set_property_from_str("location", url.to_string().as_str());
    }

    pub(crate) fn unlink_and_remove(&mut self, elem: &Element, pipeline: &Pipeline) {
        elem.unlink(&self.queue);
        self.is_linked = false;
        pipeline.remove_many(&[&self.queue, &self.sink]).expect("Could not remove OutputBin from pipeline");
    }
}

impl Drop for OutputBin {
    fn drop(&mut self) {
        if self.is_linked {
            panic!("The Bin is linked, but is being dropped");
        }
        self.queue.set_state(State::Null);
        self.sink.set_state(State::Null);
    }
}
