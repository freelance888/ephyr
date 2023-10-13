//! Data of a concrete kind of a running [FFmpeg] process performing a
//! re-streaming, that allows to spawn and re-spawn it at any time.
//!
//! [FFmpeg]: https://ffmpeg.org

use derive_more::From;
use ephyr_log::{
    tracing,
    tracing::{instrument, Instrument},
    ChildCapture, ParsedMsg, Span,
};
use lazy_static::lazy_static;
use libc::pid_t;
use regex::Regex;
use std::{
    convert::TryInto, fmt::Display, os::unix::process::ExitStatusExt,
    path::Path, time::Duration,
};
use tokio::{io, process::Command, sync::watch};
use url::Url;
use uuid::Uuid;

use crate::{
    dvr,
    ffmpeg::{
        copy_restreamer::CopyRestreamer,
        file_restreamer::FileRestreamer,
        mixing_restreamer::MixingRestreamer,
        restreamer::RestreamerStatus,
        transcoding_restreamer::{TranscodingOptions, TranscodingRestreamer},
    },
    file_manager::LocalFileInfo,
    proc::kill_process,
    state::{self, RestreamKey, State, Status},
};

/// Parse [FFmpeg] log line.
///
/// [FFmpeg]: https://ffmpeg.org
#[allow(dead_code)]
fn parse_ffmpeg_log_line(line: &str) -> ParsedMsg<'_> {
    lazy_static! {
        static ref RE: Regex = Regex::new(concat!(
            r"^(?:.*\s)?\[(?P<level>(?i)",
            r"(?:info|debug|error|fatal|panic|quiet|warning|verbose))\]",
            r"(?P<msg>.*)$",
        ))
        .unwrap();
    }
    if let Some(captures) = RE.captures(line) {
        let message = captures.name("msg").unwrap().as_str();
        let level = captures.name("level").unwrap().as_str();
        ParsedMsg { message, level }
    } else {
        ParsedMsg {
            message: line,
            level: "warn",
        }
    }
}

/// Data of a concrete kind of a running [FFmpeg] process performing a
/// re-streaming, that allows to spawn and re-spawn it at any time.
///
/// [FFmpeg]: https://ffmpeg.org
#[derive(Clone, Debug, From)]
pub enum RestreamerKind {
    /// Re-streaming of a live stream from one URL endpoint to another one "as
    /// is", without performing any live stream modifications, optionally
    /// transmuxing it to the destination format.
    Copy(CopyRestreamer),

    /// Re-streaming of a live stream from one URL endpoint to another one
    /// transcoding it with desired settings, and optionally transmuxing it to
    /// the destination format.
    Transcoding(Box<TranscodingRestreamer>),

    /// Mixing a live stream from one URL endpoint with additional live streams
    /// and re-streaming the result to another endpoint.
    Mixing(MixingRestreamer),

    /// Sourcing a video and audio from local file and streaming it to input
    /// endpoint.
    File(FileRestreamer),
}

impl Display for RestreamerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RestreamerKind::Copy(_r) => write!(f, "RestreamerKind::Copy"),
            RestreamerKind::Transcoding(_r) => {
                write!(f, "RestreamerKind::Transcoding")
            }
            RestreamerKind::Mixing(_r) => write!(f, "RestreamerKind::Mixing"),
            RestreamerKind::File(_r) => write!(f, "RestreamerKind::File"),
        }
    }
}

impl RestreamerKind {
    /// Returns unique ID of this [FFmpeg] re-streaming process.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[inline]
    #[must_use]
    pub fn id<Id: From<Uuid>>(&self) -> Id {
        match self {
            Self::Copy(c) => c.id.into(),
            Self::Transcoding(c) => c.id.into(),
            Self::Mixing(m) => m.id.into(),
            Self::File(m) => m.id.into(),
        }
    }

    /// Returns destination url of [FFmpeg].
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[inline]
    #[must_use]
    pub fn to_url(&self) -> Url {
        match self {
            Self::Copy(c) => c.to_url.clone(),
            Self::Transcoding(t) => t.to_url.clone(),
            Self::Mixing(m) => m.to_url.clone(),
            Self::File(f) => f.to_url.clone(),
        }
    }

