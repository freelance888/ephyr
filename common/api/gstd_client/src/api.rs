//! [GStreamer Daemon HTTP][1] API structures.
//!
//! [1]: https://developer.ridgerun.com/wiki/
//! index.php/GStreamer_Daemon_-_HTTP_API
#![allow(unreachable_pub)]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Param {
    pub description: String,
    pub r#type: String,
    pub access: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Integer(i32),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Property {
    pub name: String,
    pub value: PropertyValue,
    pub param: Param,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ResponseT {
    Bus(Bus),
    Properties(Properties),
    Property(Property),
}

/// Successful response, returned by [GStreamer Daemon][1] API.
///
/// [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    pub code: i32,
    pub description: String,
    pub response: ResponseT,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Properties {
    pub properties: Vec<Property>,
    pub nodes: Vec<Node>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bus {
    pub r#type: String,
    pub source: String,
    pub timestamp: String,
    pub seqnum: i64,
    pub message: String,
    pub debug: String,
}
