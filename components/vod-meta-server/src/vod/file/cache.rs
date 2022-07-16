//! Definitions related to [VOD] files cache.
//!
//! [VOD]: https://en.wikipedia.org/wiki/Video_on_demand

use std::{
    panic::AssertUnwindSafe,
    path::{self, Path, PathBuf},
};

use anyhow::anyhow;
use ephyr_log::log;
use futures::{sink, FutureExt as _, StreamExt as _, TryStreamExt as _};
use tempfile::TempDir;
use tokio::{fs, io, sync::mpsc};
use tokio_util::compat::FuturesAsyncReadCompatExt as _;
use url::Url;

use crate::util::display_panic;

/// Manager of [VOD] files cache.
///
/// It downloads the requested URLs in background and returns their path once
/// they appear in cache.
///
/// [VOD]: https://en.wikipedia.org/wiki/Video_on_demand
#[derive(Debug)]
pub struct Manager {
    /// Absolute path to the directory where cache files are downloaded to and
    /// persisted in.
    cache_dir: PathBuf,

    /// Queue of tasks to perform downloading.
    downloads: mpsc::UnboundedSender<Url>,

    /// Directory where temporary downloading files are created.
    ///
    /// It cleans up automatically on [`Drop`].
    ///
    /// The path where this directory will be created can be manipulated via
    /// [`TMPDIR` env var][1].
    ///
    /// [1]: https://en.wikipedia.org/wiki/TMPDIR
    _tmp_dir: TempDir,
}

impl Manager {
    /// Number of maximum allowed concurrent downloads at the same time.
    pub const CONCURRENT_DOWNLOADS: usize = 4;

    /// Creates new [`Manager`] running the background downloads queue
    /// processing.
    ///
    /// # Errors
    ///
    /// - If specified `dir` doesn't exist or cannot be resolved.
    /// - If temporary directory cannot be created.
    pub fn try_new<P: AsRef<Path>>(dir: P) -> io::Result<Self> {
        let cache_dir = dir.as_ref().canonicalize()?;

        let tmp_dir = tempfile::Builder::new()
            .prefix("ephyr-vod-cache.")
            .tempdir()?;

        let (tx, rx) = mpsc::unbounded_channel::<Url>();
        drop(tokio::spawn(Self::run_downloads(
            rx,
            cache_dir.clone(),
            tmp_dir.path().to_owned(),
        )));

        Ok(Self {
            cache_dir,
            downloads: tx,
            _tmp_dir: tmp_dir,
        })
    }

    /// Returns the path of a cached file for the given [`Url`], if there is in
    /// cache any.
    ///
    /// If there is no cached file for the given [`Url`], then schedules it for
    /// downloading.
    ///
    /// # Errors
    ///
    /// - If the given [`Url`] is not supported for downloading.
    /// - If the given [`Url`] cannot be scheduled for downloading.
    pub async fn get_cached_path(
        &self,
        url: &Url,
    ) -> Result<Option<PathBuf>, anyhow::Error> {
        let full =
            self.cache_dir
                .join(Self::url_to_relative_path(url).ok_or_else(|| {
                    anyhow!("Unsupported downloading URL: {}", url)
                })?);
        match fs::metadata(&full).await {
            Ok(m) if m.is_file() => match full.strip_prefix(&self.cache_dir) {
                Ok(m) => Ok(Some(m.to_owned())),
                Err(e) => Err(anyhow!(
                    "Failed to strip prefix of '{}' path: {}",
                    full.display(),
                    e
                )),
            },
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(anyhow!(
                "Failed to check '{}' file existence: {}",
                full.display(),
                e,
            )),
            _ => {
                self.downloads.send(url.clone()).map_err(|e| {
                    anyhow!(
                        "Failed to schedule '{}' URL for downloading: {}",
                        url,
                        e,
                    )
                })?;
                Ok(None)
            }
        }
    }

    /// Runs job, which awaits for new [`Url`]s for downloading and performs
    /// at most [`Manager::CONCURRENT_DOWNLOADS`] count of downloads at the same
    /// moment.
    ///
    /// The job finishes once [`Manager`] is dropped.
    async fn run_downloads(
        downloads: mpsc::UnboundedReceiver<Url>,
        dst: PathBuf,
        tmp: PathBuf,
    ) {
        let _ = downloads
            .map(move |url| {
                let dst = dst.clone();
                let tmp = tmp.clone();
                async move {
                    AssertUnwindSafe(Self::download(&url, &dst, &tmp))
                        .catch_unwind()
                        .await
                        .map_err(|p| {
                            log::error!(
                                "Panicked while downloading '{}' URL to VOD \
                                 cache: {}",
                                url,
                                display_panic(&p),
                            );
                        })
                        .unwrap_err();
                }
            })
            .buffer_unordered(Self::CONCURRENT_DOWNLOADS)
            .map(Ok)
            .forward(sink::drain())
            .await;
    }

