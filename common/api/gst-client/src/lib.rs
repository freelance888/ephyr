//! Rust client for [GStreamer Daemon][1] API.
//!
//! On official [GStD API documentation][2] page covered all use cases
//! of the client.
//!
//! This client is defining API a little bit differently then official
//! but it quite intuitive.
//!
//! The entry point is [`GstClient`] which encapsulate all communication logic.
//!
//! # Examples
//!
//! Create new client for `http://127.0.0.1:5000` host
//! ```
//! use gst_client::GstClient;
//!
//! let client = GstClient::default();
//! ```
//!
//! Create client for specific address
//! ```
//! use gst_client::GstClient;
//!
//! let client = GstClient::build("http://127.0.0.1:5000")?;
//! ```
//!
//! Perform operations with Pipeline
//!
//! ```
//! use gst_client::GstClient;
//!
//! let client = GstClient::default();
//! let new_pipeline = client.pipeline("new-pipeline").create("playbin")?;
//! new_pipeline.
//! ```
//! ```
//! use gst_client::GstClient;
//!
//! let client = GstClient::default();
//! let new_pipeline = client.pipeline("new-pipeline").create("playbin")?;
//! ```
//!
//! [1]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
//! [2]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon_-_C_API
//! [`GstClient`]: client::GstClient
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
