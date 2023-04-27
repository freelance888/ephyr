//! Manager of the server [`State`].

use std::{path::Path, sync::Arc};

use anyhow::anyhow;
use chrono::Utc;
use tokio::{fs, io::AsyncReadExt as _, sync::RwLock};

use super::{Playlist, PlaylistSlug, State};

/// Manager of the server [`State`].
///
/// It provides access to a synchronized [`State`] and takes care about
/// persisting it to filesystem to survive application restarts.
#[derive(Clone, Debug)]
pub struct Manager {
    /// Path to the file where the [`Manager::state`] should be persisted.
    file: Arc<Path>,

    /// Server's [`State`] to keep synchronized and persisted, along with its
    /// current version.
    ///
    /// Version is used for CAS (compare and swap) operations.
    state: Arc<RwLock<(State, u8)>>,
}

impl Manager {
    /// Instantiates new [`Manager`] to read from and persist the [`State`] in
    /// the provided `file`.
    ///
    /// If no `file` exists, the new empty one will be created.
    ///
    /// # Errors
    ///
    /// If the `file`:
    /// - cannot be read;
    /// - contains broken [`State`].
    pub async fn try_new<P: AsRef<Path>>(
        file: P,
    ) -> Result<Self, anyhow::Error> {
        let file = file.as_ref();

        let mut contents = vec![];
        _ = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .open(&file)
            .await
            .map_err(|e| {
                anyhow!("Failed to open '{}' file: {}", file.display(), e)
            })?
            .read_to_end(&mut contents)
            .await
            .map_err(|e| {
                anyhow!("Failed to read '{}' file: {}", file.display(), e)
            })?;

        let state = if contents.is_empty() {
            State::default()
        } else {
            serde_json::from_slice(&contents).map_err(|e| {
                anyhow!(
                    "Failed to deserialize vod::meta::State read from \
                     '{}' file: {}",
                    file.display(),
                    e,
                )
            })?
        };

        Ok(Self {
            file: file.into(),
            state: Arc::new(RwLock::new((state, 0))),
        })
    }

    /// Returns the copy of the current actual [`State`].
    #[inline]
    pub async fn state(&self) -> State {
        self.state.read().await.0.clone()
    }

    /// Returns the copy of the current actual [`State`] along with it's current
    /// version.
    #[inline]
    pub async fn state_and_version(&self) -> (State, u8) {
        let state = self.state.read().await;
        (state.0.clone(), state.1)
    }

    /// Returns from the current actual [`State`] the copy of the [`Playlist`]
    /// identified by its `slug`.
    #[inline]
    pub async fn playlist(&self, slug: &PlaylistSlug) -> Option<Playlist> {
        (self.state.read().await.0).0.get(slug).cloned()
    }

    /// Replaces the current [`State`] with a `new` one.
    ///
    /// If `ver` is specified, then makes sure that it matches the version of
    /// the current [`State`], and if it doesn't, then no-op. This provides a
    /// basic [optimistic concurrency][1] allowing to modify the current
    /// [`State`] without holding the inner lock the whole modifying time.
    ///
    /// For already existing [`Playlist`]s all [`Playlist::initial`] positions
    /// are preserved.
    ///
    /// If `dry_run` is specified, then `new` state won't be persisted and
    /// applied, but verified only.
    ///
    /// # Errors
    ///
    /// - If the `new` [`State`] fails to be persisted.
    /// - If some [`Playlist`]'s playback of the existing [`State`] will be
    ///   broken (can be suppressed with `force` argument set as `true`).
    ///
    /// [1]: https://en.wikipedia.org/wiki/Optimistic_concurrency_control
    pub async fn set_state(
        &self,
        mut new: State,
        ver: Option<u8>,
        force: bool,
        dry_run: bool,
    ) -> Result<(), anyhow::Error> {
        let mut state = self.state.write().await;

        if let Some(new_ver) = ver {
            if new_ver != state.1 {
                return Ok(());
            }
        }

        for playlist in new.values_mut() {
            Self::preserve_playlist_playback(playlist, &state.0, force)?;
        }

        if dry_run {
            return Ok(());
        }

        self.persist_state(&new).await?;

        state.0 = new;
        state.1 = state.1.checked_add(1).unwrap_or_default();

        Ok(())
    }

