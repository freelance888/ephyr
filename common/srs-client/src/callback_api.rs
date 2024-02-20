//! [HTTP Callback] definitions of [SRS].
//!
//! [SRS]: https://ossrs.io/
//! [1]: https://ossrs.io/lts/en-us/docs/v5/doc/http-callback
//!
//! To work with callbacks you have to define callback which is receiving
//! [`srs_client::SrsCallbackReq`] struct.
//!
//! # Example for actix-web:
//! ```rust
//! use actix_web::{post, web};
//! use srs_client::{SrsCallbackEvent, SrsCallbackReq};
//!
//! #[post("srs_callback")]
//! pub async fn on_callback(
//!     req: web::Json<SrsCallbackReq>,
//! ) -> Result<&'static str, String> {
//!     match req.action {
//!         SrsCallbackEvent::OnConnect => {
//!             dbg!(&req)
//!         }
//!         _ => Ok(()),
//!     }
//!     .map(|()| "0")
//! }
//! ```
#![allow(unused_imports)]
mod event;
mod request;

pub use self::{event::SrsCallbackEvent, request::SrsCallbackReq};
