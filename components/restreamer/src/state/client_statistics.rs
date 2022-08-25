use crate::state::{ServerInfo, Status};
use chrono::{DateTime, Utc};
use juniper::GraphQLObject;

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
