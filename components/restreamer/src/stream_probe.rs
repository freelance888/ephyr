//! Extracting info about stream.
//!
//! [FFprobe]: https://ffmpeg.org/ffprobe.html

use anyhow::anyhow;
use std::process::Stdio;
use tokio::process::Command;

/// Gather information about `rtmp` stream
///
/// # Errors
///
/// If `ffprobe` command fails for some reason
/// Or if we fail to deserialize results of `ffprobe` command
pub async fn stream_probe(url: String) -> anyhow::Result<StreamInfo> {
    let mut cmd = Command::new("ffprobe");
    let entries = [
        "format=bit_rate:stream=codec_type",
        "codec_name",
        "channel_layout",
        "sample_rate",
        "channels",
        "r_frame_rate",
        "width",
        "height",
    ];

    // Default args.
    _ = cmd
        .args([
            "-v",
            "quiet",
            "-show_entries",
            entries.join(",").as_str(),
            "-of",
            "json",
            url.as_str(),
        ])
        .stdin(Stdio::null())
        .kill_on_drop(true);

    let out = cmd
        .output()
        .await
        .map_err(|e| anyhow!("Error of getting info with FFPROBE: {}", e))?;

    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stdout).to_string();
        return Err(anyhow!(err));
    }

    let result =
        serde_json::from_slice::<StreamInfo>(&out.stdout).map_err(|e| {
            anyhow!("Error of deserializing output of FFPROBE: {}", e)
        })?;

    anyhow::Ok(result)
}

/// Only valuable info about video and audio streams
#[derive(
    Default, Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(
    feature = "__internal_deny_unknown_fields",
    serde(deny_unknown_fields)
)]
pub struct StreamInfo {
    /// Video, audio streams
    pub streams: Vec<Stream>,
    /// Generic parameters of stream
    pub format: Format,
}

impl StreamInfo {
    /// Search for the stream with specified `stream_type`
    #[must_use]
    pub fn find_stream(&self, stream_type: &str) -> Option<Stream> {
        self.streams
            .clone()
            .into_iter()
            .find(|x| x.codec_type.clone().unwrap_or_default() == stream_type)
    }
}

/// Common structure for info about video and audio streams
#[derive(
    Default, Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(
    feature = "__internal_deny_unknown_fields",
    serde(deny_unknown_fields)
)]
pub struct Stream {
    /// Type of codec. Example: "audio" or "video"
    pub codec_type: Option<String>,
    /// Codec name. For audio and video streams. Example: "aac", "h264"
    pub codec_name: Option<String>,
    /// Video width
    pub width: Option<u16>,
    /// Video height
    pub height: Option<u16>,
    /// Frame rate (fps). Example: "30/1"
    pub r_frame_rate: Option<String>,
    /// Only for audio stream. Sample rate. Example: "44100"
    pub sample_rate: Option<String>,
    /// Only for audio stream. Count of channels. Example: 2
    pub channels: Option<u8>,
    /// Only for audio stream. Stereo or Mono. Example: "stereo"
    pub channel_layout: Option<String>,
}

/// Generic parameters of stream
#[derive(
    Default, Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(
    feature = "__internal_deny_unknown_fields",
    serde(deny_unknown_fields)
)]
pub struct Format {
    /// Total bitrate (audio + video)
    pub bit_rate: Option<String>,
}
