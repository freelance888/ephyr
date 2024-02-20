//! SRS Client helps to
//!
//! Simplify to working process with [SRS] under Rust.
//!
//! [SRS]: https://ossrs.io
mod callback_api;
mod http_api;

pub use crate::{
    callback_api::{SrsCallbackEvent, SrsCallbackReq},
    http_api::{SrsClient, SrsClientError, SrsClientResp},
};
