//! FFmpeg related shared utils
//!
//! [FFmpeg]: https://ffmpeg.org
use libc::pid_t;
use nix::{
    sys::{signal, signal::Signal},
    unistd::Pid,
};
use std::{convert::TryInto, time::Duration};
use tokio::{sync::watch, task::JoinHandle};

/// Kill [FFmpeg] process with SIGTERM signal
///
/// [FFmpeg] not always die after single [SIGTERM] signal
/// so we send it twice with interval of 1 ms
///
/// # Panics
///
/// If not possible to get Process ID and convert it to i32
/// If OS return an error during on kill call
///
/// [FFmpeg]: https://ffmpeg.org
/// [SIGTERM]: https://en.wikipedia.org/wiki/Signal_(IPC)#SIGTERM
#[must_use]
pub fn kill_ffmpeg_process_by_sigterm(
    process_id: Option<u32>,
    mut kill_rx: watch::Receiver<i32>,
) -> JoinHandle<()> {
    let p_id: pid_t = process_id
        .expect("Failed to retrieve Process ID")
        .try_into()
        .expect("Failed to convert u32 to i32");
    // Task that sends SIGTERM if async stop of ffmpeg was invoked
    tokio::spawn(async move {
        let _ = kill_rx.changed().await;
        // It is necessary to send the signal two times and wait after
        // sending the first one to correctly close all ffmpeg processes
        signal::kill(Pid::from_raw(p_id), Signal::SIGTERM).unwrap();
        tokio::time::sleep(Duration::from_millis(1)).await;
        signal::kill(Pid::from_raw(p_id), Signal::SIGTERM).unwrap();
    })
}
