//! Handle to a running [FFmpeg] process performing a re-streaming.
//!
//! [FFmpeg]: https://ffmpeg.org

use chrono::{DateTime, Utc};
use std::{
    panic::AssertUnwindSafe, path::Path, process::Stdio, time::Duration,
};

use ephyr_log::log;
use futures::{future, pin_mut, FutureExt as _, TryFutureExt as _};
use tokio::{process::Command, time};
use tokio::io::AsyncWriteExt;
use tokio::signal::unix::SignalKind;
use nix::libc::{kill, pid_t};
use nix::unistd::Pid;
use nix::sys::signal::{self, Signal};

use crate::{
    display_panic,
    ffmpeg::restreamer_kind::RestreamerKind,
    state::{State, Status},
    types::DroppableAbortHandle,
};

/// Handle to a running [FFmpeg] process performing a re-streaming.
///
/// [FFmpeg]: https://ffmpeg.org
#[derive(Debug)]
pub struct Restreamer {
    /// Handle for stopping [FFmpeg] process of this [`Restreamer`].
    ///
    /// [FFmpeg]: https://ffmpeg.org
    kill_tx: tokio::sync::watch::Sender<i32>,

    /// Kind of a spawned [FFmpeg] process describing the actual job it
    /// performs.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    pub kind: RestreamerKind,
}

impl Restreamer {
    /// Creates a new [`Restreamer`] spawning the actual [FFmpeg] process in
    /// background. Once this [`Restreamer`] is dropped, its [FFmpeg] process is
    /// aborted.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[must_use]
    pub fn run<P: AsRef<Path> + Send + 'static>(
        ffmpeg_path: P,
        kind: RestreamerKind,
        state: State,
    ) -> Self {
        let (kind_for_abort, state_for_abort) = (kind.clone(), state.clone());
        let kind_for_spawn = kind.clone();
        let mut time_of_fail: Option<DateTime<Utc>> = None;
        let (mut kill_tx, mut kill_rx) = tokio::sync::watch::channel(0);

        let spawner = async move {
            let kill_rx_for_loop = kill_rx.clone();
            loop {
                let (kind, state) = (&kind_for_spawn, &state);
                let mut cmd = Command::new(ffmpeg_path.as_ref());
                let mut kill_rx_for_ffmpeg = kill_rx.clone();

                let result = AssertUnwindSafe(
                    async move {
                        Self::change_status(
                            time_of_fail,
                            kind,
                            state,
                            Status::Initializing,
                        );

                        kind.setup_ffmpeg(
                            cmd.kill_on_drop(true)
                                .stdin(Stdio::null())
                                .stdout(Stdio::null())
                                .stderr(Stdio::piped()),
                            state,
                        )
                        .map_err(|e| {
                            log::error!(
                                "Failed to setup FFmpeg re-streamer: {}",
                                e,
                            );
                        })
                        .await?;

                        let running = kind.run_ffmpeg(cmd, kill_rx_for_ffmpeg);
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
                                    "Failed to run FFmpeg re-streamer: {}",
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
                    })
                )
                .catch_unwind()
                .await
                .map_err(|p| {
                    log::crit!(
                        "Panicked while spawning/observing FFmpeg \
                         re-streamer: {}",
                        display_panic(&p),
                    );
                });

                if kill_rx_for_loop.borrow().clone() != 0 {
                    break;
                }

                if let RestreamerKind::File(_) = kind {
                    let _ = state
                        .restreams
                        .lock_mut()
                        .iter_mut()
                        .find(|r| r.playlist.id == kind.id())
                        .map(|r| r.playlist.currently_playing_file = None);
                    break;
                }

                time::sleep(Duration::from_secs(2)).await;
            }
        };

        // Spawn FFmpeg re-streamer manager as a child process.
        drop(tokio::spawn(spawner.map(move |_| {
            kind_for_abort.renew_status(Status::Offline, &state_for_abort);
        })));

        Self {
            kill_tx,
            kind,
        }
    }

    /// Check if the last time of fail was less that 15 sec. ago than [FFmpeg]
    /// process is unstable.
    /// In other case set new `[Status]` to `[RestreamerKind]`
    ///
    /// [FFmpeg]: https://ffmpeg.org
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
    fn drop(&mut self) {
        let _ = self.kill_tx.send(1);
    }
}
