//! [HTTP API] definitions of [SRS].
//!
//! [SRS]: https://github.com/ossrs/srs
//! [1]: https://github.com/ossrs/srs/wiki/v4_EN_HTTPApi

pub mod callback;

use derive_more::{Display, Error};

/// Client for performing requests to [HTTP API][1] of locally spawned [SRS].
///
/// [SRS]: https://github.com/ossrs/srs
/// [1]: https://github.com/ossrs/srs/wiki/v4_EN_HTTPApi
#[derive(Clone, Copy, Debug)]
pub struct Client;

impl Client {
    /// [URL] of v1 [HTTP API][1] hosted by local [SRS].
    ///
    /// [SRS]: https://github.com/ossrs/srs
    /// [URL]: https://en.wikipedia.org/wiki/URL
    /// [1]: https://github.com/ossrs/srs/wiki/v4_EN_HTTPApi
    const V1_URL: &'static str = "http://127.0.0.1:8002/api/v1";

    /// [Kicks off][1] a client connected to [SRS] server by its `id`.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`Error`](enum@Error)
    /// for details.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    /// [1]: https://github.com/ossrs/srs/wiki/v4_EN_HTTPApi#kickoff-client
    pub async fn kickoff_client(id: String) -> Result<(), Error> {
        let resp = reqwest::Client::new()
            .delete(&format!("{}/clients/{}", Self::V1_URL, id))
            .send()
            .await
            .map_err(Error::RequestFailed)?;
        if !resp.status().is_success() {
            return Err(Error::BadStatus(resp.status()));
        }
        Ok(())
    }
}

/// Possible errors of performing requests to [SRS HTTP API][1].
///
/// [1]: https://github.com/ossrs/srs/wiki/v4_EN_HTTPApi
#[derive(Debug, Display, Error)]
pub enum Error {
    /// Performing HTTP request failed itself.
    #[display(fmt = "Failed to perform HTTP request: {}", _0)]
    RequestFailed(reqwest::Error),

    /// [SRS HTTP API][1] responded with a bad [`StatusCode`].
    ///
    /// [`StatusCode`]: reqwest::StatusCode
    /// [1]: https://github.com/ossrs/srs/wiki/v4_EN_HTTPApi
    #[display(fmt = "SRS HTTP API responded with bad status: {}", _0)]
    BadStatus(#[error(not(source))] reqwest::StatusCode),
}
