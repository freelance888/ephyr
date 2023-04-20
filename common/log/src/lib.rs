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
use opentelemetry::{
    sdk::{propagation::TraceContextPropagator, trace, Resource},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use std::{net::IpAddr, str::FromStr};
use tracing::level_filters::LevelFilter;
pub use tracing::{self, Level, Span};
pub use tracing_actix_web;
pub use tracing_futures::Instrument;
pub use tracing_log::log;
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, layer::SubscriberExt, Layer, Registry};

/// Log display format type for use in layer
#[derive(Clone, Debug)]
pub enum LogFormat {
    /// Use `fmt::layer().compact()` layer
    COMPACT,
    /// Use `fmt::layer().json()` layer
    JSON,
}

impl FromStr for LogFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "compact" => Ok(LogFormat::COMPACT),
            "json" => Ok(LogFormat::JSON),
            _ => Err(format!("Invalid log format: {s}")),
        }
    }
}

/// Allow to configure the tracing.
#[derive(Clone, Debug)]
pub struct TelemetryConfig {
    /// Endpoint of [Opentelemetry] collector server to send logs to.
    ///
    /// [Opentelemetry]: https://opentelemetry.io
    pub otlp_endpoint: Option<String>,
    /// Service name to collect traces to [Opentelemetry] collector.
    ///
    /// [Opentelemetry]: https://opentelemetry.io
    pub service_name: Option<String>,
    /// Logging level
    pub level: LevelFilter,
    /// Logging output format
    pub log_format: LogFormat,
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
            otlp_endpoint: None,
            service_name: None,
            log_format: LogFormat::COMPACT,
        }
    }

    /// Set [Opentelemetry] collector endpoint to send traces to.
    ///
    /// [Opentelemetry]: https://opentelemetry.io
    #[must_use]
    pub fn otlp_endpoint(
        mut self,
        collector_ip: Option<IpAddr>,
        collector_port: Option<u16>,
    ) -> Self {
        if let (Some(ip), Some(port)) = (collector_ip, collector_port) {
            self.otlp_endpoint = Some(format!("http://{ip}:{port}"));
        };
        self
    }

    /// Set current service name to collect traces to [Opentelemetry] collector.
    ///
    /// [Opentelemetry]: https://opentelemetry.io
    #[must_use]
    pub fn service_name(mut self, service_name: String) -> Self {
        self.service_name = Some(service_name);
        self
    }

    /// Set log output format.
    #[must_use]
    pub fn log_format(mut self, log_type: Option<LogFormat>) -> Self {
        self.log_format = log_type.unwrap_or(LogFormat::COMPACT);
        self
    }

    /// Initialize the logging and telemetry.
    /// If [`TelemetryConfig.otlp_endpoint`] is set,
    /// the telemetry will be sent to [Opentelemetry] collector.
    ///
    /// # Panics
    ///
    /// If failed to initialize logger.
    ///
    /// [Opentelemetry]: https://opentelemetry.io
    pub fn init(self) {
        if let Err(e) = LogTracer::init() {
            panic!("Failed to initialize logger: {e}");
        };
        let service_name = self.service_name.unwrap_or("unknown".into());

        let mut layers = match self.log_format {
            LogFormat::COMPACT => vec![fmt::layer().compact().boxed()],
            LogFormat::JSON => vec![fmt::layer().json().boxed()],
        };

        if let Some(endpoint) = self.otlp_endpoint {
            let otlp_exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint);

            let trace_config =
                trace::config().with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name),
                ]));

            let tracer = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(otlp_exporter)
                .with_trace_config(trace_config)
                .install_simple()
                .expect("Failed to install OTLP tracer");

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
