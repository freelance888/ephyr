use derive_more::{Display, Error};
use reqwest::{Client, Error as ReqwestError, Response as ReqwestResponse};

/// Possible errors of performing requests to [SRS HTTP API][1].
///
/// [1]: https://ossrs.io/lts/en-us/docs/v5/doc/http-api
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Display, Error)]
pub enum SrsClientError {
    /// Performing HTTP request failed itself.
    #[display(fmt = "Failed to perform HTTP request: {_0}")]
    RequestFailed(ReqwestError),

    /// [SRS HTTP API][1] responded with a bad [`StatusCode`].
    ///
    /// [`StatusCode`]: reqwest::StatusCode
    /// [1]: https://ossrs.io/lts/en-us/docs/v5/doc/http-callback
    #[display(fmt = "SRS HTTP API responded with bad status: {_0}")]
    BadStatus(#[error(not(source))] reqwest::StatusCode),

    /// Performing deserialize of [SRS HTTP API][1] response
    ///
    /// [1]: https://ossrs.io/lts/en-us/docs/v5/doc/http-callback
    #[display(fmt = "Failed to perform deserialize request: {_0}")]
    DeserializeError(ReqwestError),

    /// Failed to build [`SrsClient`] client because incorrect base Url
    ///
    /// [`SrsClient`]: crate::SrsClient
    #[display(fmt = "Failed to parse base URL: {_0}")]
    IncorrectBaseUrl(url::ParseError),

    /// Failed to create [`SrsClient`] API Url
    ///
    /// [`SrsClient`]: crate::SrsClient
    #[display(fmt = "Failed to parse URL: {_0}")]
    IncorrectApiUrl(url::ParseError),
}
