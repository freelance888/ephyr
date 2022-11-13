//! Pool of [FFmpeg] processes performing re-streaming of a media traffic.
//!
//! [FFmpeg]: https://ffmpeg.org

use std::{collections::HashMap, path::PathBuf};

use ephyr_log::log;
use url::Url;
use uuid::Uuid;

use crate::{
    ffmpeg::{restreamer::Restreamer, restreamer_kind::RestreamerKind},
    state::{self, State},
};
use std::result::Result::Err;

/// Pool of [FFmpeg] processes performing re-streaming of a media traffic.
///
/// [FFmpeg]: https://ffmpeg.org
#[derive(Debug)]
pub struct RestreamersPool {
    /// Path to a [FFmpeg] binary used for spawning processes.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    ffmpeg_path: PathBuf,

    /// Pool of currently running [FFmpeg] re-streaming processes identified by
    /// an ID of the correspondent element in a [`State`].
    ///
    /// So, potentially allows duplication.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    pool: HashMap<Uuid, Restreamer>,

    /// Application [`State`] dictating which [FFmpeg] processes should run.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    /// [`State`]: crate::state::State
    state: State,

    /// Counter for already started outputs
    started_outputs_count: u16,
}

impl RestreamersPool {
    /// Creates a new [`RestreamersPool`] out of the given parameters.
    #[inline]
    #[must_use]
    pub fn new<P: Into<PathBuf>>(ffmpeg_path: P, state: State) -> Self {
        Self {
            ffmpeg_path: ffmpeg_path.into(),
            pool: HashMap::new(),
            state,
            started_outputs_count: 0,
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
            self.apply_input(&r.key, &r.input, &mut new_pool);

            if !r.input.enabled || !r.input.is_ready_to_serve() {
                continue;
            }

            let input_url = match r.main_input_rtmp_endpoint_url() {
                Ok(input_url) => input_url,
                Err(e) => {
                    log::error!(
                        "Failed to get main input RTMP endpoint: {}",
                        e
                    );
                    continue;
                }
            };
            self.started_outputs_count = 0;
            for o in &r.outputs {
                let _ = self.apply_output(&input_url, o, &mut new_pool);
            }
        }

        self.pool = new_pool;
    }

    /// Traverses the given [`state::Input`] filling the `new_pool` with
    /// required [FFmpeg] re-streaming processes. Tries to preserve already
    /// running [FFmpeg] processes in its `pool` as much as possible.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    fn apply_input(
        &mut self,
        key: &state::RestreamKey,
        input: &state::Input,
        new_pool: &mut HashMap<Uuid, Restreamer>,
    ) {
        if let Some(state::InputSrc::Failover(s)) = &input.src {
            for i in &s.inputs {
                self.apply_input(key, i, new_pool);
            }
        }
        for endpoint in &input.endpoints {
            let _ = self.apply_input_endpoint(key, input, endpoint, new_pool);
        }
    }

    /// Inspects the given [`state::InputEndpoint`] filling the `new_pool` with
    /// a required [FFmpeg] re-streaming process. Tries to preserve already
    /// running [FFmpeg] processes in its `pool` as much as possible.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    fn apply_input_endpoint(
        &mut self,
        key: &state::RestreamKey,
        input: &state::Input,
        endpoint: &state::InputEndpoint,
        new_pool: &mut HashMap<Uuid, Restreamer>,
    ) -> Option<()> {
        let id = endpoint.id.into();

        let new_kind = RestreamerKind::from_input(input, endpoint, key)?;

        let process = self
            .pool
            .remove(&id)
            .and_then(|mut p| (!p.kind.needs_restart(&new_kind)).then_some(p))
            .unwrap_or_else(|| {
                Restreamer::run(
                    self.ffmpeg_path.clone(),
                    new_kind,
                    self.state.clone(),
                    None,
                )
            });

        let old_process = new_pool.insert(id, process);
        drop(old_process);
        Some(())
    }

    /// Inspects the given [`state::Output`] filling the `new_pool` with a
    /// required [FFmpeg] re-streaming process. Tries to preserve already
    /// running [FFmpeg] processes in its `pool` as much as possible.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    fn apply_output(
        &mut self,
        from_url: &Url,
        output: &state::Output,
        new_pool: &mut HashMap<Uuid, Restreamer>,
    ) -> Option<()> {
        if !output.enabled {
            return None;
        }

        let id = output.id.into();

        let new_kind = RestreamerKind::from_output(
            output,
            from_url,
            self.pool.get(&id).map(|p| &p.kind),
        )?;

        let process = self
            .pool
            .remove(&id)
            .and_then(|mut p| (!p.kind.needs_restart(&new_kind)).then_some(p))
            .unwrap_or_else(|| {
                self.started_outputs_count += 1;
                Restreamer::run(
                    self.ffmpeg_path.clone(),
                    new_kind,
                    self.state.clone(),
                    Some(self.started_outputs_count),
                )
            });

        let old_process = new_pool.insert(id, process);
        drop(old_process);
        Some(())
    }
}
