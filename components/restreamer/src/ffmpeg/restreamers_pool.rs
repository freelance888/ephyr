//! Pool of [FFmpeg] processes performing re-streaming of a media traffic.
//!
//! [FFmpeg]: https://ffmpeg.org

use std::{collections::HashMap, path::PathBuf};

use ephyr_log::tracing;
use url::Url;
use uuid::Uuid;

use crate::{
    ffmpeg::{restreamer::Restreamer, restreamer_kind::RestreamerKind},
    state::{self, State},
};
use ephyr_log::tracing::instrument;
use std::result::Result::Err;

/// Pool of [FFmpeg] processes performing re-streaming of a media traffic.
///
/// [FFmpeg]: https://ffmpeg.org
#[derive(Debug)]
pub struct RestreamersPool {
    /// Path to a [FFmpeg] binary used for spawning processes.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    ffmpeg_path: PathBuf,

    /// Path to where local video files are downloaded to and played by ffmpeg
    files_root: PathBuf,

    /// Pool of currently running [FFmpeg] re-streaming processes identified by
    /// an ID of the correspondent element in a [`State`].
    ///
    /// So, potentially allows duplication.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    pool: HashMap<Uuid, Restreamer>,

    /// Application [`State`] dictating which [FFmpeg] processes should run.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    /// [`State`]: crate::state::State
    state: State,
}

impl RestreamersPool {
    /// Creates a new [`RestreamersPool`] out of the given parameters.
    #[inline]
    #[must_use]
    pub fn new<P: Into<PathBuf>>(
        ffmpeg_path: P,
        state: State,
        file_root: PathBuf,
    ) -> Self {
        Self {
            ffmpeg_path: ffmpeg_path.into(),
            pool: HashMap::new(),
            files_root: file_root,
            state,
        }
    }

    /// Adjusts this [`RestreamersPool`] to run [FFmpeg] re-streaming processes
    /// according to the given renewed [`state::Restream`]s.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[instrument(skip_all, name = "RestreamersPool::apply")]
    pub(crate) fn apply(&mut self, restreams: &[state::Restream]) {
        // The most often case is when one new FFmpeg process is added.
        let mut new_pool = HashMap::with_capacity(self.pool.len() + 1);

        for r in restreams {
            self.apply_playlist(r, &mut new_pool);
            self.apply_input(
                &r.key,
                &r.input,
                r.playlist.currently_playing_file.is_some(),
                r.with_playback_encoding,
                &mut new_pool,
            );

            if !r.input.enabled
                || (!r.input.is_ready_to_serve()
                    && r.playlist.currently_playing_file.is_none())
            {
                continue;
            }

            let input_url = match r.main_input_rtmp_endpoint_url() {
                Ok(input_url) => input_url,
                Err(e) => {
                    tracing::error!(
                        "Failed to get main input RTMP endpoint: {}",
                        e
                    );
                    continue;
                }
            };
            for o in &r.outputs {
                _ = self.apply_output(&input_url, o, &mut new_pool);
            }
        }

        self.pool = new_pool;
    }

    #[instrument(skip_all, fields(actor=%restream.id))]
    fn apply_playlist(
        &mut self,
        restream: &state::Restream,
        new_pool: &mut HashMap<Uuid, Restreamer>,
    ) {
        if restream.playlist.currently_playing_file.is_some() {
            let id = restream.playlist.id.into();
            // TODO: should it be made in another way?
            if let Some(new_kind) = RestreamerKind::from_playlist(
                &restream.playlist,
                &restream.key,
                &restream.input.key,
                &self.files_root,
            ) {
                self.apply_new_kind(id, new_kind, new_pool);
            };
        }
    }

    /// Traverses the given [`state::Input`] filling the `new_pool` with
    /// required [FFmpeg] re-streaming processes. Tries to preserve already
    /// running [FFmpeg] processes in its `pool` as much as possible.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[instrument(skip_all,
        fields(
            restream.key=%key,
            input.key=%input.key,
            is_playing_playlist,
            with_playback_encoding)
        )
    ]
    fn apply_input(
        &mut self,
        key: &state::RestreamKey,
        input: &state::Input,
        is_playing_playlist: bool,
        with_playback_encoding: bool,
        new_pool: &mut HashMap<Uuid, Restreamer>,
    ) {
        if let Some(state::InputSrc::Failover(s)) = &input.src {
            for i in &s.inputs {
                tracing::debug!(actor=%i.id,"Failover input");
                self.apply_input(
                    key,
                    i,
                    false,
                    with_playback_encoding,
                    new_pool,
                );
            }
        }

        for endpoint in &input.endpoints {
            let id = endpoint.id.into();
            tracing::debug!(actor=%id, "Input endpoint aka Restreamer");
            let kind = RestreamerKind::from_input(
                input,
                endpoint,
                key,
                is_playing_playlist,
                with_playback_encoding,
                &self.state.files.lock_ref(),
                &self.files_root,
            );

            if let Some(new_kind) = kind {
                self.apply_new_kind(id, new_kind, new_pool);
            }
        }
    }

    /// Inspects the given [`state::Output`] filling the `new_pool` with a
    /// required [FFmpeg] re-streaming process. Tries to preserve already
    /// running [FFmpeg] processes in its `pool` as much as possible.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[instrument(skip_all, fields(
        src=%from_url.path(), dst=output.dst.path())
    )]
    fn apply_output(
        &mut self,
        from_url: &Url,
        output: &state::Output,
        new_pool: &mut HashMap<Uuid, Restreamer>,
    ) -> Option<()> {
        if !output.enabled {
            return None;
        }

        let id = output.id.into();

        let new_kind = RestreamerKind::from_output(
            output,
            from_url,
            self.pool.get(&id).map(|p| &p.kind),
        )?;

        self.apply_new_kind(id, new_kind, new_pool);
        Some(())
    }

    /// Tries to remove process with provided `id` from current process pool
    /// and checks if it needs to be restarted bases on `new_kind`. If not
    /// the process is inserted to `new_pool`, otherwise a new process is
    /// created with new settings.
    #[instrument(skip_all, fields(id=%id,
        src=%new_kind.src_url().path(), dst=%new_kind.to_url())
    )]
    fn apply_new_kind(
        &mut self,
        id: Uuid,
        new_kind: RestreamerKind,
        new_pool: &mut HashMap<Uuid, Restreamer>,
    ) {
        tracing::info!(%id, %new_kind, "New kind applied");

        let process = self
            .pool
            .remove(&id)
            .and_then(|mut p| (!p.kind.needs_restart(&new_kind)).then_some(p))
            .unwrap_or_else(|| {
                Restreamer::run(
                    self.ffmpeg_path.clone(),
                    new_kind,
                    self.state.clone(),
                )
            });

        let old_process = new_pool.insert(id, process);
        drop(old_process);
    }
}