    /// Returns source url of [FFmpeg].
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[inline]
    #[must_use]
    pub fn src_url(&self) -> Url {
        match self {
            Self::Copy(c) => c.from_url.clone(),
            Self::Transcoding(t) => t.from_url.clone(),
            Self::Mixing(m) => m.from_url.clone(),
            Self::File(f) => f.from_url.clone(),
        }
    }

    /// Creates a new [FFmpeg] process re-streaming a [`state::InputSrc`] to its
    /// [`state::Input`] endpoint.
    ///
    /// Returns [`None`] if a [FFmpeg] re-streaming process cannot not be
    /// created for the given [`state::Input`], or the later doesn't require it.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[must_use]
    #[instrument(skip_all, fields(
        restream.key=%key, %is_playing_playlist, input.key=%input.key)
    )]
    pub fn from_input(
        input: &state::Input,
        endpoint: &state::InputEndpoint,
        key: &RestreamKey,
        is_playing_playlist: bool,
        with_playback_encoding: bool,
        files: &[LocalFileInfo],
        file_root: &Path,
    ) -> Option<Self> {
        if !input.enabled {
            return None;
        }

        Some(match endpoint.kind {
            state::InputEndpointKind::Rtmp => {
                if is_playing_playlist {
                    return None;
                }
                let from_url =
                    input.src.as_ref()?.src_url(key, files, file_root)?;
                let to_url = endpoint.kind.rtmp_url(key, &input.key);
                let id: Uuid = endpoint.id.into();

                if input.key.is_playback() && with_playback_encoding {
                    Box::new(TranscodingRestreamer {
                        id,
                        from_url,
                        to_url,
                        options: TranscodingOptions::default(),
                    })
                    .into()
                } else {
                    CopyRestreamer {
                        id,
                        from_url,
                        to_url,
                    }
                    .into()
                }
            }

            state::InputEndpointKind::Hls => {
                if !input.is_ready_to_serve() {
                    return None;
                }
                Box::new(TranscodingRestreamer {
                    id: endpoint.id.into(),
                    from_url: state::InputEndpointKind::Rtmp
                        .rtmp_url(key, &input.key),
                    to_url: endpoint.kind.rtmp_url(key, &input.key),
                    options: TranscodingOptions::default(),
                })
                .into()
            }

            state::InputEndpointKind::File => {
                return None;
            }
        })
    }

    /// Creates a new [FFmpeg] process streaming a file from playlist to
    /// [`state::Input`] endpoint.
    ///
    /// Returns [`None`] if a [FFmpeg] re-streaming process cannot not be
    /// created for the given [`state::Playlist`].
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[must_use]
    #[instrument(skip_all, fields(
        restream.key=%restream_key,
        input.key=%input_key,
        file_root=%file_root.display())
    )]
    pub fn from_playlist(
        playlist: &state::Playlist,
        restream_key: &RestreamKey,
        input_key: &state::InputKey,
        file_root: &Path,
    ) -> Option<Self> {
        let from_url =
            playlist.currently_playing_file.as_ref().and_then(|file| {
                if let Ok(from_url) = Url::from_file_path(
                    file_root.join(file.file_id.to_string()),
                )
                .map_err(|()| {
                    tracing::error!(
                        %file.file_id,
                        "Failed to parse `from_url` from `file_id`"
                    );
                }) {
                    Some(from_url)
                } else {
                    None
                }
            });

        let to_url = Url::parse(&format!(
            "rtmp://127.0.0.1:1935/{restream_key}/{input_key}",
        ))
        .map_err(|e| {
            tracing::error!(%e, "Failed to parse `to_url`");
        });

        match (from_url, to_url) {
            (Some(from_url), Ok(to_url)) => Some(Self::File(FileRestreamer {
                id: playlist.id.into(),
                from_url,
                to_url,
            })),
            _ => None,
        }
    }

    /// Creates a new [FFmpeg] process re-streaming a live stream from a
    /// [`state::Restream::input`] to the given [`state::Output::dst`] endpoint.
    ///
    /// `prev` value may be specified to consume already initialized resources,
    /// which are unwanted to be re-created.
    ///
    /// Returns [`None`] if a [FFmpeg] re-streaming process cannot not be
    /// created for the given [`state::Output`].
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[must_use]
    #[instrument(skip_all, fields(
        src=%from_url, dst=%output.dst)
    )]
    pub fn from_output(
        output: &state::Output,
        from_url: &Url,
        prev: Option<&RestreamerKind>,
    ) -> Option<Self> {
        if !output.enabled {
            return None;
        }

        Some(if output.mixins.is_empty() {
            CopyRestreamer {
                id: output.id.into(),
                from_url: from_url.clone(),
                to_url: Self::dst_url(output),
            }
            .into()
        } else {
            MixingRestreamer::new(output, from_url, prev).into()
        })
    }

    /// Extracts the correct [`Url`] acceptable by [FFmpeg] for sinking a live
    /// stream by the given [`state::Output`].
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[inline]
    #[must_use]
    pub(crate) fn dst_url(output: &state::Output) -> Url {
        (output.dst.scheme() == "file")
            .then(|| dvr::Storage::global().file_url(output).unwrap())
            .unwrap_or_else(|| output.dst.clone().into())
    }

    /// Checks whether this [`Restreamer`] must be restarted, as cannot apply
    /// the new `actual` params on itself correctly, without interruptions.
    ///
    /// [`Restreamer`]: crate::ffmpeg::Restreamer
    #[inline]
    #[must_use]
    pub fn needs_restart(&mut self, actual: &Self) -> bool {
        match (self, actual) {
            (Self::Copy(old), Self::Copy(new)) => old.needs_restart(new),
            (Self::Transcoding(old), Self::Transcoding(new)) => {
                old.needs_restart(new)
            }
            (Self::Mixing(old), Self::Mixing(new)) => old.needs_restart(new),
            (Self::File(old), Self::File(new)) => old.needs_restart(new),
            _ => true,
        }
    }

    fn setup_logger(cmd: &mut Command) {
        let loglevel_prefix = "repeat+level";
        _ = match tracing::level_filters::LevelFilter::current().into_level() {
            Some(tracing::Level::DEBUG | tracing::Level::TRACE) => cmd.args([
                "-hide_banner",
                "-loglevel",
                &format!("{loglevel_prefix}+verbose"),
            ]),
            Some(tracing::Level::INFO) => cmd.args([
                "-hide_banner",
                "-loglevel",
                &format!("{loglevel_prefix}+info"),
            ]),
            Some(tracing::Level::WARN) => cmd.args([
                "-hide_banner",
                "-loglevel",
                &format!("{loglevel_prefix}++warning"),
            ]),
            _ => cmd,
        };
    }
    /// Properly setups the given [FFmpeg] [`Command`] before running it.
    ///
    /// The specified [`State`] may be used to retrieve up-to-date parameters,
    /// which don't trigger re-creation of the whole [FFmpeg] re-streaming
    /// process.
    ///
    /// # Errors
    ///
    /// If the given [FFmpeg] [`Command`] fails to be setup.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[inline]
    pub(crate) async fn setup_ffmpeg(
        &self,
        cmd: &mut Command,
        state: &State,
    ) -> io::Result<()> {
        Self::setup_logger(cmd);
        match self {
            Self::Copy(c) => c.setup_ffmpeg(cmd).await?,
            Self::Transcoding(c) => c.setup_ffmpeg(cmd),
            Self::Mixing(m) => m.setup_ffmpeg(cmd, state).await?,
            Self::File(m) => m.setup_ffmpeg(cmd, false).await?,
        };
        Ok(())
    }

    /// Properly runs the given [FFmpeg] [`Command`] awaiting its completion.
    ///
    /// Returns [`Ok`] if the [`kill_rx`] was sent and the ffmpeg process
    /// was stopped properly or if the entire input file was played to the end.
    ///
    /// In case of [`Self::Mixin`] before starting [`Command`]
    /// the FIFO files are created. For each pair of [`Mixin`] and FIFO the
    /// new task are created and transfer data from [`Mixin.stdin`] to FIFO.
    ///
    /// # Errors
    ///
    /// It can return an [`io::Error`] if something unexpected happened and the
    /// [FFmpeg] process was stopped.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[inline]
    #[instrument(skip_all, fields(?cmd))]
    pub(crate) async fn run_ffmpeg(
        &self,
        cmd: Command,
        kill_rx: watch::Receiver<RestreamerStatus>,
    ) -> io::Result<()> {
        if let Self::Mixing(m) = self {
            m.start_fed_mixins_fifo(&kill_rx);
        }
        tracing::debug!("Starting ffmpeg process {cmd:?}");
        Self::run_ffmpeg_(cmd, kill_rx).await
    }

    /// Properly runs the given [FFmpeg] [`Command`] awaiting its completion.
    ///
    /// Returns [`Ok`] if the [`kill_rx`] was sent and the ffmpeg process
    /// was stopped properly or if the entire input file was played to the end.
    ///
    /// # Errors
    ///
    /// It can return an [`io::Error`] if something unexpected happened and the
    /// [FFmpeg] process was stopped.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    async fn run_ffmpeg_(
        mut cmd: Command,
        mut kill_rx: watch::Receiver<RestreamerStatus>,
    ) -> io::Result<()> {
        let process = cmd.spawn()?;

        // To avoid instant resolve on await for `kill_rx`
        _ = *kill_rx.borrow_and_update();

        let pid: pid_t = process
            .id()
            .expect("Failed to retrieve Process ID")
            .try_into()
            .expect("Failed to convert u32 to i32");

        // Task that sends SIGTERM if async stop of ffmpeg was invoked
        let kill_task = tokio::spawn(
            async move {
                _ = kill_rx.changed().await;
                tracing::debug!("Signal for FFmpeg received");
                // It is necessary to send the signal two times and wait after
                // sending the first one to correctly close all ffmpeg processes
                kill_process(pid).expect("Failed to kill process");
                tokio::time::sleep(Duration::from_millis(1)).await;
                kill_process(pid).expect("Failed to kill process");
            }
            .in_current_span(),
        );

        let out = process
            .capture_logs_and_wait_for_output(
                tracing::info_span!(parent: Span::current(), "ffmpeg_proc"),
                parse_ffmpeg_log_line,
            )
            .await?;
        kill_task.abort();

        let status_code = out.status.code();
        let signal_code = out.status.signal();
        // if the process exited because of SIGTERM signal (exit code 255)
        // or exited with 0
        if out.status.success()
            || status_code.and_then(|v| (v == 255).then_some(())).is_some()
            || signal_code.and_then(|v| (v == 15).then_some(())).is_some()
        {
            tracing::debug!(
                status_code,
                signal_code,
                "FFmpeg re-streamer successfully stopped"
            );
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "FFmpeg re-streamer unsuccessfully stopped \
                    with exit code: {}\n{}",
                    out.status,
                    String::from_utf8_lossy(&out.stderr),
                ),
            ))
        }
    }

    /// Renews [`Status`] of this [FFmpeg] re-streaming process in the `actual`
    /// [`State`].
    ///
    /// [FFmpeg]: https://ffmpeg.org
    pub fn renew_status(&self, status: Status, actual: &State) {
        for restream in actual.restreams.lock_mut().iter_mut() {
            if !restream.outputs.is_empty() {
                let my_id = self.id();
                for o in &mut restream.outputs {
                    if o.id == my_id {
                        o.status = status;
                        return;
                    }
                }
            }

            // `Status::Online` for `state::Input` is set by SRS HTTP Callback.
            if status != Status::Online {
                if let Some(endpoint) = restream.input.find_endpoint(self.id())
                {
                    endpoint.status = status;
                    return;
                }
            }
        }
    }
}
