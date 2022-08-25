use crate::state::Status;
use chrono::{DateTime, Utc};

use derive_more::{Deref, Display, Into};
use juniper::{
    GraphQLObject, GraphQLScalar, InputValue, ParseScalarResult,
    ParseScalarValue, ScalarToken, ScalarValue, Value,
};

use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

/// Statistics of statuses in [`Input`]s or [`Output`]s of [`Client`]
#[derive(Clone, Debug, Eq, GraphQLObject, PartialEq)]
pub struct StatusStatistics {
    /// Status of [`Input`]s or [`Output`]
    pub status: Status,

    /// Count of items having [`Status`]
    /// GraphQLScalar requires i32 numbers
    pub count: i32,
}

/// Information about status of all [`Input`]s and [`Output`]s and
/// server health info (CPU usage, memory usage, etc.)
#[derive(Clone, Debug, GraphQLObject, PartialEq)]
pub struct ClientStatistics {
    /// Client title
    pub client_title: String,

    /// Time when statistics was taken
    pub timestamp: DateTime<Utc>,

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
    #[must_use]
    pub fn new(
        client_title: String,
        inputs: Vec<StatusStatistics>,
        outputs: Vec<StatusStatistics>,
        server_info: ServerInfo,
    ) -> Self {
        Self {
            client_title,
            timestamp: Utc::now(),
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
