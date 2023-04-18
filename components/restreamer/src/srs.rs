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
use ephyr_log::{
    tracing,
    tracing::{instrument, Instrument, Span},
    ChildCapture, ParsedMsg,
};
use futures::future::{self, FutureExt as _, TryFutureExt as _};
use regex::Regex;
use smart_default::SmartDefault;
use structopt::lazy_static::lazy_static;
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

/// Parse [SRS] log line to extract message
///
/// Description of log format[1].
///
/// # Examples
///
/// ```ignore
/// let r =
///     parse_srs_log("[2014-08-06 10:09:34.579][trace][22314][108] Message");
/// assert_eq!(r, "Message");
/// ```
/// [SRS]: https://github.com/ossrs/srs
/// [1]: https://ossrs.io/lts/en-us/docs/v4/doc/log#log-format
fn parse_srs_log_line(line: &str) -> ParsedMsg<'_> {
    lazy_static! {
        static ref RE: Regex = Regex::new(concat!(
            r".*\[\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}\]",
            r"\[(?P<level>(?i)(?:verbose|info|trace|warn|error))\]",
            r"(?:\[\d+\])?(?:\[\w+\])?(?:\[\d+\])?",
            r"(\s(?P<msg>.*))?$"
        ))
        .unwrap();
    }
    if let Some(captures) = RE.captures(line) {
        let message = captures.name("msg").unwrap().as_str().trim_start();
        let level = captures.name("level").unwrap().as_str().trim();
        ParsedMsg { message, level }
    } else {
        ParsedMsg {
            message: line,
            level: "warn",
        }
    }
}

impl Server {
    /// Tries to create and run a new [SRS] server process.
    ///
    /// # Errors
    ///
    /// If [SRS] configuration file fails to be created.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    #[instrument(err, name = "srs", skip_all)]
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
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .current_dir(workdir)
            .arg("-c")
            .arg(&conf_path);

        let (spawner, abort_handle) = future::abortable(
            async move {
                loop {
                    let cmd = &mut cmd;
                    let _ = AssertUnwindSafe(async move {
                        let process = cmd.spawn().map_err(|e| {
                            tracing::error!("Cannot start SRS server: {e}");
                        })?;
                        let out = process
                            .capture_logs_and_wait_for_output(
                                tracing::info_span!(
                                    parent: Span::current(),
                                    "srs_proc"
                                ),
                                parse_srs_log_line,
                            )
                            .await
                            .map_err(|e| {
                                tracing::error!(
                                    "Failed to observe SRS server: {e}"
                                );
                            })?;
                        tracing::warn!(
                            "SRS server stopped with exit code: {}",
                            out.status
                        );
                        Ok(())
                    })
                    .unwrap_or_else(|_: ()| ())
                    .catch_unwind()
                    .await
                    .map_err(|p| {
                        tracing::error!(
                            "Panicked while spawning/observing SRS server: {}",
                            display_panic(&p),
                        );
                    });
                }
            }
            .in_current_span(),
        );

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
    //#[instrument(err, skip_all, fields(group = "srs"))]
    pub async fn refresh(&self, cfg: &Config) -> anyhow::Result<()> {
        // SRS server reloads automatically on its conf file changes.
        fs::write(
            &self.conf_path,
            cfg.render().map_err(|e| {
                anyhow!("Failed to render SRS config from template: {e}")
            })?,
        )
        .await
        .map_err(|e| anyhow!("Failed to write SRS config file: {e}"))
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
pub struct ClientId(Arc<String>);

impl ClientId {
    /// Returns value of `client_id`
    #[must_use]
    pub fn get_value(&self) -> Option<String> {
        Some(self.0.as_ref().to_string())
    }
}

impl From<String> for ClientId {
    #[inline]
    fn from(id: String) -> Self {
        Self(Arc::new(id))
    }
}

impl Deref for ClientId {
    type Target = String;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Borrow<String> for ClientId {
    #[inline]
    fn borrow(&self) -> &String {
        self
    }
}

impl Drop for ClientId {
    /// Kicks a client behind this [`ClientId`] from [SRS] server it there are
    /// no more copies left.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    fn drop(&mut self) {
        if let Some(client_id) = Arc::get_mut(&mut self.0).cloned() {
            drop(tokio::spawn(
                api::srs::Client::kickoff_client(client_id.clone()).map_err(
                    move |e| {
                        tracing::error!(
                            client=client_id,
                            e=%e,
                            "Failed to kickoff client",
                        );
                    },
                ),
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
/// [1]: https://ossrs.io/lts/en-us/docs/v4/doc/log
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

impl From<tracing::Level> for LogLevel {
    #[inline]
    fn from(lvl: tracing::Level) -> Self {
        match lvl {
            tracing::Level::ERROR => Self::Error,
            tracing::Level::WARN | tracing::Level::INFO => Self::Warn,
            tracing::Level::DEBUG => Self::Trace,
            tracing::Level::TRACE => Self::Info,
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
