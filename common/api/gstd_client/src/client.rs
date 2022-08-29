use crate::{api, Error};
use reqwest::Client;
use url::Url;

/// [`GstdClient`] for [GStreamer Daemon][1] API.
///
/// [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
#[derive(Debug, Clone)]
pub struct GstdClient {
    http_client: Client,
    base_url: Url,
}

impl GstdClient {
    /// Build [`GstdClient`] for future call to [GStreamer Daemon][1] API.
    ///
    /// # Errors
    ///
    /// If incorrect `base_url` passed
    ///
    /// [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
    pub fn build(base_url: &str) -> Result<Self, Error> {
        Ok(Self {
            http_client: Client::new(),
            base_url: Url::parse(base_url).map_err(Error::IncorrectBaseUrl)?,
        })
    }

    /// Performs `GET /pipelines` API request, returning the
    /// parsed [`api::ResponseT`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails. See [`Error`]
    /// for details.
    pub async fn list_pipelines(&self) -> Result<api::ResponseT, Error> {
        let resp = self
            .http_client
            .get(
                self.base_url
                    .join("pipelines")
                    .map_err(Error::IncorrectApiUrl)?,
            )
            .send()
            .await
            .map_err(Error::RequestFailed)?;
        if !resp.status().is_success() {
            return Err(Error::BadStatus(resp.status()));
        }
        Ok(resp
            .json::<api::Response>()
            .await
            .map_err(Error::BadBody)?
            .response)
    }
}

#[cfg(test)]
mod spec {
    use super::*;
    const BASE_URL: &'static str = "http://10.211.55.4:5000";

    #[tokio::test]
    async fn retrieve_pipelines() {
        if let Ok(res) = GstdClient::build(&BASE_URL) {
            let res = res.list_pipelines().await;
            println!("{:?}", res);
            assert!(res.is_ok());
        };
    }
}
