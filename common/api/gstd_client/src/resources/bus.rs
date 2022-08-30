use crate::resources::Pipeline;
use crate::{gstd_types, Error, GstClient};

/// Performs requests to `pipelines/{name}/bus` endpoints
#[derive(Debug, Clone)]
pub struct PipelineBus {
    client: GstClient,
    pipeline: Pipeline,
}

impl PipelineBus {
    pub(crate) fn new(pipeline: &Pipeline) -> Self {
        Self {
            pipeline: pipeline.clone(),
            client: pipeline.client.clone(),
        }
    }
    /// Performs `GET pipelines/{name}/bus/message`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn read(&self) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .get(&format!("pipelines/{}/bus/message", self.pipeline.name))
            .await?;
        self.client.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}?timeout={time_ns}`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn set_timeout(
        &self,
        time_ns: i32,
    ) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .put(&format!(
                "pipelines/{}/bus/timeout?name={time_ns}",
                self.pipeline.name
            ))
            .await?;
        self.client.process_resp(resp).await
    }
    /// Performs `PUT pipelines/{name}?types={filter}`
    /// API request, returning the parsed [`gstd_types::Response`]
    ///
    /// # Errors
    ///
    /// If API request cannot be performed, or fails.
    /// See [`Error`] for details.
    pub async fn set_filter(
        &self,
        filter: &str,
    ) -> Result<gstd_types::Response, Error> {
        let resp = self
            .client
            .put(&format!(
                "pipelines/{}/bus/types?name={filter}",
                self.pipeline.name
            ))
            .await?;
        self.client.process_resp(resp).await
    }
}
