//! Kind of a [FFmpeg] re-streaming process that re-streams a live stream from
//! one URL endpoint to another one transcoding it with desired settings, and
//! optionally transmuxing it to the destination format.
//!
//! [FFmpeg]: https://ffmpeg.org

use std::borrow::Cow;

use tokio::process::Command;
use url::Url;
use uuid::Uuid;

/// Options for transcoding video and audio streams.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscodingOptions {
    /// [FFmpeg video encoder][1] to encode the transcoded live stream with.
    ///
    /// [1]: https://ffmpeg.org/ffmpeg-codecs.html#Video-Encoders
    pub vcodec: Option<Cow<'static, str>>,

    /// [Preset] of the [`TranscodingOptions::vcodec`] if it has one.
    ///
    /// [Preset]: https://trac.ffmpeg.org/wiki/Encode/H.264#Preset
    pub vpreset: Option<Cow<'static, str>>,

    /// [Profile] of the [`TranscodingOptions::vcodec`] if it has one.
    ///
    /// [Profile]: https://trac.ffmpeg.org/wiki/Encode/H.264#Profile
    pub vprofile: Option<Cow<'static, str>>,

    /// [FFmpeg audio encoder][1] to encode the transcoded live stream with.
    ///
    /// [1]: https://ffmpeg.org/ffmpeg-codecs.html#Audio-Encoders
    pub acodec: Option<Cow<'static, str>>,

    /// [Maximum bitrate][1] for the output video stream.
    ///
    /// [1]: https://trac.ffmpeg.org/wiki/Encode/H.264#ConstrainedencodingVBVmaximumbitrate
    pub maxrate: Option<Cow<'static, str>>,

    /// Size of the output video stream buffer.
    pub bufsize: Option<Cow<'static, str>>,

    /// Audio sampling rate for the output audio stream.
    pub ar: Option<Cow<'static, str>>,

    /// Frames per second for the output video stream.
    pub fps: Option<Cow<'static, str>>,

    /// [Tune][1] to optimize encoding settings for a specific type of video content.
    ///
    /// [1]: https://trac.ffmpeg.org/wiki/Encode/H.264#Tune
    pub tune: Option<Cow<'static, str>>,
}

impl Default for TranscodingOptions {
    /// Returns a new [`TranscodingOptions`] struct with all fields set to `None`.
    fn default() -> Self {
        TranscodingOptions {
            vcodec: Some("libx264".into()),
            vprofile: Some("baseline".into()),
            vpreset: Some("superfast".into()),
            acodec: Some("aac".into()),
            // TODO: change to it on prod
            // acodec: Some("libfdk_aac".into()),
            maxrate: Some("8M".into()),
            bufsize: Some("16M".into()),
            ar: Some("48000".into()),
            fps: Some("25".into()),
            tune: Some("zerolatency".into()),
        }
    }
}

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

    /// [TranscodingOptions] options for [FFmpeg] transcoding.
    pub options: TranscodingOptions,
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
                _ = cmd.arg("-re").args(["-stream_loop", "-1"]);
            }
            _ => unimplemented!(),
        }
        // Setup input
        _ = cmd.args(["-i", self.from_url.as_str()]);
        let opts = &self.options;
        // Video options
        if let Some(val) = opts.vcodec.as_ref() {
            _ = cmd.args(["-c:v", val]);
        }
        if let Some(val) = opts.vpreset.as_ref() {
            _ = cmd.args(["-preset", val]);
        }
        if let Some(val) = opts.tune.as_ref() {
            let _ = cmd.args(["-tune", val]);
        }
        if let Some(val) = opts.vprofile.as_ref() {
            _ = cmd.args(["-profile:v", val]);
        }

        // Audio options
        if let Some(val) = opts.acodec.as_ref() {
            _ = cmd.args(["-c:a", val]);
        }
        if let Some(val) = opts.ar.as_ref() {
            let _ = cmd.args(["-ar", val]);
        }
        if let Some(val) = opts.maxrate.as_ref() {
            let _ = cmd.args(["-maxrate", val]);
        }

        // Output options
        if let Some(val) = opts.maxrate.as_ref() {
            _ = cmd.args(["-bufsize", val]);
        }
        if let Some(val) = opts.fps.as_ref() {
            _ = cmd.args(["-r", val]);
        }

        _ = match self.to_url.scheme() {
            "rtmp" | "rtmps" => cmd.args(["-f", "flv"]),
            _ => unimplemented!(),
        }
        .arg(self.to_url.as_str());
    }
}
