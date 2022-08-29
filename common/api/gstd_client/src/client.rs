use crate::{api, Error};
use reqwest::{Client, Response};
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

    async fn get(&self, url: &str) -> Result<Response, Error> {
        self.http_client
            .get(self.base_url.join(url).map_err(Error::IncorrectApiUrl)?)
            .send()
            .await
            .map_err(Error::RequestFailed)
    }

    async fn post(&self, url: &str) -> Result<Response, Error> {
        self.http_client
            .post(self.base_url.join(url).map_err(Error::IncorrectApiUrl)?)
            .send()
            .await
            .map_err(Error::RequestFailed)
    }

    async fn put(&self, url: &str) -> Result<Response, Error> {
        self.http_client
            .put(self.base_url.join(url).map_err(Error::IncorrectApiUrl)?)
            .send()
            .await
            .map_err(Error::RequestFailed)
    }

    async fn delete(&self, url: &str) -> Result<Response, Error> {
        self.http_client
            .put(self.base_url.join(url).map_err(Error::IncorrectApiUrl)?)
            .send()
            .await
            .map_err(Error::RequestFailed)
    }

    async fn process_resp(
        &self,
        resp: Response,
    ) -> Result<api::Response, Error> {
        if !resp.status().is_success() {
            return Err(Error::BadStatus(resp.status()));
        }

        Ok(resp.json::<api::Response>().await.map_err(Error::BadBody)?)
    }

    /// Performs `GET /pipelines` API request, returning the
    /// parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn list_pipelines(&self) -> Result<api::Response, Error> {
        let resp = self.get("pipelines").await?;
        self.process_resp(resp).await
    }

    /// Performs `GET /pipelines/{name}/graph` API request, returning the
    /// parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn pipeline_graph(
        &self,
        name: &str,
    ) -> Result<api::Response, Error> {
        let resp = self.get(&format!("pipelines/{name}/graph")).await?;
        self.process_resp(resp).await
    }
    /// Performs `GET /pipelines/{name}/elements`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn pipeline_elements(
        &self,
        name: &str,
    ) -> Result<api::Response, Error> {
        let resp = self.get(&format!("pipelines/{name}/elements")).await?;
        self.process_resp(resp).await
    }

    /// Performs `GET /pipelines/{name}/properties`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn pipeline_properties(
        &self,
        name: &str,
    ) -> Result<api::Response, Error> {
        let resp = self.get(&format!("pipelines/{name}/properties")).await?;
        self.process_resp(resp).await
    }

    /// Performs `GET pipelines/{name}/elements/
    /// {element}/properties/{property}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn pipeline_element_property(
        &self,
        name: &str,
        element: &str,
        property: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .get(&format!(
                "pipelines/{name}/elements/{element}/properties/{property}"
            ))
            .await?;
        self.process_resp(resp).await
    }
    /// Performs `GET pipelines/{name}/bus/message`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn pipeline_bus_read(
        &self,
        name: &str,
    ) -> Result<api::Response, Error> {
        let resp = self.get(&format!("pipelines/{name}/bus/message")).await?;
        self.process_resp(resp).await
    }
    /// Performs `GET pipelines/{name}/
    /// elements/{element}/signals/{signal}/callback`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn pipeline_signal_connect(
        &self,
        name: &str,
        element: &str,
        signal: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .get(&format!(
                "pipelines/{name}/\
            elements/{element}/signals/{signal}/callback"
            ))
            .await?;
        self.process_resp(resp).await
    }

    /// Performs `GET pipelines/{name}/
    /// elements/{element}/signals/{signal}/disconnect`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn pipeline_signal_disconnect(
        &self,
        name: &str,
        element: &str,
        signal: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .get(&format!(
                "pipelines/{name}/\
            elements/{element}/signals/{signal}/disconnect"
            ))
            .await?;
        self.process_resp(resp).await
    }
    /// Performs `POST pipelines?name={name}&description={description}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn create_pipeline(
        &self,
        name: &str,
        description: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .post(&format!(
                "pipelines/{name}?name={name}&description={description}"
            ))
            .await?;
        self.process_resp(resp).await
    }
    /// Performs `POST pipelines/{name}/event?name=eos`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn create_event_eos(
        &self,
        name: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .post(&format!("pipelines/{name}/event?name=eos"))
            .await?;
        self.process_resp(resp).await
    }
    /// Performs `POST pipelines/{name}/event?name=flush_start`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn create_event_flush_start(
        &self,
        name: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .post(&format!("pipelines/{name}/event?name=flush_start"))
            .await?;
        self.process_resp(resp).await
    }
    /// Performs `POST pipelines/{name}/event?name=flush_stop`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn create_event_flush_stop(
        &self,
        name: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .post(&format!("pipelines/{name}/event?name=flush_stop"))
            .await?;
        self.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}/state?name=playing`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn play_pipeline(
        &self,
        name: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .put(&format!("pipelines/{name}/state?name=playing"))
            .await?;
        self.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}/state?name=paused`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn paused_pipeline(
        &self,
        name: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .put(&format!("pipelines/{name}/state?name=paused"))
            .await?;
        self.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}/state?name=stop`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn stop_pipeline(
        &self,
        name: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .put(&format!("pipelines/{name}/state?name=stop"))
            .await?;
        self.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}/elements/
    /// {element}/properties/{property}?name={value}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn set_element(
        &self,
        name: &str,
        element: &str,
        property: &str,
        value: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .put(&format!(
                "pipelines/{name}/elements/\
            {element}/properties/{property}?name={value}"
            ))
            .await?;
        self.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}/verbose?name={value}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn set_pipeline_verbose(
        &self,
        name: &str,
        value: bool,
    ) -> Result<api::Response, Error> {
        let val = if value { "true" } else { "false" };
        let resp = self
            .put(&format!("pipelines/{name}/verbose?name={val}"))
            .await?;
        self.process_resp(resp).await
    }
    /// Performs `PUT debug/enable?name={value}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn set_debug(&self, value: bool) -> Result<api::Response, Error> {
        let val = if value { "true" } else { "false" };
        let resp = self.put(&format!("debug/enable?name={val}")).await?;
        self.process_resp(resp).await
    }
    /// Performs `PUT debug/reset?name={value}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn reset_debug(
        &self,
        value: bool,
    ) -> Result<api::Response, Error> {
        let val = if value { "true" } else { "false" };
        let resp = self.put(&format!("debug/reset?name={val}")).await?;
        self.process_resp(resp).await
    }
    /// Performs `PUT debug/threshold?name={value}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn set_debug_threshold(
        &self,
        value: &str,
    ) -> Result<api::Response, Error> {
        let resp = self.put(&format!("debug/threshold?name={value}")).await?;
        self.process_resp(resp).await
    }
    /// Performs `PUT debug/color?name={value}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn set_debug_color(
        &self,
        value: bool,
    ) -> Result<api::Response, Error> {
        let val = if value { "true" } else { "false" };
        let resp = self.put(&format!("debug/color?name={val}")).await?;
        self.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}?timeout={time_ns}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn set_bus_timeout(
        &self,
        name: &str,
        time_ns: i32,
    ) -> Result<api::Response, Error> {
        let resp = self
            .put(&format!("pipelines/{name}/bus/timeout?name={time_ns}"))
            .await?;
        self.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}?types={filter}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn set_bus_filter(
        &self,
        name: &str,
        filter: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .put(&format!("pipelines/{name}/bus/types?name={filter}"))
            .await?;
        self.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}/
    /// elements/{element}/signals/{signal}/timeout?name={timeout}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn set_signal_timeout(
        &self,
        name: &str,
        element: &str,
        signal: &str,
        timeout: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .put(&format!(
                "pipelines/{name}/\
            elements/{element}/signals/{signal}/timeout?name={timeout}"
            ))
            .await?;
        self.process_resp(resp).await
    }
    /// Performs `DELETE pipelines/{name}/`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn delete_pipeline(
        &self,
        name: &str,
    ) -> Result<api::Response, Error> {
        let resp = self.delete(&format!("pipelines/{name}")).await?;
        self.process_resp(resp).await
    }
}

