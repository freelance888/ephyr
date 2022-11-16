///
pub fn ffprobe(url: Url) -> anyhow<StreamInfo> {
    let mut cmd = std::process::Command::new("ffprobe");

    let entries = [
        "format=bit_rate:stream=codec_type",
        "codec_name",
        "channel_layout",
        "sample_rate",
        "channels",
        "bit_rate",
        "r_frame_rate",
        "width",
        "height",
    ]
    .join(",");

    // Default args.
    cmd.args(&["-v", "quiet", "-show_entries", entries, "-of", "json"]);
    cmd.arg(path);

    let out = cmd.output()?;

    if !out.status.success() {
        return Err(out);
    }

    serde_json::from_slice::<FfProbe>(&out.stdout)
        .map_err(FfProbeError::Deserialize)
}

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

#[derive(
    Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(
    feature = "__internal_deny_unknown_fields",
    serde(deny_unknown_fields)
)]
pub struct Stream {
    pub index: i64,
    pub codec_name: Option<String>,
    pub sample_aspect_ratio: Option<String>,
    pub display_aspect_ratio: Option<String>,
    pub color_range: Option<String>,
    pub color_space: Option<String>,
    pub bits_per_raw_sample: Option<String>,
    pub channel_layout: Option<String>,
    pub max_bit_rate: Option<String>,
    pub nb_frames: Option<String>,
    /// Number of frames seen by the decoder.
    /// Requires full decoding and is only available if the 'count_frames'
    /// setting was enabled.
    pub nb_read_frames: Option<String>,
    pub codec_long_name: Option<String>,
    pub codec_type: Option<String>,
    pub codec_time_base: Option<String>,
    pub codec_tag_string: String,
    pub codec_tag: String,
    pub sample_fmt: Option<String>,
    pub sample_rate: Option<String>,
    pub channels: Option<i64>,
    pub bits_per_sample: Option<i64>,
    pub r_frame_rate: String,
    pub avg_frame_rate: String,
    pub time_base: String,
    pub start_pts: Option<i64>,
    pub start_time: Option<String>,
    pub duration_ts: Option<i64>,
    pub duration: Option<String>,
    pub bit_rate: Option<String>,
    pub disposition: Disposition,
    pub tags: Option<StreamTags>,
    pub profile: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub coded_width: Option<i64>,
    pub coded_height: Option<i64>,
    pub closed_captions: Option<i64>,
    pub has_b_frames: Option<i64>,
    pub pix_fmt: Option<String>,
    pub level: Option<i64>,
    pub chroma_location: Option<String>,
    pub refs: Option<i64>,
    pub is_avc: Option<String>,
    pub nal_length: Option<String>,
    pub nal_length_size: Option<String>,
    pub field_order: Option<String>,
    pub id: Option<String>,
    #[serde(default)]
    pub side_data_list: Vec<SideData>,
}

#[derive(
    Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(
    feature = "__internal_deny_unknown_fields",
    serde(deny_unknown_fields)
)]
pub struct SideData {
    pub side_data_type: String,
}

#[derive(
    Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(
    feature = "__internal_deny_unknown_fields",
    serde(deny_unknown_fields)
)]
pub struct Disposition {
    pub default: i64,
    pub dub: i64,
    pub original: i64,
    pub comment: i64,
    pub lyrics: i64,
    pub karaoke: i64,
    pub forced: i64,
    pub hearing_impaired: i64,
    pub visual_impaired: i64,
    pub clean_effects: i64,
    pub attached_pic: i64,
    pub timed_thumbnails: i64,
}

#[derive(
    Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(
    feature = "__internal_deny_unknown_fields",
    serde(deny_unknown_fields)
)]
pub struct StreamTags {
    pub language: Option<String>,
    pub creation_time: Option<String>,
    pub handler_name: Option<String>,
    pub encoder: Option<String>,
}

#[derive(
    Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(
    feature = "__internal_deny_unknown_fields",
    serde(deny_unknown_fields)
)]
pub struct Format {
    pub filename: String,
    pub nb_streams: i64,
    pub nb_programs: i64,
    pub format_name: String,
    pub format_long_name: String,
    pub start_time: Option<String>,
    pub duration: Option<String>,
    // FIXME: wrap with Option<_> on next semver breaking release.
    #[serde(default)]
    pub size: String,
    pub bit_rate: Option<String>,
    pub probe_score: i64,
    pub tags: Option<FormatTags>,
}

impl Format {
    /// Get the duration parsed into a [`std::time::Duration`].
    pub fn try_get_duration(
        &self,
    ) -> Option<Result<std::time::Duration, std::num::ParseFloatError>> {
        self.duration
            .as_ref()
            .map(|duration| match duration.parse::<f64>() {
                Ok(num) => Ok(std::time::Duration::from_secs_f64(num)),
                Err(error) => Err(error),
            })
    }

    /// Get the duration parsed into a [`std::time::Duration`].
    ///
    /// Will return [`None`] if no duration is available, or if parsing fails.
    /// See [`Self::try_get_duration`] for a method that returns an error.
    pub fn get_duration(&self) -> Option<std::time::Duration> {
        self.try_get_duration()?.ok()
    }
}

#[derive(
    Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(
    feature = "__internal_deny_unknown_fields",
    serde(deny_unknown_fields)
)]
pub struct FormatTags {
    #[serde(rename = "WMFSDKNeeded")]
    pub wmfsdkneeded: Option<String>,
    #[serde(rename = "DeviceConformanceTemplate")]
    pub device_conformance_template: Option<String>,
    #[serde(rename = "WMFSDKVersion")]
    pub wmfsdkversion: Option<String>,
    #[serde(rename = "IsVBR")]
    pub is_vbr: Option<String>,
    pub major_brand: Option<String>,
    pub minor_version: Option<String>,
    pub compatible_brands: Option<String>,
    pub creation_time: Option<String>,
    pub encoder: Option<String>,
}
