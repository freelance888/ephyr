//! [DVR]-related definitions and implementations.
//!
//! [DVR]: https://en.wikipedia.org/wiki/Digital_video_recorder

use std::{
    ffi::OsString,
    io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::anyhow;
use ephyr_log::log;
use futures::{future, TryFutureExt as _, TryStreamExt as _};
use once_cell::sync::OnceCell;
use tokio::fs;
use url::Url;
use uuid::Uuid;

use crate::state;

/// Global instance of a [DVR] files [`Storage`] used by this application.
///
/// [DVR]: https://en.wikipedia.org/wiki/Digital_video_recorder
static STORAGE: OnceCell<Storage> = OnceCell::new();

/// Storage of [DVR] files.
///
/// [DVR]: https://en.wikipedia.org/wiki/Digital_video_recorder
#[derive(Debug)]
pub struct Storage {
    /// Absolute path where the [DVR] files are stored.
    ///
    /// [DVR]: https://en.wikipedia.org/wiki/Digital_video_recorder
    pub root_path: PathBuf,
}

impl Storage {
    /// Returns the global instance of [`Storage`].
    ///
    /// # Panics
    ///
    /// If the global instance hasn't been initialized yet via
    /// [`Storage::set_global()`].
    #[inline]
    #[must_use]
    pub fn global() -> &'static Storage {
        // TODO: Inject `Storage` normally as dependency rather than use global
        //       instance.
        STORAGE.get().expect("dvr::Storage is not initialized")
    }

    /// Sets the global instance of [`Storage`].
    ///
    /// # Errors
    ///
    /// If the global instance has been set already.
    #[inline]
    pub fn set_global(self) -> anyhow::Result<()> {
        STORAGE
            .set(self)
            .map_err(|_| anyhow!("dvr::Storage has been initialized already"))
    }

    /// Forms a correct [`Url`] pointing to the file for recording a live stream
    /// by the given [`state::Output`].
    ///
    /// # Errors
    /// If failed to convert path to [`Url`]
    pub fn file_url(&self, output: &state::Output) -> anyhow::Result<Url> {
        let mut full = self.root_path.clone();
        full.push(output.id.to_string());
        full.push(output.dst.path().trim_start_matches('/'));
        Url::from_file_path(full)
            .map_err(|e| anyhow!("Failed convert path to URL: {:?}", e))
    }

    /// Lists stored [DVR] files of the given [`state::Output`].
    ///
    /// Returns them as relative paths to this [`Storage::root_path`].
    ///
    /// [DVR]: https://en.wikipedia.org/wiki/Digital_video_recorder
    pub async fn list_files(&self, id: state::OutputId) -> Vec<String> {
        let dir = &self.root_path;

        let mut output_dir = dir.clone();
        output_dir.push(id.to_string());

        fs::read_dir(output_dir)
            .try_flatten_stream()
            .try_filter_map(|i| async move {
                Ok(i.file_type().await?.is_file().then(|| i.path()).and_then(
                    |p| Some(p.strip_prefix(dir).ok()?.display().to_string()),
                ))
            })
            .try_collect()
            .await
            .unwrap_or_else(|e| {
                if e.kind() != io::ErrorKind::NotFound {
                    log::error!("Failed to list {} DVR files: {}", id, e);
                }
                vec![]
            })
    }

    /// Removes a [DVR] file from this [`Storage`] identified by its relative
    /// `path` to this [`Storage::root_path`].
    ///
    /// Returns `true` if the file has been removed, otherwise `false`.
    ///
    /// [DVR]: https://en.wikipedia.org/wiki/Digital_video_recorder
    pub async fn remove_file<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();

        let mut full = self.root_path.clone();
        full.push(path.strip_prefix("/").unwrap_or(path));

        if let Err(e) = fs::remove_file(full).await {
            if e.kind() != io::ErrorKind::NotFound {
                log::error!(
                    "Failed to remove {} DVR file: {}",
                    path.display(),
                    e,
                );
            }
            return false;
        }
        true
    }

    /// Cleans up any [DVR] files of this [`Storage`] not being associated with
    /// [`state::Output`]s of the given renewed [`state::Restream`]s.
    ///
    /// [DVR]: https://en.wikipedia.org/wiki/Digital_video_recorder
    pub async fn cleanup(&self, restreams: &[state::Restream]) {
        // TODO: Consider only `file:///` outputs?
        fs::read_dir(&self.root_path)
            .try_flatten_stream()
            .try_filter(|i| {
                future::ready(
                    i.file_name()
                        .to_str()
                        .and_then(|n| Uuid::parse_str(n).ok())
                        .map_or(true, |id| {
                            let id = id.into();
                            !restreams
                                .iter()
                                .any(|r| r.outputs.iter().any(|o| o.id == id))
                        }),
                )
            })
            .try_for_each_concurrent(4, |i| async move {
                if i.file_type().await?.is_dir() {
                    fs::remove_dir_all(i.path()).await
                } else {
                    fs::remove_file(i.path()).await
                }
            })
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to cleanup DVR files: {}", e);
            });
    }
}

/// Creates a new recording file path from the given DVR file [`Url`] (formed by
/// [`Storage::file_url()`]) appended with the current timestamp in microseconds
/// to ensure its uniqueness.
///
/// Also, ensures that the appropriate parent directory for the file exists.
///
/// # Errors
///
/// If cannot create a file path from the given [`Url`], or fails to create its
/// parent directory.
#[allow(clippy::missing_panics_doc)]
pub async fn new_file_path(url: &Url) -> io::Result<PathBuf> {
    let mut path = url.to_file_path().map_err(|_| {
        io::Error::new(io::ErrorKind::Other, "File URL contains bad file path")
    })?;

    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).await?;
    }

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let mut file_name = OsString::new();
    if let Some(name) = path.file_stem() {
        file_name.push(name);
    }
    file_name.push(format!("_{}.", now.as_micros()));
    if let Some(ext) = path.extension() {
        file_name.push(ext);
    }
    path.set_file_name(file_name);

    Ok(path)
}
