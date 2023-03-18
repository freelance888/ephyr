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
use tracing::level_filters::LevelFilter;
pub use tracing::{self, Level, Span};
pub use tracing_actix_web;
use tracing_forest::ForestLayer;
pub use tracing_futures::Instrument;
pub use tracing_log::log;
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, Registry};

/// Initializes global logger with the given verbosity `level` ([`Info`] by
/// default, if [`None`]), returning its guard that should be held as long as
/// program runs.
///
/// # Panics
///
/// If failed to initialize logger.
///
/// [`Info`]: tracing::Level::INFO
pub fn init(level: Option<Level>) {
    if let Err(e) = LogTracer::init() {
        panic!("Failed to initialize logger: {e}");
    };
    let level = level.unwrap_or(Level::INFO);

    let subscriber = Registry::default()
        .with(LevelFilter::from(level))
        .with(ForestLayer::default());
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting tracing subscriber failed");
}
