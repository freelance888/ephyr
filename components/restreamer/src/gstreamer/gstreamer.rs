use std::collections::HashMap;
use uuid::Uuid;
use crate::{State, state};
use crate::gstreamer::restream_pipeline::RestreamPipeline;
use gstreamer as gst;

#[derive(Debug)]
pub struct Gstreamer {
    /// Pool of currently running [FFmpeg] re-streaming processes identified by
    /// an ID of the correspondent element in a [`State`].
    ///
    /// So, potentially allows duplication.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    pool: HashMap<Uuid, RestreamPipeline>,

    /// Application [`State`] dictating which [FFmpeg] processes should run.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    /// [`State`]: crate::state::State
    state: State,
}

impl Gstreamer {
    /// Creates a new [`RestreamersPool`] out of the given parameters.
    #[inline]
    #[must_use]
    pub fn new(state: State) -> Self {
        Self {
            pool: HashMap::new(),
            state,
        }
    }

    /// Adjusts this [`RestreamersPool`] to run [FFmpeg] re-streaming processes
    /// according to the given renewed [`state::Restream`]s.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    pub fn apply(&mut self, restreams: &[state::Restream]) {
        // The most often case is when one new FFmpeg process is added.
        let mut new_pool = HashMap::with_capacity(self.pool.len() + 1);

        for r in restreams {
            let mut pipeline = self.pool.remove(&r.id.into()).unwrap_or_else(|| {
                RestreamPipeline::new(&r.id.to_string())
            });

            let ready_url = r.input.get_ready_url(&r.key);
            if !r.input.enabled || ready_url.is_none() {
                pipeline.stop();
            } else {
                pipeline.set_source(&ready_url.unwrap());

                let res = pipeline.set_sinks(r.outputs.iter().filter(|o| o.enabled));

                if let Some(count) = res {
                    if count > 0 {
                        pipeline.run();
                    } else {
                        pipeline.stop();
                    }
                }

            }
            new_pool.insert(r.id.into(), pipeline);

        }

        self.pool = new_pool;
    }
}
