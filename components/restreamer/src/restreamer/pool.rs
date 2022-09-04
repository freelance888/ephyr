//! Pool of [GStD] processes performing re-streaming of a media traffic.
//!
//! [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon

use std::collections::HashMap;

use ephyr_log::log;
use url::Url;
use uuid::Uuid;

use crate::{
    restreamer::{Restreamer, RestreamerKind},
    state::{self, State},
};
use gst_client::GstClient;
use std::result::Result::Err;

/// Pool of [GStD] processes performing re-streaming of a media traffic.
///
/// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
#[derive(Debug)]
pub struct RestreamersPool {
    /// Address to a [GStD] daemon used for spawning processes.
    ///
    /// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
    client: GstClient,

    /// Pool of currently running [GStD] re-streaming processes identified by
    /// an ID of the correspondent element in a [`State`].
    ///
    /// So, potentially allows duplication.
    ///
    /// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
    pool: HashMap<Uuid, Restreamer>,

    /// Application [`State`] dictating which [GStD] processes should run.
    ///
    /// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
    /// [`State`]: crate::state::State
    state: State,
}

impl RestreamersPool {
    /// Creates a new [`RestreamersPool`] out of the given parameters.
    #[inline]
    #[must_use]
    pub fn new(client: GstClient, state: State) -> Self {
        Self {
            client,
            pool: HashMap::new(),
            state,
        }
    }

    /// Adjusts this [`RestreamersPool`] to run [GStD] re-streaming processes
    /// according to the given renewed [`state::Restream`]s.
    ///
    /// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
    pub fn apply(&mut self, restreams: &[state::Restream]) {
        // The most often case is when one new GStD process is added.
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
            for o in &r.outputs {
                let _ = self.apply_output(&input_url, o, &mut new_pool);
            }
        }

        self.pool = new_pool;
    }

    /// Traverses the given [`state::Input`] filling the `new_pool` with
    /// required [GStD] re-streaming processes. Tries to preserve already
    /// running [GStD] processes in its `pool` as much as possible.
    ///
    /// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
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
    /// a required [GStD] re-streaming process. Tries to preserve already
    /// running [GStD] processes in its `pool` as much as possible.
    ///
    /// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
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
            .and_then(|mut p| (!p.kind.needs_restart(&new_kind)).then(|| p))
            .unwrap_or_else(|| {
                Restreamer::run(
                    self.client.clone(),
                    new_kind,
                    self.state.clone(),
                )
            });

        let old_process = new_pool.insert(id, process);
        drop(old_process);
        Some(())
    }

    /// Inspects the given [`state::Output`] filling the `new_pool` with a
    /// required [GStD] re-streaming process. Tries to preserve already
    /// running [GStD] processes in its `pool` as much as possible.
    ///
    /// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
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
            .and_then(|mut p| (!p.kind.needs_restart(&new_kind)).then(|| p))
            .unwrap_or_else(|| {
                Restreamer::run(
                    self.client.clone(),
                    new_kind,
                    self.state.clone(),
                )
            });

        let old_process = new_pool.insert(id, process);
        drop(old_process);
        Some(())
    }
}
