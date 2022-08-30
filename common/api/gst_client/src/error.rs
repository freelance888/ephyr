//! Defining library errors.
use derive_more::{Display, Error};

/// Possible errors of performing [`GstClient`] requests.
///
/// [`GstClient`]: crate::GstClient
#[derive(Debug, Display, Error)]
pub enum Error {
    /// Performing HTTP request failed itself.
    #[display(fmt = "Failed to perform HTTP request: {}", _0)]
    RequestFailed(reqwest::Error),

    /// [`GstClient`] responded with a bad [`StatusCode`].
    ///
    /// [`StatusCode`]: reqwest::StatusCode
    /// [`GstClient`]: crate::GstClient
    #[display(fmt = "API responded with bad status: {}", _0)]
    BadStatus(#[error(not(source))] reqwest::StatusCode),

    /// [`GstClient`] responded with a bad body, which cannot be deserialized.
    ///
    /// [`GstClient`]: crate::GstClient
    #[display(fmt = "Failed to decode API response: {}", _0)]
    BadBody(reqwest::Error),

    /// Failed to build [`GstClient`] client because incorrect base Url
    ///
    /// [`GstClient`]: crate::GstClient
    #[display(fmt = "Failed to parse base URL: {}", _0)]
    IncorrectBaseUrl(url::ParseError),

    /// Failed to create [`GstClient`] API Url
    ///
    /// [`GstClient`]: crate::GstClient
    #[display(fmt = "Failed to parse URL: {}", _0)]
    IncorrectApiUrl(url::ParseError),
}
