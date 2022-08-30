use crate::{api, Error};
use reqwest::{Client, Response};
use url::Url;

#[derive(Debug, Clone)]
pub struct GstPipeline {
    pub name: String,
    pub gst_client: GstClient,
}

impl GstPipeline {
    /// Performs `GET /pipelines/{name}/graph` API request, returning the
    /// parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn graph(&self) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .get(&format!("pipelines/{}/graph", self.name))
            .await?;
        self.gst_client.process_resp(resp).await
    }
    /// Performs `GET /pipelines/{name}/elements`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn elements(&self) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .get(&format!("pipelines/{}/elements", self.name))
            .await?;
        self.gst_client.process_resp(resp).await
    }

    /// Performs `GET /pipelines/{name}/properties`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn properties(&self) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .get(&format!("pipelines/{}/properties", self.name))
            .await?;
        self.gst_client.process_resp(resp).await
    }

    /// Performs `GET pipelines/{name}/elements/
    /// {element}/properties/{property}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn element_property(
        &self,
        element: &str,
        property: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .get(&format!(
                "pipelines/{}/elements/{element}/properties/{property}",
                self.name
            ))
            .await?;
        self.gst_client.process_resp(resp).await
    }
    /// Performs `GET pipelines/{name}/bus/message`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn bus_read(&self) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .get(&format!("pipelines/{}/bus/message", self.name))
            .await?;
        self.gst_client.process_resp(resp).await
    }
    /// Performs `GET pipelines/{name}/
    /// elements/{element}/signals/{signal}/callback`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn signal_connect(
        &self,
        element: &str,
        signal: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .get(&format!(
                "pipelines/{}/\
            elements/{element}/signals/{signal}/callback",
                self.name
            ))
            .await?;
        self.gst_client.process_resp(resp).await
    }

    /// Performs `GET pipelines/{name}/
    /// elements/{element}/signals/{signal}/disconnect`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn signal_disconnect(
        &self,
        element: &str,
        signal: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .get(&format!(
                "pipelines/{}/\
            elements/{element}/signals/{signal}/disconnect",
                self.name
            ))
            .await?;
        self.gst_client.process_resp(resp).await
    }
    /// Performs `POST pipelines?name={name}&description={description}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn pipeline(
        &self,
        description: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .post(&format!(
                "pipelines?name={}&description={description}",
                self.name
            ))
            .await?;
        self.gst_client.process_resp(resp).await
    }
    /// Performs `POST pipelines/{name}/event?name=eos`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn event_eos(&self) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .post(&format!("pipelines/{}/event?name=eos", self.name))
            .await?;
        self.gst_client.process_resp(resp).await
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
    ) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .post(&format!("pipelines/{}/event?name=flush_start", self.name))
            .await?;
        self.gst_client.process_resp(resp).await
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
    ) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .post(&format!("pipelines/{}/event?name=flush_stop", self.name))
            .await?;
        self.gst_client.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}/state?name=playing`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn play(&self) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .put(&format!("pipelines/{}/state?name=playing", self.name))
            .await?;
        self.gst_client.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}/state?name=paused`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn pause(&self) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .put(&format!("pipelines/{}/state?name=paused", self.name))
            .await?;
        self.gst_client.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}/state?name=stop`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn stop(&self) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .put(&format!("pipelines/{}/state?name=stop", self.name))
            .await?;
        self.gst_client.process_resp(resp).await
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
        element: &str,
        property: &str,
        value: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .put(&format!(
                "pipelines/{}/elements/\
            {element}/properties/{property}?name={value}",
                self.name
            ))
            .await?;
        self.gst_client.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}/verbose?name={value}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn set_verbose(
        &self,
        value: bool,
    ) -> Result<api::Response, Error> {
        let val = if value { "true" } else { "false" };
        let resp = self
            .gst_client
            .put(&format!("pipelines/{}/verbose?name={val}", self.name))
            .await?;
        self.gst_client.process_resp(resp).await
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
        time_ns: i32,
    ) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .put(&format!(
                "pipelines/{}/bus/timeout?name={time_ns}",
                self.name
            ))
            .await?;
        self.gst_client.process_resp(resp).await
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
        filter: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .put(&format!("pipelines/{}/bus/types?name={filter}", self.name))
            .await?;
        self.gst_client.process_resp(resp).await
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
        element: &str,
        signal: &str,
        timeout: &str,
    ) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .put(&format!(
                "pipelines/{}/\
            elements/{element}/signals/{signal}/timeout?name={timeout}",
                self.name
            ))
            .await?;
        self.gst_client.process_resp(resp).await
    }
    /// Performs `DELETE pipelines/{name}/`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn delete(&self) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .delete(&format!("pipelines/{}", self.name))
            .await?;
        self.gst_client.process_resp(resp).await
    }
}

#[derive(Debug, Clone)]
pub struct GstDebug {
    pub gst_client: GstClient,
}

impl GstDebug {
    /// Performs `PUT debug/enable?name=enable`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn enable(&self) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .put(&format!("debug/enable?name=true"))
            .await?;
        self.gst_client.process_resp(resp).await
    }

    /// Performs `PUT debug/enable?name=false`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn disable(&self) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .put(&format!("debug/enable?name=false"))
            .await?;
        self.gst_client.process_resp(resp).await
    }

    /// Performs `PUT debug/reset?name={value}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn reset(&self, value: bool) -> Result<api::Response, Error> {
        let val = if value { "true" } else { "false" };
        let resp = self
            .gst_client
            .put(&format!("debug/reset?name={val}"))
            .await?;
        self.gst_client.process_resp(resp).await
    }
    /// Performs `PUT debug/threshold?name={value}`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn threshold(&self, value: &str) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .put(&format!("debug/threshold?name={value}"))
            .await?;
        self.gst_client.process_resp(resp).await
    }
    /// Performs `PUT debug/color?name=true`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn enable_color(&self) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .put(&format!("debug/color?name=true"))
            .await?;
        self.gst_client.process_resp(resp).await
    }
    /// Performs `PUT debug/color?name=false`
    /// API request, returning the parsed [`api::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn disable_color(&self) -> Result<api::Response, Error> {
        let resp = self
            .gst_client
            .put(&format!("debug/color?name=false"))
            .await?;
        self.gst_client.process_resp(resp).await
    }
}
/// [`GstdClient`] for [GStreamer Daemon][1] API.
///
/// [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
#[derive(Debug, Clone)]
pub struct GstClient {
    http_client: Client,
    base_url: Url,
}

impl GstClient {
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
    pub async fn pipelines(&self) -> Result<api::Response, Error> {
        let resp = self.get("pipelines").await?;
        self.process_resp(resp).await
    }

    pub fn pipeline(&self, name: &str) -> GstPipeline {
        GstPipeline {
            name: name.to_owned(),
            gst_client: self.clone(),
        }
    }
    pub fn debug(&self) -> GstDebug {
        GstDebug {
            gst_client: self.clone(),
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
                .element_property("rtmp2src", "location")
                .await;
            println!("{:?}", res);
            assert!(res.is_ok());
        };
    }
    #[tokio::test]
    async fn retrieve_pipeline_bus_read() {
        if let Ok(client) = GstClient::build(&BASE_URL) {
            let res = client.pipeline("test-pipeline").bus_read().await;
            println!("{:?}", res);
            assert!(res.is_ok());
        };
    }
}
