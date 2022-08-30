//! API endpoints used inside [`crate::GstClient`]
mod bus;
mod debug;
mod element;
mod pipeline;

pub use self::{
    bus::PipelineBus, debug::Debug, element::PipelineElement,
    pipeline::Pipeline,
};
