//! Information about status of all [`Input`]s and [`Output`]s and
//! server health info (CPU usage, memory usage, etc.)
//!
//! [`Input`]: crate::state::Input
//! [`Output`]: crate::state::Output
use crate::state::Status;
use anyhow::anyhow;
use chrono::Utc;

use derive_more::{Deref, Display, Into};
use juniper::{
    GraphQLObject, GraphQLScalar, InputValue, ParseScalarResult,
    ParseScalarValue, ScalarToken, ScalarValue, Value,
};
use regex::Regex;

use crate::{stream_probe::StreamInfo, types::UNumber};
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

/// Statistics of statuses in [`Input`]s or [`Output`]s of [`Client`]
///
/// [`Input`]: crate::state::Input
/// [`Output`]: crate::state::Output
#[derive(Clone, Debug, Eq, GraphQLObject, PartialEq)]
pub struct StatusStatistics {
    /// Status of [`Input`]s or [`Output`]
    ///
    /// [`Input`]: crate::state::Input
    /// [`Output`]: crate::state::Output
    pub status: Status,

    /// Count of items having [`Status`]
    /// GraphQLScalar requires i32 numbers
    pub count: i32,
}

/// Information about status of all [`Input`]s and [`Output`]s and
/// server health info (CPU usage, memory usage, etc.)
///
/// [`Input`]: crate::state::Input
/// [`Output`]: crate::state::Output
#[derive(Clone, Debug, GraphQLObject, PartialEq)]
pub struct ClientStatistics {
    /// Client title
    pub client_title: String,

    /// Time when statistics was taken
    pub timestamp: String,

    /// Count of inputs grouped by status
    pub inputs: Vec<StatusStatistics>,

    /// Count of outputs grouped by status
    pub outputs: Vec<StatusStatistics>,

    /// Info about server info (CPU, Memory, Network)
    pub server_info: ServerInfo,
}

impl ClientStatistics {
    /// Creates a new [`ClientStatistics`] object with snapshot of
    /// current client's statistics regarding [`Input`]s and [`Output`]s
    ///
    /// [`Input`]: crate::state::Input
    /// [`Output`]: crate::state::Output
    #[must_use]
    pub fn new(
        client_title: String,
        inputs: Vec<StatusStatistics>,
        outputs: Vec<StatusStatistics>,
        server_info: ServerInfo,
    ) -> Self {
        Self {
            client_title,
            timestamp: Utc::now().format("%d.%m.%Y %H:%M").to_string(),
            inputs,
            outputs,
            server_info,
        }
    }
}

/// Current state of [`ClientStatistics`] request
#[derive(Clone, Debug, GraphQLObject, PartialEq)]
pub struct ClientStatisticsResponse {
    /// Statistics data
    pub data: Option<ClientStatistics>,

    /// The top-level errors returned by the server.
    pub errors: Option<Vec<String>>,
}

/// Server's info
#[derive(
    Clone, Debug, Deserialize, Serialize, GraphQLObject, PartialEq, Default,
)]
pub struct ServerInfo {
    /// Total CPU usage, %
    pub cpu_usage: Option<f64>,

    /// CPU cores count
    pub cpu_cores: Option<i32>,

    /// Total RAM installed on current machine
    pub ram_total: Option<f64>,

    /// Free (available) RAM
    pub ram_free: Option<f64>,

    /// Network traffic, transferred last second
    pub tx_delta: Option<f64>,

    /// Network traffic, received last second
    pub rx_delta: Option<f64>,

    /// Error message
    pub error_msg: Option<String>,
}

impl ServerInfo {
    /// Updates cpu usage
    pub fn update_cpu(&mut self, cpu: Option<f64>) {
        self.cpu_usage = cpu;
    }

    /// Updates cpu cores
    pub fn update_cores(&mut self, cpu: Option<i32>) {
        self.cpu_cores = cpu;
    }

    /// Sets error message
    pub fn set_error(&mut self, msg: Option<String>) {
        self.error_msg = msg;
    }

    /// Updates ram usage
    pub fn update_ram(
        &mut self,
        ram_total: Option<f64>,
        ram_free: Option<f64>,
    ) {
        self.ram_total = ram_total;
        self.ram_free = ram_free;
    }

    /// Updates traffic usage
    pub fn update_traffic_usage(
        &mut self,
        tx_delta: Option<f64>,
        rx_delta: Option<f64>,
    ) {
        self.tx_delta = tx_delta;
        self.rx_delta = rx_delta;
    }
}

