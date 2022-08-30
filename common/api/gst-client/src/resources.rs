//! API endpoints used inside [`crate::GstClient`] for
//! communication with [GStD] based on
//!
//! [GStD]: https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon
mod bus;
mod debug;
mod element;
mod pipeline;

pub use self::{bus::PipelineBus, debug::Debug, element::PipelineElement, pipeline::Pipeline};
