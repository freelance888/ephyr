//! Stream statistics
use crate::{stream_probe::StreamInfo, types::UNumber};
use anyhow::anyhow;
use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};

/// Stream statistics
#[derive(
    Clone, Debug, Deserialize, Eq, Serialize, PartialEq, GraphQLObject,
)]
pub struct StreamStatistics {
    /// Name of audio codec.  Example: "aac"
    pub audio_codec_name: Option<String>,
    /// Stereo / Mono layout
    pub audio_channel_layout: Option<String>,
    /// Audio sample rate. Example - "44100"
    pub audio_sample_rate: Option<String>,
    /// Count of audio channels. Example: 2
    pub audio_channels: Option<UNumber>,
    /// Name of video codec. Example: "h264"
    pub video_codec_name: Option<String>,
    /// Video frame rate (fps). Example: "30/1"
    pub video_r_frame_rate: Option<String>,
    /// Video width
    pub video_width: Option<UNumber>,
    /// Video height
    pub video_height: Option<UNumber>,
    /// Total bit rate
    pub bit_rate: Option<String>,
    /// Error message, if we could not retrieve stream info
    pub error: Option<String>,
}

impl StreamStatistics {
    /// Constructs [`StreamStatistics`] from [`Result`]
    #[must_use]
    pub fn new(result: anyhow::Result<StreamInfo>) -> Self {
        match result {
            Err(e) => Self::create_error_instance(&e),
            Ok(info) => {
                let Some(audio_stream) = info.find_stream("audio") else {
                    return Self::create_error_instance(&anyhow!(
                        "Can't find 'audio' stream"
                    ))
                };

                let Some(video_stream) = info.find_stream("video") else {
                    return Self::create_error_instance(&anyhow!(
                        "Can't find 'video' stream"
                    ))
                };

                Self {
                    audio_codec_name: audio_stream.codec_name,
                    audio_channel_layout: audio_stream.channel_layout,
                    audio_sample_rate: audio_stream.sample_rate,
                    audio_channels: audio_stream
                        .channels
                        .map(|x| UNumber::new(x.into())),
                    video_codec_name: video_stream.codec_name,
                    video_r_frame_rate: video_stream.r_frame_rate,
                    video_width: video_stream.width.map(UNumber::new),
                    video_height: video_stream.height.map(UNumber::new),
                    bit_rate: info.format.bit_rate,
                    error: None,
                }
            }
        }
    }

    pub fn create_error_instance(e: &anyhow::Error) -> Self {
        Self {
            audio_codec_name: None,
            audio_channel_layout: None,
            audio_sample_rate: None,
            audio_channels: None,
            video_codec_name: None,
            video_r_frame_rate: None,
            video_width: None,
            video_height: None,
            bit_rate: None,
            error: Some(e.to_string()),
        }
    }
}
