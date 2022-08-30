//! Define [`PipelineElement`] which encapsulate methods
//! of [Elements Pipeline API][1]
//!
//! The actual element is [GStreamer] [GstElement][2] by itself
//!
//! [GStreamer]: https://gstreamer.freedesktop.org/
//! [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon_-_C_API#Elements
//! [2]: https://gstreamer.freedesktop.org/documentation/additional/design/gstelement.html
use crate::{gstd_types, resources::Pipeline, Error, GstClient};

/// Performs requests to
/// `pipelines/{name}/elements/{element}` endpoints
#[derive(Debug, Clone)]
pub struct PipelineElement {
    name: String,
    client: GstClient,
    pipeline: Pipeline,
}

impl PipelineElement {
    pub(crate) fn new(name: &str, pipeline: &Pipeline) -> Self {
        Self {
            name: name.to_owned(),
            client: pipeline.client.clone(),
            pipeline: pipeline.clone(),
        }
    }

    /// Performs `GET pipelines/{name}/elements/
    /// {element}/properties/{property}`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn property(&self, property: &str) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .get(&format!(
                "pipelines/{}/elements/{}/properties/{property}",
                self.pipeline.name, self.name
            ))
            .await?;
        self.client.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}/elements/
    /// {element}/properties/{property}?name={value}`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn set_property(
        &self,
        property: &str,
        value: &str,
    ) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .put(&format!(
                "pipelines/{}/elements/\
            {}/properties/{property}?name={value}",
                self.pipeline.name, self.name
            ))
            .await?;
        self.client.process_resp(resp).await
    }

    /// Performs `GET pipelines/{name}/
    /// elements/{element}/signals/{signal}/callback`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn signal_connect(&self, signal: &str) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .get(&format!(
                "pipelines/{}/\
            elements/{}/signals/{signal}/callback",
                self.pipeline.name, self.name
            ))
            .await?;
        self.client.process_resp(resp).await
    }

    /// Performs `GET pipelines/{name}/
    /// elements/{element}/signals/{signal}/disconnect`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn signal_disconnect(&self, signal: &str) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .get(&format!(
                "pipelines/{}/\
            elements/{}/signals/{signal}/disconnect",
                self.pipeline.name, self.name
            ))
            .await?;
        self.client.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}/
    /// elements/{element}/signals/{signal}/timeout?name={timeout}`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Arguments
    ///
    /// * `signal` - signal to connect
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn set_signal_timeout(
        &self,
        signal: &str,
        timeout: &str,
    ) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .put(&format!(
                "pipelines/{}/\
            elements/{}/signals/{signal}/timeout?name={timeout}",
                self.pipeline.name, self.name
            ))
            .await?;
        self.client.process_resp(resp).await
    }
}