#[cfg(test)]
mod spec {
    use super::*;
    const BASE_URL: &'static str = "http://10.211.55.4:5000";

    #[tokio::test]
    async fn retrieve_pipelines() {
        if let Ok(client) = GstdClient::build(&BASE_URL) {
            let res = client.list_pipelines().await;
            println!("{:?}", res);
            assert!(res.is_ok());
        };
    }

    #[tokio::test]
    async fn retrieve_pipeline_graph() {
        if let Ok(client) = GstdClient::build(&BASE_URL) {
            let res = client.pipeline_graph("test-pipeline").await;
            println!("{:?}", res);
            assert!(res.is_ok());
        };
    }

    #[tokio::test]
    async fn retrieve_pipeline_elements() {
        if let Ok(client) = GstdClient::build(&BASE_URL) {
            let res = client.pipeline_elements("test-pipeline").await;
            println!("{:?}", res);
            assert!(res.is_ok());
        };
    }
    #[tokio::test]
    async fn retrieve_pipeline_properties() {
        if let Ok(client) = GstdClient::build(&BASE_URL) {
            let res = client.pipeline_properties("test-pipeline").await;
            println!("{:?}", res);
            assert!(res.is_ok());
        };
    }
    #[tokio::test]
    async fn retrieve_pipeline_element_property() {
        if let Ok(client) = GstdClient::build(&BASE_URL) {
            let res = client
                .pipeline_element_property(
                    "test-pipeline",
                    "rtmp2src",
                    "location",
                )
                .await;
            println!("{:?}", res);
            assert!(res.is_ok());
        };
    }
    #[tokio::test]
    async fn retrieve_pipeline_bus_read() {
        if let Ok(client) = GstdClient::build(&BASE_URL) {
            let res = client.pipeline_bus_read("test-pipeline").await;
            println!("{:?}", res);
            assert!(res.is_ok());
        };
    }
}
