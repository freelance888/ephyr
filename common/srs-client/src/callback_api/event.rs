//! [HTTP Callback API][1] of [SRS] exposed by application.
//!
//! [SRS]: https://ossrs.io/
//! [1]: https://ossrs.io/lts/en-us/docs/v5/doc/http-callback

use derive_more::Display;
use serde::{Deserialize, Serialize};

/// Possible [SRS] events in [HTTP Callback API][1] that this application reacts
/// onto.
///
/// [SRS]: https://ossrs.io/
/// [1]: https://ossrs.io/lts/en-us/docs/v5/doc/http-callback
#[allow(clippy::enum_variant_names, clippy::module_name_repetitions)]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Display)]
#[serde(rename_all = "snake_case")]
pub enum SrsCallbackEvent {
    /// [SRS] client connects to [SRS] `app`.
    ///
    /// [SRS]: https://ossrs.io/
    OnConnect,

    /// [SRS] client publishes a new stream.
    ///
    /// [SRS]: https://ossrs.io/
    OnPublish,

    /// [SRS] client stops publishing its stream.
    ///
    /// [SRS]: https://ossrs.io/
    OnUnpublish,

    /// [SRS] client plays an existing stream.
    ///
    /// [SRS]: https://ossrs.io/
    OnPlay,

    /// [SRS] client stops playing an existing stream.
    ///
    /// [SRS]: https://ossrs.io/
    OnStop,

    /// [SRS] records an existing stream.
    ///
    /// [SRS]: https://ossrs.io/
    OnDvr,

    /// [SRS] client plays an existing stream via [HLS].
    ///
    /// [HLS]: https://en.wikipedia.org/wiki/HTTP_Live_Streaming
    /// [SRS]: https://ossrs.io/
    OnHls,
}
