//! Defines [`GstClient`] for communication with
//! [GStreamer Daemon][1] API.
//!
//! [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
use crate::{gstd_types, resources, Error};
use reqwest::{Client, Response};
use url::Url;

/// [`GstClient`] for [GStreamer Daemon][1] API.
///
/// [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
#[derive(Debug, Clone)]
pub struct GstClient {
    http_client: Client,
    base_url: Url,
}

impl GstClient {
    /// Build [`GstClient`] for future call to [GStreamer Daemon][1] API.
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

    pub(crate) async fn get(&self, url: &str) -> Result<Response, Error> {
        self.http_client
            .get(self.base_url.join(url).map_err(Error::IncorrectApiUrl)?)
            .send()
            .await
            .map_err(Error::RequestFailed)
    }

    pub(crate) async fn post(&self, url: &str) -> Result<Response, Error> {
        self.http_client
            .post(self.base_url.join(url).map_err(Error::IncorrectApiUrl)?)
            .send()
            .await
            .map_err(Error::RequestFailed)
    }

    pub(crate) async fn put(&self, url: &str) -> Result<Response, Error> {
        self.http_client
            .put(self.base_url.join(url).map_err(Error::IncorrectApiUrl)?)
            .send()
            .await
            .map_err(Error::RequestFailed)
    }

    pub(crate) async fn delete(&self, url: &str) -> Result<Response, Error> {
        self.http_client
            .put(self.base_url.join(url).map_err(Error::IncorrectApiUrl)?)
            .send()
            .await
            .map_err(Error::RequestFailed)
    }

    pub(crate) async fn process_resp(&self, resp: Response) -> Result<gstd_types::Response, Error> {
        if !resp.status().is_success() {
            return Err(Error::BadStatus(resp.status()));
        }

        let res = resp
            .json::<gstd_types::Response>()
            .await
            .map_err(Error::BadBody)?;

        if res.code != gstd_types::ResponseCode::Success {
            return Err(Error::GstdError(res.code));
        }
        Ok(res)
    }

    /// Performs `GET /pipelines` API request, returning the
    /// parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn pipelines(&self) -> Result<gstd_types::Response, Error> {
        let resp = self.get("pipelines").await?;
        self.process_resp(resp).await
    }
    /// Operate with [GStreamer Daemon][1] pipelines.
    ///
    /// # Arguments
    ///
    /// * `name` - name of the pipeline
    ///
    /// [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
    pub fn pipeline(&self, name: &str) -> resources::Pipeline {
        resources::Pipeline::new(name, self)
    }
    /// Manage [GStreamer Daemon][1] Debug mode.
    ///
    /// [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
    pub fn debug(&self) -> resources::Debug {
        resources::Debug::new(self)
    }
}

impl Default for GstClient {
    fn default() -> Self {
        Self {
            http_client: Client::new(),
            base_url: Url::parse("http://127.0.0.1:5000").unwrap(),
        }
    }
}
#[cfg(test)]
mod spec {
    use super::*;
    const BASE_URL: &'static str = "http://10.211.55.4:5000";

    #[tokio::test]
    async fn retrieve_pipelines() {
        if let Ok(client) = GstClient::build(&BASE_URL) {
            let res = client.pipelines().await;
            println!("{:?}", res);
            assert!(res.is_ok());
        };
    }

    #[tokio::test]
    async fn retrieve_pipeline_graph() {
        if let Ok(client) = GstClient::build(&BASE_URL) {
            let res = client.pipeline("test-pipeline").graph().await;
            println!("{:?}", res);
            assert!(res.is_ok());
        };
    }

    #[tokio::test]
    async fn retrieve_pipeline_elements() {
        if let Ok(client) = GstClient::build(&BASE_URL) {
            let res = client.pipeline("test-pipeline").elements().await;
            println!("{:?}", res);
            assert!(res.is_ok());
        };
    }
    #[tokio::test]
    async fn retrieve_pipeline_properties() {
        if let Ok(client) = GstClient::build(&BASE_URL) {
            let res = client.pipeline("test-pipeline").properties().await;
            println!("{:?}", res);
            assert!(res.is_ok());
        };
    }
    #[tokio::test]
    async fn retrieve_pipeline_element_property() {
        if let Ok(client) = GstClient::build(&BASE_URL) {
            let res = client
                .pipeline("test-pipeline")
                .element("rtmp2src")
                .property("location")
                .await;
            println!("{:?}", res);
            assert!(res.is_ok());
        };
    }
    #[tokio::test]
    async fn retrieve_pipeline_bus_read() {
        if let Ok(client) = GstClient::build(&BASE_URL) {
            let res = client.pipeline("test-pipeline").bus().read().await;
            println!("{:?}", res);
            assert!(res.is_ok());
        };
    }
}
