//! [GStreamer Daemon HTTP][1] API structures.
//!
//! [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon_-_HTTP_API
#![allow(unreachable_pub, missing_docs)]

use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Response returned by [GStreamer Daemon][1] API.
///
/// [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Status of response.
    pub code: ResponseCode,
    /// Description of command response.
    /// Same as [`Response::code`] but with text
    pub description: String,
    /// The actual response data from the server
    pub response: ResponseT,
}

/// Response Codes for [`Response`] of [GStD]
///
/// [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Debug, Clone, Copy, Error, Display)]
#[repr(u8)]
pub enum ResponseCode {
    ///Everything went OK
    Success = 0,
    /// A mandatory argument was passed NULL
    NullArgument = 1,
    /// A bad pipeline description was provided
    BadDescription = 2,
    /// The name trying to be used already exists
    ExistingName = 3,
    /// Missing initialization
    MissingInitialization = 4,
    /// The requested pipeline was not found
    NoPipeline = 5,
    /// The requested resource was not found
    NoResource = 6,
    /// Cannot create a resource in the given property
    NoCreate = 7,
    /// The resource to create already exists
    ExistingResource = 8,
    /// Cannot update the given property
    NoUpdate = 9,
    /// Unknown command
    BadCommand = 10,
    /// Cannot read the given resource
    NoRead = 11,
    ///Cannot connect
    NoConnection = 12,
    /// The given value is incorrect
    BadValue = 13,
    /// Failed to change state of a pipeline
    StateError = 14,
    /// Failed to start IPC
    IpcError = 15,
    /// Unknown event
    EventError = 16,
    /// Incomplete arguments in user input
    MissingArgument = 17,
    /// Missing name of the pipeline
    MissingName = 18,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ResponseT {
    Bus(Option<Bus>),
    Properties(Properties),
    Property(Property),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Param {
    pub description: String,
    pub r#type: String,
    pub access: String,
}

/// Possible result in [`Response::response`] after
/// `GET /pipelines` API request
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Properties {
    pub properties: Vec<Property>,
    pub nodes: Vec<Node>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node {
    /// The name of [GStreamer element]
    ///
    /// [GStreamer element]: https://gstreamer.freedesktop.org/documentation/
    /// application-development/basics/elements.html
    pub name: String,
}

/// Possible result in [`Response::response`] after
/// `GET /pipelines/{pipeline_name}/graph` API request
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Property {
    pub name: String,
    pub value: PropertyValue,
    pub param: Param,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Integer(i32),
}

/// Possible result in [`Response::response`] after
/// `GET /pipelines/{name}/bus/message` API request
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bus {
    pub r#type: String,
    pub source: String,
    pub timestamp: String,
    pub seqnum: i64,
    pub message: String,
    pub debug: String,
}
