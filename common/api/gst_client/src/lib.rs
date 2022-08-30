//! Definitions of [GStreamer Daemon][1] API and a client to request it.
//!
//! [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
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

pub mod client;
mod error;
pub mod gstd_types;
pub mod resources;

pub use crate::{client::GstClient, error::Error};
