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
pub mod console_logger;
pub mod dvr;
pub mod ffmpeg;
pub mod file_manager;
mod proc;
pub mod server;
pub mod spec;
pub mod srs;
pub mod state;
pub mod stream_probe;
pub mod stream_statistics;
pub mod teamspeak;
pub mod types;

use itertools::Itertools;
use std::any::Any;

use ephyr_log::tracing;

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
            Some(tracing::Level::DEBUG)
        } else {
            None
        }
    });
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

/// This way of reordering prevent us to loose data in case if count of ids is less that count
/// of items
pub fn reorder_items<T, F, Id>(items: &[T], ids: &[Id], get_id: F) -> Vec<T>
where
    T: Clone,
    Id: Eq,
    F: Fn(&T) -> Id,
{
    items
        .iter()
        .map(|item| {
            let pos = ids.iter().position(|id| *id == get_id(item));
            match pos {
                Some(p) => (p, item),
                None => (usize::MAX, item),
            }
        })
        .sorted_by(|a, b| a.0.cmp(&b.0))
        .map(|x| x.1.clone())
        .collect()
}
