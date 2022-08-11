//! [FFmpeg]-based definitions and implementations.
//!
//! [FFmpeg]: https://ffmpeg.org

mod copy_restreamer;
mod mixing_restreamer;
mod restreamer;
mod restreamer_kind;
mod restreamers_pool;
mod transcoding_restreamer;

pub use self::{
    copy_restreamer::CopyRestreamer, mixing_restreamer::MixingRestreamer,
    restreamer::Restreamer, restreamer_kind::RestreamerKind,
    restreamers_pool::RestreamersPool,
    transcoding_restreamer::TranscodingRestreamer,
};
