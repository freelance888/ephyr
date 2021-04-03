//! Logging tools and their initialization.

#![deny(
    broken_intra_doc_links,
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

pub use slog::{self, Drain};
pub use slog_scope::{self as log, logger};

/// Initializes global logger with the given verbosity `level` ([`Info`] by
/// default, if [`None`]), returning its guard that should be held as long as
/// program runs.
///
/// # Panics
///
/// If failed to initialize logger.
///
/// [`Info`]: slog::Level::Info
pub fn init(level: Option<slog::Level>) -> slog_scope::GlobalLoggerGuard {
    let guard = slog_scope::set_global_logger(main_logger(
        level.unwrap_or(slog::Level::Info),
    ));
    if let Err(e) = slog_stdlog::init() {
        panic!("Failed to initialize logger: {}", e)
    };
    guard
}

/// Creates, configures and returns main [`Logger`] of the application.
///
/// [`Logger`]: slog::Logger
#[must_use]
pub fn main_logger(level: slog::Level) -> slog::Logger {
    use slog::Drain as _;
    use slog_async::OverflowStrategy::Drop;

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();

    let drain = drain
        .filter_level(level)
        .filter(|rec| {
            // Disable annoying DEBUG logs from `hyper` crate.
            !(rec.level() == slog::Level::Debug
                && rec.module() == "hyper::proto::h1::io")
        })
        .fuse();

    let drain = slog_async::Async::new(drain)
        .overflow_strategy(Drop)
        .build()
        .fuse();

    slog::Logger::root(drain, slog::o!())
}
