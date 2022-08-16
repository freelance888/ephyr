//! Types relevant to FFmpeg module
/// Status of FFmpeg process
///
/// Using for communication through [`tokio::sync::watch`]
/// between [`Restreamer`] and [`MixingRestreamer`] with [`RestreamerKind`].
///
/// [FFmpeg]: https://ffmpeg.org
/// [`Restreamer`]: crate::ffmpeg::Restreamer
/// [`MixingRestreamer`]: crate::ffmpeg::MixingRestreamer
/// [`RestreamerKind`]: crate::ffmpeg::RestreamerKind
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum FFmpegStatus {
    /// Process is started and running
    Running = 0,
    /// Process was or already aborted
    Aborted = 1,
}
