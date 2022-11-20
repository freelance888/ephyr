//! Extracting info about stream.
//!
//! [FFprobe]: https://ffmpeg.org/ffprobe.html

use anyhow::anyhow;
use juniper::GraphQLScalar;
use url::Url;

/// Gather information about `rtmp` stream
///
pub fn stream_probe(url: Url) -> anyhow::Result<StreamInfo> {
    let mut cmd = std::process::Command::new("ffprobe");

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
    cmd.args(&[
        "-v",
        "quiet",
        "-show_entries",
        entries.join(",").as_str(),
        "-of",
        "json",
    ]);
    cmd.arg(url.as_str());

    let out = cmd.output()?;

    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).to_string();
        return Err(anyhow!(err));
    }

    let result = serde_json::from_slice::<StreamInfo>(&out.stdout)?;
    anyhow::Ok(result)
}

/// Short and only valuable info about video and audio streams
#[derive(
    Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(
    feature = "__internal_deny_unknown_fields",
    serde(deny_unknown_fields)
)]
pub struct StreamInfo {
    pub streams: Vec<Stream>,
    pub format: Format,
}

// Common structure for info about video and audio streams
#[derive(
    Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(
    feature = "__internal_deny_unknown_fields",
    serde(deny_unknown_fields)
)]
pub struct Stream {
    pub codec_type: Option<String>,
    pub codec_name: Option<String>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub r_frame_rate: Option<String>,
    pub sample_rate: Option<String>,
    pub channels: Option<u8>,
    pub channel_layout: Option<String>,
}

///
#[derive(
    Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(
    feature = "__internal_deny_unknown_fields",
    serde(deny_unknown_fields)
)]
pub struct Format {
    pub bit_rate: Option<String>,
}
