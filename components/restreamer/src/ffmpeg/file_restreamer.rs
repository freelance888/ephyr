use std::{
    path::{Path},
    process::Stdio,
};

use tokio::{io, process::Command};
use url::Url;
use uuid::Uuid;

use crate::{
    display_panic, dvr,
    state::{self},
};

/// Kind of a [FFmpeg] re-streaming process that streams a local file to input
/// endpoint "as is", without performing any live stream modifications.
///
/// [FFmpeg]: https://ffmpeg.org
#[derive(Clone, Debug)]
pub struct FileRestreamer {
    /// ID of an element in a [`State`] this [`FileRestreamer`] process is
    /// related to.
    pub id: Uuid,

    // TODO change this to file_ID
    /// [`Url`] to pull a live stream from.
    pub from_url: Url,

    /// [`Url`] to publish the pulled live stream onto.
    pub to_url: Url,
}

impl FileRestreamer {
    /// Checks whether this [`FileRestreamer`] process must be restarted, as
    /// cannot apply the new `actual` params on itself correctly, without
    /// interruptions.
    #[inline]
    #[must_use]
    pub fn needs_restart(&self, actual: &Self) -> bool {
        self.from_url != actual.from_url || self.to_url != actual.to_url
    }

    /// Properly setups the given [FFmpeg] [`Command`] for this
    /// [`FileRestreamer`] before running it.
    ///
    /// # Errors
    ///
    /// If the given [FFmpeg] [`Command`] fails to be setup.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    pub(crate) async fn setup_ffmpeg(
        &self,
        cmd: &mut Command,
        repeat: bool,
    ) -> io::Result<()> {
        let _ = cmd.stderr(Stdio::inherit()).args(&["-loglevel", "debug"]);
        match self.from_url.scheme() {
            "file" => {
                let _ = cmd.arg("-re");
                if repeat {
                    let _ = cmd.args(&["-stream_loop", "-1"]);
                }
            }

            _ => unimplemented!(),
        };
        let _ = cmd.args(&["-i", self.from_url.as_str()]);

        let _ = match self.to_url.scheme() {
            "file"
            if Path::new(self.to_url.path()).extension()
                == Some("flv".as_ref()) =>
                {
                    cmd.args(&["-c", "copy"])
                        .arg(dvr::new_file_path(&self.to_url).await?)
                }

            "icecast" => cmd
                .args(&["-c:a", "libmp3lame", "-b:a", "64k"])
                .args(&["-f", "mp3", "-content_type", "audio/mpeg"])
                .arg(self.to_url.as_str()),

            "rtmp" | "rtmps" => cmd
                .args(&["-c", "copy"])
                .args(&["-f", "flv"])
                .arg(self.to_url.as_str()),

            "srt" => cmd
                .args(&["-c", "copy"])
                .args(&["-strict", "-2", "-y", "-f", "mpegts"])
                .arg(self.to_url.as_str()),

            _ => unimplemented!(),
        };
        Ok(())
    }
}
