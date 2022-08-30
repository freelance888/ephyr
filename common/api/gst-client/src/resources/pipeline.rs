//! Define [`Pipeline`] which encapsulate methods
//! of [Pipeline API][1]
//!
//! The actual pipeline is [GStreamer] [GstPipeline][2]
//!
//! [GStreamer]: https://gstreamer.freedesktop.org/
//! [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon_-_C_API#Pipelines
//! [2]: https://gstreamer.freedesktop.org/documentation/additional/design/gstpipeline.html
use crate::{
    gstd_types,
    resources::{bus::PipelineBus, element::PipelineElement},
    Error, GstClient,
};

/// Performs requests to `pipelines/` endpoint
#[derive(Debug, Clone)]
pub struct Pipeline {
    /// name of the pipeline
    pub name: String,
    pub(crate) client: GstClient,
}

impl Pipeline {
    pub(crate) fn new(name: &str, client: &GstClient) -> Self {
        Self {
            name: name.to_owned(),
            client: client.clone(),
        }
    }
    /// Creates a new pipeline .
    ///
    /// Performs `POST pipelines?name={name}&description={description}`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Arguments
    ///
    /// * `description` - pipeline with gst-launch syntax
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn create(&self, description: &str) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .post(&format!(
                "pipelines?name={}&description={description}",
                self.name
            ))
            .await?;
        self.client.process_resp(resp).await
    }

    /// Performs `GET /pipelines/{name}/graph` API request, returning the
    /// parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn graph(&self) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .get(&format!("pipelines/{}/graph", self.name))
            .await?;
        self.client.process_resp(resp).await
    }
    /// Performs `GET /pipelines/{name}/elements`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn elements(&self) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .get(&format!("pipelines/{}/elements", self.name))
            .await?;
        self.client.process_resp(resp).await
    }

    /// Performs `GET /pipelines/{name}/properties`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn properties(&self) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .get(&format!("pipelines/{}/properties", self.name))
            .await?;
        self.client.process_resp(resp).await
    }

    /// Operate with [GStreamer Daemon][1] pipeline element.
    ///
    /// # Arguments
    ///
    /// * `name` - name of the element in the pipeline
    ///
    /// [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
    pub fn element(&self, name: &str) -> PipelineElement {
        PipelineElement::new(name, self)
    }
    /// Operate with [GStreamer Daemon][1] pipeline bus.
    ///
    /// [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
    pub fn bus(&self) -> PipelineBus {
        PipelineBus::new(self)
    }

    /// Performs `POST pipelines/{name}/event?name=eos`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn event_eos(&self) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .post(&format!("pipelines/{}/event?name=eos", self.name))
            .await?;
        self.client.process_resp(resp).await
    }
    /// Performs `POST pipelines/{name}/event?name=flush_start`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn create_event_flush_start(&self) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .post(&format!("pipelines/{}/event?name=flush_start", self.name))
            .await?;
        self.client.process_resp(resp).await
    }
    /// Performs `POST pipelines/{name}/event?name=flush_stop`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn create_event_flush_stop(&self) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .post(&format!("pipelines/{}/event?name=flush_stop", self.name))
            .await?;
        self.client.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}/state?name=playing`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn play(&self) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .put(&format!("pipelines/{}/state?name=playing", self.name))
            .await?;
        self.client.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}/state?name=paused`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn pause(&self) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .put(&format!("pipelines/{}/state?name=paused", self.name))
            .await?;
        self.client.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}/state?name=stop`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn stop(&self) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .put(&format!("pipelines/{}/state?name=stop", self.name))
            .await?;
        self.client.process_resp(resp).await
    }

    /// Performs `PUT pipelines/{name}/verbose?name={value}`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn set_verbose(&self, value: bool) -> Result<gstd_types::Response, Error> {
        let val = if value { "true" } else { "false" };
        let resp = self
            .client
            .put(&format!("pipelines/{}/verbose?name={val}", self.name))
            .await?;
        self.client.process_resp(resp).await
    }

    /// Performs `DELETE pipelines/{name}/`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn delete(&self) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .delete(&format!("pipelines/{}", self.name))
            .await?;
        self.client.process_resp(resp).await
    }
}
