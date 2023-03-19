//! Logging tools and their initialization.

#![deny(
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations,
    nonstandard_style,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code
)]
#![warn(
    deprecated_in_future,
    missing_docs,
    unreachable_pub,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

mod capture_logs;

pub use capture_logs::{ChildCapture, ParsedMsg};
use std::net::IpAddr;

use opentelemetry::sdk::propagation::TraceContextPropagator;
use tracing::level_filters::LevelFilter;
pub use tracing::{self, Level, Span};
pub use tracing_actix_web;
use tracing_forest::ForestLayer;
pub use tracing_futures::Instrument;
pub use tracing_log::log;
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, Layer, Registry};

/// Allow to configure the tracing.
#[derive(Clone, Debug)]
pub struct TelemetryConfig {
    /// Endpoint of Jaeger server to send logs to.
    pub jaeger_endpoint: Option<String>,
    /// Service name to collect logs on Jaeger.
    pub jaeger_service_name: Option<String>,
    /// Logging level
    pub level: LevelFilter,
}

impl TelemetryConfig {
    /// Create  [`TelemetryConfig`] with the given verbosity `level`
    /// ([`Info`] by default, if [`None`]).
    ///
    /// [`Info`]: tracing::Level::INFO
    #[must_use]
    pub fn new(level: Option<Level>) -> Self {
        Self {
            level: LevelFilter::from(level.unwrap_or(Level::INFO)),
            jaeger_endpoint: None,
            jaeger_service_name: None,
        }
    }

    /// Set Jaeger endpoint to send traces to.
    #[must_use]
    pub fn jaeger_endpoint(
        mut self,
        agent_ip: Option<IpAddr>,
        agent_port: Option<u16>,
    ) -> Self {
        if let (Some(ip), Some(port)) = (agent_ip, agent_port) {
            self.jaeger_endpoint = Some(format!("{ip}:{port}"));
        };
        self
    }

    /// Set Jaeger service name to collect logs on Jaeger.
    #[must_use]
    pub fn jaeger_service_name(mut self, service_name: String) -> Self {
        self.jaeger_service_name = Some(service_name);
        self
    }

    /// Initialize the logging and telemetry.
    /// If [`TelemetryConfig.jaeger_endpoint`] is set,
    /// the telemetry will be sent to Jaeger
    ///
    /// # Panics
    ///
    /// If failed to initialize logger.
    pub fn init(self) {
        if let Err(e) = LogTracer::init() {
            panic!("Failed to initialize logger: {e}");
        };

        let mut layers = vec![ForestLayer::default().boxed()];

        if let Some(endpoint) = self.jaeger_endpoint {
            let tracer = opentelemetry_jaeger::new_agent_pipeline()
                .with_endpoint(endpoint)
                .with_service_name(
                    self.jaeger_service_name.unwrap_or("unknown".into()),
                )
                .install_simple()
                .expect("Failed to install jaeger agent");
            opentelemetry::global::set_text_map_propagator(
                TraceContextPropagator::new(),
            );
            layers.push(
                tracing_opentelemetry::layer().with_tracer(tracer).boxed(),
            );
        }

        let subscriber = Registry::default().with(self.level).with(layers);

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting tracing subscriber failed");
    }
}
