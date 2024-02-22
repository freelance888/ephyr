//! [HTTP API] definitions of [SRS].
//!
//! [SRS]: https://ossrs.io/
//! [1]: https://ossrs.io/lts/en-us/docs/v5/doc/http-api
#![allow(unused_imports)]

mod client;
mod common;
mod error;
mod feature;
mod meminfos;
mod response;
mod rusages;
mod self_proc_stats;
mod stream;
mod summary;
mod system_proc_stats;
mod vhost;

pub use error::SrsClientError;
pub use response::{SrsClientResp, SrsClientRespData};

use reqwest::{Client, Response as ReqwestResponse};
use url::Url;

/// Client for performing requests to [HTTP API][1] of spawned [SRS].
///
/// [SRS]: https://ossrs.io/
/// [1]: https://ossrs.io/lts/en-us/docs/v5/doc/http-api
#[derive(Clone, Debug)]
pub struct SrsClient {
    http_client: Client,
    base_url: Url,
}

impl SrsClient {
    /// Build [`SrsClient`] for future call to [HTTP API][1] API of spawned [SRS]. .
    ///
    /// # Errors
    ///
    /// If incorrect `base_url` passed
    ///
    /// [SRS]: https://ossrs.io/
    /// [1]: https://ossrs.io/lts/en-us/docs/v5/doc/http-api
    pub fn build<S: Into<String>>(base_url: S) -> Result<Self, SrsClientError> {
        let base_url = Url::parse(&base_url.into())
            .and_then(|url| url.join("/api/v1/"))
            .map_err(SrsClientError::IncorrectBaseUrl)?;
        tracing::debug!("base_url: {base_url}");
        Ok(Self {
            http_client: Client::new(),
            base_url,
        })
    }

    async fn get(&self, url: &str) -> Result<ReqwestResponse, SrsClientError> {
        self.http_client
            .get(
                self.base_url
                    .join(url)
                    .map_err(SrsClientError::IncorrectApiUrl)?,
            )
            .send()
            .await
            .map_err(SrsClientError::RequestFailed)
    }

    async fn delete(
        &self,
        url: &str,
    ) -> Result<ReqwestResponse, SrsClientError> {
        self.http_client
            .delete(
                self.base_url
                    .join(url)
                    .map_err(SrsClientError::IncorrectApiUrl)?,
            )
            .send()
            .await
            .map_err(SrsClientError::RequestFailed)
    }

    async fn process_resp(
        &self,
        resp: ReqwestResponse,
    ) -> Result<SrsClientResp, SrsClientError> {
        if !resp.status().is_success() {
            return Err(SrsClientError::BadStatus(resp.status()));
        }
        // tracing::debug!(url = resp.url().to_string(), "processing request");
        tracing::debug!("processing request to: {}", resp.url());
        let resp = resp
            .json::<SrsClientResp>()
            .await
            .map_err(SrsClientError::DeserializeError)?;
        Ok(resp)
    }

    /// [Kicks off][1] a client connected to [SRS] server by its `id`.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    ///
    /// [SRS]: https://ossrs.io/
    /// [1]: https://ossrs.io/lts/en-us/docs/v5/doc/http-api#kickoff-client
    pub async fn kickoff_client<T: Into<String>>(
        self,
        id: T,
    ) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.delete(&format!("clients/{}/", id.into())).await?;
        self.process_resp(resp).await
    }

    /// Retrieves the server version.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_version(self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("versions").await?;
        self.process_resp(resp).await
    }

    /// Manages all vhosts or a specified vhost.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_vhosts(self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("vhosts").await?;
        self.process_resp(resp).await
    }

    /// Manages all streams or a specified stream.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_streams(self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("streams").await?;
        self.process_resp(resp).await
    }

    /// Manages all clients or a specified client, default query top 10 clients.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_clients(self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("clients").await?;
        self.process_resp(resp).await
    }

    /// Retrieves the supported features of SRS.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_features(self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("features").await?;
        self.process_resp(resp).await
    }

    /// Retrieves the rusage of SRS.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_rusages(self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("rusages").await?;
        self.process_resp(resp).await
    }

    /// Retrieves the self process stats.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_self_proc_stats(
        self,
    ) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("self_proc_stats").await?;
        self.process_resp(resp).await
    }

    /// Retrieves the system process stats.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_system_proc_stats(
        self,
    ) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("system_proc_stats").await?;
        self.process_resp(resp).await
    }

    /// Retrieves the meminfo of system.
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`SrsClientError`](enum@SrsClientError)
    /// for details.
    pub async fn get_meminfos(self) -> Result<SrsClientResp, SrsClientError> {
        let resp = self.get("meminfos").await?;
        self.process_resp(resp).await
    }
}