    /// Downloads the given [`Url`] into `dst_dir` using `tmp_dir` for keeping
    /// temporary file while downloading happens.
    ///
    /// The temporary file is required to avoid any problems with partially
    /// downloaded files. That's why, first, the file is downloaded into
    /// `tmp_dir`, and only after downloading is fully complete, it's moved
    /// to `dst_dir`.
    ///
    /// # Errors
    ///
    /// - If file in `tmp_dir` or `dst_dir` cannot be created.
    /// - If the given [`Url`] couldn't be reached or responses with non-success
    ///   HTTP code.
    /// - If downloading of file from the given [`Url`] fails or is interrupted.
    #[allow(clippy::too_many_lines)]
    async fn download(
        url: &Url,
        dst_dir: &Path,
        tmp_dir: &Path,
    ) -> Result<(), anyhow::Error> {
        let rel_path = Self::url_to_relative_path(url)
            .ok_or_else(|| anyhow!("Unsupported downloading URL: {}", url))?;

        let dst_path = dst_dir.join(&rel_path);
        // Early check whether file was downloaded already.
        if matches!(
            fs::metadata(&dst_path).await.map(|m| m.is_file()),
            Ok(true)
        ) {
            log::debug!(
                "URL '{}' already downloaded to '{}' VOD cache file, skipping",
                url,
                dst_path.display(),
            );
            return Ok(());
        }

        let tmp_path = tmp_dir.join(&rel_path);
        // Early check whether file is downloading at the moment.
        if matches!(
            fs::metadata(&tmp_path).await.map(|m| m.is_file()),
            Ok(true)
        ) {
            log::debug!(
                "URL '{}' already downloading at the moment to '{}' VOD cache \
                 file, skipping",
                url,
                dst_path.display(),
            );
            return Ok(());
        }
        // Prepare parent directory for temporary file.
        if let Some(path) = tmp_path.as_path().parent() {
            fs::create_dir_all(path).await.map_err(|e| {
                anyhow!("Failed to create '{}' dir: {}", path.display(), e)
            })?;
        }

        log::info!(
            "Start downloading '{}' URL to '{}' VOD cache file",
            url,
            dst_path.display(),
        );

        let mut resp = reqwest::get(url.clone())
            .await
            .map_err(|e| anyhow!("Failed to perform GET '{}': {}", url, e))?
            .error_for_status()
            .map_err(|e| anyhow!("Bad response for GET '{}': {}", url, e))?
            .bytes_stream()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            .into_async_read()
            .compat();

        let tmp_file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
            .await
            .map(Some)
            .or_else(|e| {
                if let io::ErrorKind::NotFound = e.kind() {
                    Ok(None)
                } else {
                    Err(e)
                }
            })
            .map_err(|e| {
                anyhow!("Failed to create '{}' file: {}", tmp_path.display(), e)
            })?;
        if tmp_file.is_none() {
            return Ok(());
        }
        let mut tmp_file = tmp_file.unwrap();

        let _ = io::copy(&mut resp, &mut tmp_file).await.map_err(|e| {
            anyhow!(
                "Failed to download into '{}' file: {}",
                tmp_path.display(),
                e,
            )
        })?;

        match fs::metadata(&dst_path).await {
            // Check whether file has been downloaded concurrently.
            Ok(m) if m.is_file() => {
                log::info!(
                    "URL '{}' has been already concurrently downloaded to '{}' \
                     VOD cache file, skipping",
                    url,
                    dst_path.display(),
                );
                return Ok(());
            }
            // Remove if there is a directory with the same name.
            Ok(m) if m.is_dir() => {
                fs::remove_dir_all(&dst_path).await.map_err(|e| {
                    anyhow!(
                        "Failed to remove '{}' dir: {}",
                        dst_path.display(),
                        e,
                    )
                })?;
            }
            _ => {}
        }
        // Prepare parent directory for destination file.
        if let Some(path) = dst_path.as_path().parent() {
            fs::create_dir_all(path).await.map_err(|e| {
                anyhow!("Failed to create '{}' dir: {}", path.display(), e)
            })?;
        }

        if fs::rename(&tmp_path, &dst_path).await.is_err() {
            // If moving file has failed (due to moving onto another physical
            // disk, for example), then try to copy and delete it explicitly.
            let _ = fs::copy(&tmp_path, &dst_path).await.map_err(|e| {
                anyhow!(
                    "Failed to move downloaded file from '{}' to '{}': {}",
                    tmp_path.display(),
                    dst_path.display(),
                    e,
                )
            })?;
            fs::remove_file(&tmp_path)
                .await
                .map_err(|e|
                    anyhow!(
                        "Failed to remove '{}' file: {}",
                        tmp_path.display(), e,)
                )?;
        }

        log::info!(
            "Successfully downloaded URL '{}' to '{}' VOD cache file",
            url,
            dst_path.display(),
        );

        Ok(())
    }

    /// Extracts path of the file in cache from the given [`Url`].
    ///
    /// If [`None`] is returned, then such [`Url`] is not supported for
    /// downloading.
    #[must_use]
    pub fn url_to_relative_path(url: &Url) -> Option<PathBuf> {
        let prefix = match url.host() {
            Some(url::Host::Domain("api.allatra.video")) => "/storage/videos",
            _ => return None,
        };
        let path = Path::new(url.path()).strip_prefix(prefix).ok()?;

        // Path with `..` segments are not supported due to security reasons:
        // provided URL should never give a possibility to point outside the
        // `Manager::cache_dir`.
        if path.components().any(|c| c == path::Component::ParentDir) {
            return None;
        }

        Some(path.to_owned())
    }
}