    /// Sets the given [`Playlist`] to the current [`State`].
    ///
    /// If there is no such [`Playlist`] in the current [`State`], then it will
    /// be added "as is". Otherwise, the existing [`Playlist`] will be updated
    /// preserving its [`Playlist::initial`] position.
    ///
    /// If `dry_run` is specified, then the given [`Playlist`] won't be
    /// persisted and applied, but verified only.
    ///
    /// # Errors
    ///
    /// - If updated [`State`] fails to be persisted.
    /// - If the current [`Playlist`]'s playback will be broken (can be
    ///   suppressed with `force` argument set as `true`).
    pub async fn set_playlist(
        &self,
        mut playlist: Playlist,
        force: bool,
        dry_run: bool,
    ) -> Result<(), anyhow::Error> {
        let mut state = self.state.write().await;

        Self::preserve_playlist_playback(&mut playlist, &state.0, force)?;

        if dry_run {
            return Ok(());
        }

        let mut new = state.0.clone();
        let _inserted = new.insert(playlist.slug.clone(), playlist);

        self.persist_state(&new).await?;

        state.0 = new;
        state.1 = state.1.checked_add(1).unwrap_or_default();

        Ok(())
    }

    /// Removes the given [`Playlist`] from the current [`State`].
    ///
    /// If there is no such [`Playlist`] in the current [`State`], then no-op.
    ///
    /// # Errors
    ///
    /// If updated [`State`] fails to be persisted.
    pub async fn delete_playlist(
        &self,
        slug: &PlaylistSlug,
    ) -> Result<(), anyhow::Error> {
        if !self.state.read().await.0.contains_key(slug) {
            return Ok(());
        }

        let mut state = self.state.write().await;

        let mut new = state.0.clone();
        let _removed = new.remove(slug);
        self.persist_state(&new).await?;

        state.0 = new;
        state.1 = state.1.checked_add(1).unwrap_or_default();

        Ok(())
    }

    /// Refreshes all [`Playlist::initial`] positions in the [`State`] managed
    /// by this [`Manager`].
    ///
    /// # Errors
    ///
    /// If updated [`State`] fails to be persisted.
    pub async fn refresh_playlists_positions(
        &self,
    ) -> Result<(), anyhow::Error> {
        let mut state = self.state.write().await;

        let now = Utc::now();
        for playlist in state.0.values_mut() {
            let _result = playlist.schedule_nginx_vod_module_set(Some(now), 1);
        }

        self.persist_state(&state.0).await
    }

    /// Persists the given [`State`] to this [`Manager::file`].
    ///
    /// # Errors
    ///
    /// If the [`State`] fails to be persisted.
    async fn persist_state(&self, state: &State) -> Result<(), anyhow::Error> {
        fs::write(
            &*self.file,
            serde_json::to_vec(state)
                .expect("Failed to serialize vod::meta::State"),
        )
        .await
        .map_err(|e| {
            anyhow!(
                "Failed to persist vod::meta::State into '{}' file: {}",
                self.file.display(),
                e,
            )
        })
    }

    /// Preserves the [`Playlist::initial`] position for the given `playlist`
    /// from the given `old` [`State`] in case it contains such [`Playlist`].
    ///
    /// # Errors
    ///
    /// If the current playback of the given `playlist` doesn't much the one in
    /// the `old` [`State`].
    ///
    /// If `force` is `true` then resets the [`Playlist::initial`] position
    /// assuming that breaking playback is okay and never errors.
    fn preserve_playlist_playback(
        playlist: &mut Playlist,
        old: &State,
        force: bool,
    ) -> Result<(), anyhow::Error> {
        if let Some(old) = old.get(&playlist.slug) {
            let new_initial = playlist.initial;

            playlist.initial = old.initial;

            let now = Utc::now();
            if playlist.schedule_nginx_vod_module_set(Some(now), 3)
                != old.clone().schedule_nginx_vod_module_set(Some(now), 3)
            {
                if force {
                    playlist.initial = new_initial;
                } else {
                    return Err(anyhow!(
                        "Updating playlist '{}' breaks its playback",
                        playlist.title,
                    ));
                }
            }
        }
        Ok(())
    }
}
