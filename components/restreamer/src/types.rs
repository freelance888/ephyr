//! Common types
//!
use futures::future;

/// Abort handle of a future.
///
#[derive(Clone, Debug)]
pub struct DroppableAbortHandle(future::AbortHandle);

impl DroppableAbortHandle {
    /// Creates and initialise callback for aborting future on `drop()`
    pub fn new(callback: future::AbortHandle) -> Self {
        Self(callback)
    }
}

impl Drop for DroppableAbortHandle {
    #[inline]
    fn drop(&mut self) {
        self.0.abort();
    }
}