//! Redirect audio from [TeamSpeak] to [FIFO]
//!
//! [TeamSpeak]: https://teamspeak.com
//! [FIFO]: https://www.unix.com/man-page/linux/7/fifo/
use crate::{
    audio_redirect::{get_fifo_path, teamspeak},
    state,
    state::MixinId,
};
use ephyr_log::log;
use futures::future;
use interprocess::os::unix::fifo_file::create_fifo;
use std::{borrow::Cow, collections::HashMap, sync::Arc};
use tokio::{fs::File, io, sync::Mutex};
use tsclientlib::Identity;

/// Handle to a running data transfer process.
#[derive(Debug)]
pub struct TeamspeakToFIFO {
    abort_handle: future::AbortHandle,
    pub(crate) mixin_id: MixinId,
    pub(crate) input: TeamspeakInput,
}

impl Drop for TeamspeakToFIFO {
    #[inline]
    fn drop(&mut self) {
        self.abort_handle.abort();
        // Clean up FIFO file
        let _ = std::fs::remove_file(get_fifo_path(self.mixin_id))
            .map_err(|e| log::error!("Failed to remove FIFO: {}", e));
    }
}
impl TeamspeakToFIFO {
    pub(crate) fn run(input: TeamspeakInput) -> Self {
        let mixin_id = input.mixin_id;
        let cloned_ts_input = Arc::clone(&input.input);
        let (spawner, abort_handle) = future::abortable(
            TeamspeakToFIFO::copy_data(cloned_ts_input, mixin_id),
        );
        drop(tokio::spawn(spawner));
        Self {
            abort_handle,
            mixin_id,
            input,
        }
    }

    /// Copy data from [`TeamspeakToFIFO::input`] to [FIFO].
    ///
    /// Each data copying is operated in separate thread.
    /// [FIFO] should be fed before [FFmpeg].
    ///
    /// # Errors
    ///
    /// If [FIFI] file failed to create.
    /// We need it because [FFmpeg] cannot start if no [FIFO] file.
    ///
    /// [FIFO]: https://www.unix.com/man-page/linux/7/fifo/
    /// [FFmpeg]: https://ffmpeg.org
    async fn copy_data(
        input: Arc<Mutex<teamspeak::Input>>,
        mixin_id: MixinId,
    ) -> io::Result<()> {
        let fifo_path = get_fifo_path(mixin_id);

        // FIFO should be created before open
        if !fifo_path.exists() {
            let _ = create_fifo(&fifo_path, 0o777)
                .map_err(|e| log::error!("Failed to create FIFO: {}", e));
        }

        // Initialize copying future to fed it into select
        let mut src = input.lock().await;
        let mut file = File::create(&fifo_path).await?;
        let _ = io::copy(&mut *src, &mut file)
            .await
            .map_err(|e| log::error!("Failed to write into FIFO: {}", e));

        Ok(())
    }
}

/// Additional live stream for mixing in a [`TeamspeakToFIFO`].
#[derive(Clone, Debug)]
pub struct TeamspeakInput {
    /// ID of a [`state::Mixin`] represented by this [`TeamspeakInput`].
    pub mixin_id: MixinId,

    /// Actual live audio stream captured from the [TeamSpeak] server.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    input: Arc<Mutex<teamspeak::Input>>,
}

impl TeamspeakInput {
    /// Creates a new [`TeamspeakInput`]
    pub fn new(
        state: &state::Mixin,
        label: Option<&state::Label>,
        prev: Option<&TeamspeakInput>,
    ) -> Option<Self> {
        let mixin_id = state.id;
        let input = if let Some(p) = prev {
            Arc::clone(&p.input)
        } else {
            let mut host = Cow::Borrowed(state.src.host_str()?);
            if let Some(port) = state.src.port() {
                host = Cow::Owned(format!("{}:{}", host, port));
            }
            let channel = state.src.path().trim_start_matches('/');

            let query: HashMap<String, String> =
                state.src.query_pairs().into_owned().collect();

            let name = query
                .get("name")
                .cloned()
                .or_else(|| label.map(|l| format!("🤖 {}", l)))
                .unwrap_or_else(|| format!("🤖 {}", state.id));

            let identity =
                query.get("identity").map_or_else(Identity::create, |v| {
                    Identity::new_from_str(v).unwrap_or_else(|e| {
                        log::error!(
                            "Failed to create identity `{}`\
                                    \n\t with error: {}",
                            &v,
                            &e
                        );
                        Identity::create()
                    })
                });
            Arc::new(Mutex::new(teamspeak::Input::new(
                teamspeak::Connection::build(host.into_owned())
                    .channel(channel.to_owned())
                    .name(name)
                    .identity(identity),
            )))
        };
        Some(Self { mixin_id, input })
    }
}
