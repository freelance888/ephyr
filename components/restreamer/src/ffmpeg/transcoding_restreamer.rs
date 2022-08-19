//! Kind of a [FFmpeg] re-streaming process that re-streams a live stream from
//! one URL endpoint to another one transcoding it with desired settings, and
//! optionally transmuxing it to the destination format.
//!
//! [FFmpeg]: https://ffmpeg.org

use std::borrow::Cow;

use tokio::process::Command;
use url::Url;
use uuid::Uuid;

/// Kind of a [FFmpeg] re-streaming process that re-streams a live stream from
/// one URL endpoint to another one transcoding it with desired settings, and
/// optionally transmuxing it to the destination format.
///
/// [FFmpeg]: https://ffmpeg.org
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscodingRestreamer {
    /// ID of an element in a [`State`] this [`TranscodingRestreamer`] process
    /// is related to.
    ///
    /// [`State`]: crate::state::State
    pub id: Uuid,

    /// [`Url`] to pull a live stream from.
    pub from_url: Url,

    /// [`Url`] to publish the transcoded live stream onto.
    pub to_url: Url,

    /// [FFmpeg video encoder][1] to encode the transcoded live stream with.
    ///
    /// [1]: https://ffmpeg.org/ffmpeg-codecs.html#Video-Encoders
    pub vcodec: Option<Cow<'static, str>>,

    /// [Preset] of the [`TranscodingRestreamer::vcodec`] if it has one.
    ///
    /// [Preset]: https://trac.ffmpeg.org/wiki/Encode/H.264#Preset
    pub vpreset: Option<Cow<'static, str>>,

    /// [Profile] of the [`TranscodingRestreamer::vcodec`] if it has one.
    ///
    /// [Profile]: https://trac.ffmpeg.org/wiki/Encode/H.264#Profile
    pub vprofile: Option<Cow<'static, str>>,

    /// [FFmpeg audio encoder][1] to encode the transcoded live stream with.
    ///
    /// [1]: https://ffmpeg.org/ffmpeg-codecs.html#Audio-Encoders
    pub acodec: Option<Cow<'static, str>>,
}

impl TranscodingRestreamer {
    /// Checks whether this [`TranscodingRestreamer`] process must be restarted,
    /// as cannot apply the new `actual` params on itself correctly, without
    /// interruptions.
    #[inline]
    #[must_use]
    pub fn needs_restart(&self, actual: &Self) -> bool {
        self != actual
    }

    /// Properly setups the given [FFmpeg] [`Command`] for this
    /// [`TranscodingRestreamer`] before running it.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    pub(crate) fn setup_ffmpeg(&self, cmd: &mut Command) {
        match self.from_url.scheme() {
            "http" | "https" | "rtmp" | "rtmps" => (),
            "file" => {
                let _ = cmd.arg("-re").args(&["-stream_loop", "-1"]);
            }
            _ => unimplemented!(),
        }

        let _ = cmd.args(&["-i", self.from_url.as_str()]);

        if let Some(val) = self.vcodec.as_ref() {
            let _ = cmd.args(&["-c:v", val]);
        }
        if let Some(val) = self.vpreset.as_ref() {
            let _ = cmd.args(&["-preset", val]);
        }
        if let Some(val) = self.vprofile.as_ref() {
            let _ = cmd.args(&["-profile:v", val]);
        }

        if let Some(val) = self.acodec.as_ref() {
            let _ = cmd.args(&["-c:a", val]);
        }

        let _ = match self.to_url.scheme() {
            "rtmp" | "rtmps" => cmd.args(&["-f", "flv"]),
            _ => unimplemented!(),
        }
        .arg(self.to_url.as_str());
    }
}
