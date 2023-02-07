//! Ephyr [RTMP] re-streaming server.
//!
//! [RTMP]: https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol

#![allow(  // TODO: Find a way to ignore in generated code instead globally
    clippy::unreadable_literal,
    clippy::wildcard_imports)
]
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

pub mod api;
pub mod broadcaster;
pub mod cli;
pub mod client_stat;
pub mod dvr;
pub mod ffmpeg;
pub mod serde;
pub mod server;
pub mod spec;
pub mod srs;
pub mod state;
pub mod stream_probe;
pub mod teamspeak;
pub mod types;

use std::{any::Any, mem};

use ephyr_log::slog;

pub use self::{spec::Spec, state::State};

/// Runs application.
///
/// # Errors
///
/// If running has failed and could not be performed. The appropriate error
/// is logged.
pub fn run() -> Result<(), cli::Failure> {
    let mut cfg = cli::Opts::from_args();
    cfg.verbose = cfg.verbose.or({
        if cfg.debug {
            Some(slog::Level::Debug)
        } else {
            None
        }
    });

    // This guard should be held till the end of the program for the logger
    // to present in global context.
    mem::forget(ephyr_log::init(cfg.verbose));

    server::run(cfg)
}

/// Interprets given [panic payload][1] as displayable message.
///
/// [1]: std::panic::PanicInfo::payload
pub fn display_panic<'a>(err: &'a (dyn Any + Send + 'static)) -> &'a str {
    if let Some(s) = err.downcast_ref::<&str>() {
        return s;
    }
    if let Some(s) = err.downcast_ref::<String>() {
        return s.as_str();
    }
    "Box<Any>"
}