/// Client represents server with running `ephyr` app and can return some
/// statistics about status of [`Input`]s, [`Output`]s .
///
/// [`Input`]: crate::state::Input
/// [`Output`]: crate::state::Output
#[derive(Clone, Debug, GraphQLObject, PartialEq, Serialize, Deserialize)]
pub struct Client {
    /// Unique id of client. Url of the host.
    pub id: ClientId,

    /// Whether the client url is protected by base auth
    #[serde(default)]
    pub is_protected: bool,

    /// Statistics for this [`Client`].
    #[serde(skip)]
    pub statistics: Option<ClientStatisticsResponse>,
}

impl Client {
    /// Creates a new [`Client`] passing host or ip address as identity.
    #[must_use]
    pub fn new(client_id: &ClientId) -> Self {
        Self {
            id: client_id.clone(),
            is_protected: client_id.has_base_auth(),
            statistics: None,
        }
    }
}

/// ID of a [`Client`].
#[derive(
    Clone,
    Debug,
    Deref,
    Display,
    Eq,
    Hash,
    Into,
    PartialEq,
    Serialize,
    GraphQLScalar,
)]
#[graphql(with = Self)]
pub struct ClientId(Url);

impl ClientId {
    /// Constructs [`ClientId`] from string.
    #[must_use]
    pub fn new(url: Url) -> Self {
        Self(url)
    }

    /// Checks whether client id url is base auth ulr
    pub fn has_base_auth(&self) -> bool {
        let re = Regex::new(r"^(?P<protocol>.+?\\)(?P<username>.+?):(?P<password>.+?)@(?P<address>.+)$").unwrap();
        re.is_match(self.0.as_str())
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_output<S: ScalarValue>(&self) -> Value<S> {
        Value::scalar(self.0.as_str().to_owned())
    }

    fn from_input<S>(v: &InputValue<S>) -> Result<Self, String>
    where
        S: ScalarValue,
    {
        let s = v
            .as_scalar()
            .and_then(ScalarValue::as_str)
            .and_then(|s| Url::parse(s).ok())
            .map(Self::new);
        match s {
            None => Err(format!("Expected `String` or `Int`, found: {v}")),
            Some(e) => Ok(e),
        }
    }

    fn parse_token<S>(value: ScalarToken<'_>) -> ParseScalarResult<S>
    where
        S: ScalarValue,
    {
        <String as ParseScalarValue<S>>::from_str(value)
    }
}

impl<'de> Deserialize<'de> for ClientId {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::new(Url::deserialize(deserializer)?))
    }
}

/// Stream statistics
#[derive(
    Clone, Debug, Deserialize, Eq, Serialize, PartialEq, GraphQLObject,
)]
pub struct StreamStatistics {
    /// Name of audio codec.  Example: "aac"
    pub audio_codec_name: Option<String>,
    /// Stereo / Mono layout
    pub audio_channel_layout: Option<String>,
    // Audio sample rate. Example - "44100"
    pub audio_sample_rate: Option<String>,
    /// Count of audio channels. Example: 2
    pub audio_channels: Option<UNumber>,
    // Name of video codec. Example: "h264"
    pub video_codec_name: Option<String>,
    // Video frame rate (fps). Example: "30/1"
    pub video_r_frame_rate: Option<String>,
    /// Video width
    pub video_width: Option<UNumber>,
    // Video height
    pub video_height: Option<UNumber>,
    // Total bit rate
    pub bit_rate: Option<String>,
    // Error message, if we could not retrieve stream info
    pub error: Option<String>,
}

impl StreamStatistics {
    /// Constructs [`StreamStatistics`] from [`Result`]
    #[must_use]
    pub fn new(result: anyhow::Result<StreamInfo>) -> Self {
        match result {
            Err(e) => Self::create_error_instance(&e),
            Ok(info) => {
                let audio_stream = match info.find_stream("audio") {
                    Some(audio) => audio,
                    None => {
                        return Self::create_error_instance(&anyhow!(
                            "Can't find 'audio' stream"
                        ))
                    }
                };

                let video_stream = match info.find_stream("video") {
                    Some(video) => video,
                    None => {
                        return Self::create_error_instance(&anyhow!(
                            "Can't find 'video' stream"
                        ))
                    }
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
