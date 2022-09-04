//! [`GStreamer`]-based definitions and implementations.
//!
//! Handle to a running [GStD] process performing a re-streaming.
//!
//! [`GStreamer`]: https://gstreamer.freedesktop.org/
//! [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
mod copy;
mod kind;
mod mixing;
mod pool;
mod transcoding;

pub use self::{
    copy::CopyRestreamer,
    kind::RestreamerKind,
    mixing::{MixingRestreamer, RestreamerStatus},
    pool::RestreamersPool,
    transcoding::TranscodingRestreamer,
};

use chrono::{DateTime, Utc};
use std::{panic::AssertUnwindSafe, path::Path, time::Duration};

use ephyr_log::log;
use futures::{future, pin_mut, FutureExt as _, TryFutureExt as _};
use gst_client::GstClient;
use tokio::{process::Command, time};
use url::Url;
use uuid::Uuid;

use crate::{
    display_panic,
    state::{State, Status},
};

/// Handle to a running [GStD] process performing a re-streaming.
///
/// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
#[derive(Debug)]
pub struct Restreamer {
    /// Kind of a spawned [GStD] process describing the actual job it
    /// performs.
    ///
    /// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
    pub kind: RestreamerKind,

    /// Handle for hanged [GStD] process of this [`Restreamer`].
    ///
    /// Kill with SIGKILL if handed
    ///
    /// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
    abort_if_hanged: future::AbortHandle,

    client: gst_client::GstClient,
}

impl Restreamer {
    /// Creates a new [`Restreamer`] spawning the actual [GStD] process in
    /// background. Once this [`Restreamer`] is dropped, its [GStD] process is
    /// killed with SIGTERM or aborted.
    ///
    /// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
    #[must_use]
    pub fn run(client: GstClient, kind: RestreamerKind, state: State) -> Self {
        let (kind_for_abort, state_for_abort) = (kind.clone(), state.clone());
        let kind_for_spawn = kind.clone();
        let mut time_of_fail: Option<DateTime<Utc>> = None;

        let client_for_loop = client.clone();
        let (spawner, abort_if_hanged) = future::abortable(async move {
            loop {
                let (kind, state) = (&kind_for_spawn, &state);
                let client = client_for_loop.clone();

                let _ = AssertUnwindSafe(
                    async move {
                        Self::change_status(
                            time_of_fail,
                            kind,
                            state,
                            Status::Initializing,
                        );

                        let pipeline_cmd =
                            kind.setup_pipeline(state, client.clone()).await;
                        // .map_err(|e| {
                        //     log::error!(
                        //         "Failed to setup GStD re-streamer: {}",
                        //         e,
                        //     );
                        // })?;

                        let running =
                            kind.run_pipeline(pipeline_cmd, client.clone());
                        pin_mut!(running);

                        let set_online = async move {
                            // If ffmpeg process does not fail within 10 sec
                            // than set `Online` status.
                            time::sleep(Duration::from_secs(10)).await;
                            kind.renew_status(Status::Online, state);
                            future::pending::<()>().await;
                            Ok(())
                        };
                        pin_mut!(set_online);

                        future::try_select(running, set_online)
                            .await
                            .map_err(|e| {
                                log::error!(
                                    "Failed to run GStD re-streamer: {}",
                                    e.factor_first().0,
                                );
                            })
                            .map(|r| r.factor_first().0)
                    }
                    .unwrap_or_else(|_| {
                        Self::change_status(
                            time_of_fail,
                            kind,
                            state,
                            Status::Offline,
                        );
                        time_of_fail = Some(Utc::now());
                    }),
                )
                .catch_unwind()
                .await
                .map_err(|p| {
                    log::crit!(
                        "Panicked while spawning/observing GStD \
                         re-streamer: {}",
                        display_panic(&p),
                    );
                });

                time::sleep(Duration::from_secs(2)).await;
            }
        });

        // Spawn GStD re-streamer manager as a child process.
        drop(tokio::spawn(spawner.map(move |_| {
            kind_for_abort.renew_status(Status::Offline, &state_for_abort);
        })));

        Self {
            kind,
            abort_if_hanged,
            client: client.clone(),
        }
    }

    /// Check if the last time of fail was less that 15 sec. ago than [GStD]
    /// process is unstable.
    /// In other case set new `[Status]` to `[RestreamerKind]`
    ///
    /// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
    fn change_status(
        time_of_fail: Option<DateTime<Utc>>,
        kind: &RestreamerKind,
        state: &State,
        new_status: Status,
    ) {
        match time_of_fail {
            Some(dt) => {
                let seconds =
                    Utc::now().signed_duration_since(dt).num_seconds();
                let status = if seconds < 15 {
                    Status::Unstable
                } else {
                    new_status
                };
                kind.renew_status(status, state);
            }
            None => {
                kind.renew_status(new_status, state);
            }
        }
    }
}

impl Drop for Restreamer {
    /// Send signal that [`Restreamer`] process is finished
    fn drop(&mut self) {
        // If GStD wasn't killed kill it with SIGKILL
        let abort_for_future = self.abort_if_hanged.clone();
        let id = self.kind.id::<Uuid>().to_string();
        let _ = self.client.pipeline(id).delete();
        drop(tokio::spawn(async move {
            log::debug!("Abort Restreamer in 5 sec if not killed");
            time::sleep(Duration::from_secs(5)).await;
            abort_for_future.abort();
        }));
    }
}
