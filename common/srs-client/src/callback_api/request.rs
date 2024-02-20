//! [HTTP Callback API][1] of [SRS] exposed by application.
//!
//! [SRS]: https://ossrs.io/
//! [1]: https://ossrs.io/lts/en-us/docs/v5/doc/http-callback

use std::net::IpAddr;

use super::SrsCallbackEvent;
use serde::{Deserialize, Serialize};

/// Request performed by [SRS] to [HTTP Callback API][1].
///
/// [SRS]: https://ossrs.io/
/// [1]: https://ossrs.io/lts/en-us/docs/v5/doc/http-callback
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SrsCallbackReq {
    /// ID of the [SRS] server
    ///
    /// [SRS]: https://ossrs.io/
    pub server_id: String,

    /// Event that [SRS] reports about.
    ///
    /// [SRS]: https://ossrs.io/
    pub action: SrsCallbackEvent,

    /// ID of [SRS] client that happened event is related to.
    ///
    /// [SRS]: https://ossrs.io/
    pub client_id: String,

    /// IP address of [SRS] client that happened event is related to.
    ///
    /// [SRS]: https://ossrs.io/
    pub ip: IpAddr,

    /// [SRS] `vhost` ([virtual host][1]) of RTMP stream that happened event is
    /// related to.
    ///
    /// [SRS]: https://ossrs.io/
    /// [1]: https://github.com/ossrs/srs/wiki/migrate_v4_EN_rtmp-url-vhost
    pub vhost: String,

    /// [SRS] `app` of RTMP stream that happened event is related to.
    ///
    /// [SRS]: https://ossrs.io/
    pub app: String,

    /// [SRS] `stream` of RTMP stream that happened event is related to.
    ///
    /// [SRS]: https://ossrs.io/
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,
}

impl SrsCallbackReq {
    /// Combine [`SrsCallbackReq::app`] and [`SrsCallbackReq::stream`] fields.
    /// Uses for tracing
    #[allow(dead_code)]
    #[must_use]
    pub fn app_stream(&self) -> String {
        if let Some(stream) = &self.stream {
            format!("{}/{}", self.app, stream)
        } else {
            self.app.to_string()
        }
    }
}
