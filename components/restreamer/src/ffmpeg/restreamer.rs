//! Handle to a running [FFmpeg] process performing a re-streaming.
//!
//! [FFmpeg]: https://ffmpeg.org

use chrono::{DateTime, Utc};
use std::{
    panic::AssertUnwindSafe, path::Path, process::Stdio, time::Duration,
};

use ephyr_log::log;
use futures::{future, pin_mut, FutureExt as _, TryFutureExt as _};
use tokio::{process::Command, sync::watch, time};

use crate::{
    display_panic,
    ffmpeg::restreamer_kind::RestreamerKind,
    state::{State, Status},
};

/// Status of [Restreamer] process
///
/// Using for communication through [`tokio::sync::watch`]
/// between [`Restreamer`] and [`MixingRestreamer`] with [`RestreamerKind`].
///
/// [`MixingRestreamer`]: crate::ffmpeg::MixingRestreamer
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub(crate) enum RestreamerStatus {
    /// [`Restreamer`] process is started and running
    Started = 0,
    /// [`Restreamer`] process is finishing
    Finished = 1,
}

/// Handle to a running [FFmpeg] process performing a re-streaming.
///
/// [FFmpeg]: https://ffmpeg.org
#[derive(Debug)]
pub struct Restreamer {
    /// Kind of a spawned [FFmpeg] process describing the actual job it
    /// performs.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    pub kind: RestreamerKind,

    /// Handle for stopping [FFmpeg] process of this [`Restreamer`].
    ///
    /// [FFmpeg]: https://ffmpeg.org
    kill_tx: watch::Sender<RestreamerStatus>,

    /// Handle for SIGKILL hanged [FFmpeg] process of this [`Restreamer`].
    ///
    /// [FFmpeg]: https://ffmpeg.org
    abort_if_hanged: future::AbortHandle,
}

impl Restreamer {
    /// Creates a new [`Restreamer`] spawning the actual [FFmpeg] process in
    /// background. Once this [`Restreamer`] is dropped, its [FFmpeg] process is
    /// killed with SIGTERM or aborted.
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
        let (kill_tx, kill_rx) = watch::channel(RestreamerStatus::Started);

        let (spawner, abort_if_hanged) = future::abortable(async move {
            let kill_rx_for_loop = kill_rx.clone();
            loop {
                let (kind, state) = (&kind_for_spawn, &state);
                let mut cmd = Command::new(ffmpeg_path.as_ref());
                let kill_rx_for_ffmpeg = kill_rx.clone();

                let _ = AssertUnwindSafe(
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
                    }),
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

                if *kill_rx_for_loop.borrow() == RestreamerStatus::Finished {
                    break;
                }

                time::sleep(Duration::from_secs(2)).await;
            }
        });

        // Spawn FFmpeg re-streamer manager as a child process.
        drop(tokio::spawn(spawner.map(move |_| {
            kind_for_abort.renew_status(Status::Offline, &state_for_abort);
        })));

        Self {
            kind,
            kill_tx,
            abort_if_hanged,
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
    /// Send signal that [`Restreamer`] process is finished
    fn drop(&mut self) {
        // Send notification to kill FFMPEG with SIGTERM
        log::debug!("Send signal to FFmpeg's");
        let _ = self.kill_tx.send(RestreamerStatus::Finished);

        // If FFmpeg wasn't killed kill it with SIGKILL
        let abort_for_future = self.abort_if_hanged.clone();
        drop(tokio::spawn(async move {
            log::debug!("Abort Restreamer in 5 sec if not killed");
            time::sleep(Duration::from_secs(5)).await;
            abort_for_future.abort();
        }));
    }
}
