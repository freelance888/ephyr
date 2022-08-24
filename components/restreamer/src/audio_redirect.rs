//! Process audio redirection in Ephyr

use crate::state::MixinId;
use std::path::PathBuf;

pub mod audio_processing_pool;
pub mod teamspeak;
pub mod teamspeak_to_fifo;

/// [FIFO] path where stream captures from the [TeamSpeak] server.
///
/// Should be fed into [FFmpeg]'s as file input.
///
/// [FFmpeg]: https://ffmpeg.org
/// [TeamSpeak]: https://teamspeak.com
/// [FIFO]: https://www.unix.com/man-page/linux/7/fifo/
#[inline]
#[must_use]
pub fn get_fifo_path(mixin_id: MixinId) -> PathBuf {
    std::env::temp_dir().join(format!("ephyr_mixin_{}.pipe", mixin_id))
}
