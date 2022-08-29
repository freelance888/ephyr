//! Defining library errors.
use derive_more::{Display, Error};

/// Possible errors of performing [`GstdClient`] requests.
///
/// [`GstdClient`]: crate::GstdClient
#[derive(Debug, Display, Error)]
pub enum Error {
    /// Performing HTTP request failed itself.
    #[display(fmt = "Failed to perform HTTP request: {}", _0)]
    RequestFailed(reqwest::Error),

    /// [`GstdClient`] responded with a bad [`StatusCode`].
    ///
    /// [`StatusCode`]: reqwest::StatusCode
    /// [`GstdClient`]: crate::GstdClient
    #[display(fmt = "API responded with bad status: {}", _0)]
    BadStatus(#[error(not(source))] reqwest::StatusCode),

    /// [`GstdClient`] responded with a bad body, which cannot be deserialized.
    ///
    /// [`GstdClient`]: crate::GstdClient
    #[display(fmt = "Failed to decode API response: {}", _0)]
    BadBody(reqwest::Error),

    /// Failed to build [`GstdClient`] client because incorrect base Url
    ///
    /// [`GstdClient`]: crate::GstdClient
    #[display(fmt = "Failed to parse base URL: {}", _0)]
    IncorrectBaseUrl(url::ParseError),

    /// Failed to create [`GstdClient`] API Url
    ///
    /// [`GstdClient`]: crate::GstdClient
    #[display(fmt = "Failed to parse URL: {}", _0)]
    IncorrectApiUrl(url::ParseError),
}
