//! [SRS]-based definitions and implementations.
//!
//! [SRS]: https://github.com/ossrs/srs

use std::{
    borrow::Borrow,
    ops::Deref,
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
};

use anyhow::anyhow;
use askama::Template;
use derive_more::{AsRef, Deref, Display, From, Into};
use ephyr_log::{log, slog};
use futures::future::{self, FutureExt as _, TryFutureExt as _};
use smart_default::SmartDefault;
use tokio::{fs, process::Command};

use crate::{api, display_panic, dvr};

/// [SRS] server spawnable as a separate process.
///
/// [SRS]: https://github.com/ossrs/srs
#[derive(Clone, Debug)]
pub struct Server {
    /// Path where [SRS] configuration file should be created.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    conf_path: PathBuf,

    /// Handle to the actual spawned [SRS] process.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    _process: Arc<ServerProcess>,
}

impl Server {
    /// Tries to create and run a new [SRS] server process.
    ///
    /// # Errors
    ///
    /// If [SRS] configuration file fails to be created.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    pub async fn try_new<P: AsRef<Path>>(
        workdir: P,
        cfg: &Config,
    ) -> Result<Self, anyhow::Error> {
        let workdir = workdir.as_ref();
        let mut bin_path = workdir.to_path_buf();
        bin_path.push("objs/srs");

        let mut conf_path = workdir.to_path_buf();
        conf_path.push("conf/srs.conf");

        let http_dir = if cfg.http_server_dir.is_relative() {
            let mut dir = workdir.to_path_buf();
            dir.push(&cfg.http_server_dir);
            dir
        } else {
            cfg.http_server_dir.clone().into()
        };

        // Pre-create directory for HLS.
        let mut hls_dir = http_dir.clone();
        hls_dir.push("hls");
        fs::create_dir_all(&hls_dir).await.map_err(|e| {
            anyhow!(
                "Failed to pre-create HLS directory {} : {}",
                hls_dir.display(),
                e,
            )
        })?;

        // Set directory for dvr::Storage served by this SRS instance.
        let mut dvr_dir = http_dir.clone();
        dvr_dir.push("dvr");
        dvr::Storage { root_path: dvr_dir }.set_global()?;

        let mut cmd = Command::new(bin_path);
        let _ = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .current_dir(workdir)
            .arg("-c")
            .arg(&conf_path);

        let (spawner, abort_handle) = future::abortable(async move {
            loop {
                let cmd = &mut cmd;
                let _ = AssertUnwindSafe(async move {
                    let process = cmd.spawn().map_err(|e| {
                        log::crit!("Cannot start SRS server: {}", e);
                    })?;
                    let out =
                        process.wait_with_output().await.map_err(|e| {
                            log::crit!("Failed to observe SRS server: {}", e);
                        })?;
                    log::crit!(
                        "SRS server stopped with exit code: {}",
                        out.status,
                    );
                    Ok(())
                })
                .unwrap_or_else(|_: ()| ())
                .catch_unwind()
                .await
                .map_err(|p| {
                    log::crit!(
                        "Panicked while spawning/observing SRS server: {}",
                        display_panic(&p),
                    );
                });
            }
        });

        let srv = Self {
            conf_path,
            _process: Arc::new(ServerProcess(abort_handle)),
        };

        // Pre-create SRS conf file.
        srv.refresh(cfg).await?;

        // Start SRS server as a child process.
        drop(tokio::spawn(spawner));

        Ok(srv)
    }

    /// Updates [SRS] configuration file and reloads the spawned [SRS] server
    /// to catch up the changes.
    ///
    /// # Errors
    ///
    /// If [SRS] configuration file fails to be created.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    pub async fn refresh(&self, cfg: &Config) -> anyhow::Result<()> {
        // SRS server reloads automatically on its conf file changes.
        fs::write(
            &self.conf_path,
            cfg.render().map_err(|e| {
                anyhow!("Failed to render SRS config from template: {}", e)
            })?,
        )
        .await
        .map_err(|e| anyhow!("Failed to write SRS config file: {}", e))
    }
}

