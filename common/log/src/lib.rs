//! Logging tools and their initialization.

#![deny(
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
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
pub use tracing::{self, Level};
pub use tracing_actix_web;
pub use tracing_log::log;
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

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
        panic!("Failed to initialize logger: {}", e);
    };
    let level = level.unwrap_or(Level::INFO);
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting tracing subscriber failed");
}