/// Handle to a spawned [SRS] server process.
///
/// [SRS]: https://github.com/ossrs/srs
#[derive(Clone, Debug)]
struct ServerProcess(future::AbortHandle);

impl Drop for ServerProcess {
    #[inline]
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// ID of [SRS] server client guarded by its participation.
///
/// Once this ID is fully [`Drop`]ped the client will be kicked from [SRS]
/// server.
///
/// [SRS]: https://github.com/ossrs/srs
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ClientId(Arc<u32>);

impl From<u32> for ClientId {
    #[inline]
    fn from(id: u32) -> Self {
        Self(Arc::new(id))
    }
}

impl Deref for ClientId {
    type Target = u32;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl Borrow<u32> for ClientId {
    #[inline]
    fn borrow(&self) -> &u32 {
        &*self
    }
}

impl Drop for ClientId {
    /// Kicks a client behind this [`ClientId`] from [SRS] server it there are
    /// no more copies left.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    fn drop(&mut self) {
        if let Some(&mut client_id) = Arc::get_mut(&mut self.0) {
            drop(tokio::spawn(
                api::srs::Client::kickoff_client(client_id).map_err(move |e| {
                    log::warn!(
                        "Failed to kickoff client {} from SRS: {}",
                        client_id,
                        e,
                    );
                }),
            ));
        }
    }
}

/// Configuration parameters of [SRS] server used by this application.
///
/// [SRS]: https://github.com/ossrs/srs
#[derive(Clone, Debug, Template)]
#[template(path = "restreamer.srs.conf.j2", escape = "none")]
pub struct Config {
    /// Port that [HTTP Callback API][1] is exposed on.
    ///
    /// [1]: https://en.wikipedia.org/wiki/Basic_access_authentication
    pub callback_port: u16,

    /// Path to the directory served by [SRS] HTTP server (HLS chunks, etc).
    ///
    /// [SRS]: https://github.com/ossrs/srs
    pub http_server_dir: DisplayablePath,

    /// Severity of [SRS] server logs.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    pub log_level: LogLevel,
}

/// Severity of [SRS] [server logs][1].
///
/// [SRS]: https://github.com/ossrs/srs
/// [1]: https://github.com/ossrs/srs/wiki/v3_EN_SrsLog#loglevel
#[derive(Clone, Copy, Debug, Display, SmartDefault)]
pub enum LogLevel {
    /// Error level.
    #[display(fmt = "error")]
    Error,

    /// Warning log, without debug log.
    #[display(fmt = "warn")]
    Warn,

    /// Important log, less and [SRS] enables it as a default level.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    #[default]
    #[display(fmt = "trace")]
    Trace,

    /// Detail log, which huts performance.
    ///
    /// [SRS] defaults to disable it when compile.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    #[display(fmt = "info")]
    Info,

    /// Lots of log, which hurts performance.
    ///
    /// [SRS] defaults to disable it when compile.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    #[display(fmt = "verbose")]
    Verbose,
}

impl From<slog::Level> for LogLevel {
    #[inline]
    fn from(lvl: slog::Level) -> Self {
        match lvl {
            slog::Level::Critical | slog::Level::Error => Self::Error,
            slog::Level::Warning | slog::Level::Info => Self::Warn,
            slog::Level::Debug => Self::Trace,
            slog::Level::Trace => Self::Info,
        }
    }
}

/// [`Display`]able wrapper around [`PathBuf`] for using in
/// [`askama::Template`]s.
///
/// [`Display`]: std::fmt::Display
#[derive(AsRef, Clone, Debug, Deref, Display, From, Into)]
#[as_ref(forward)]
#[display(fmt = "{}", "_0.display()")]
pub struct DisplayablePath(PathBuf);
