//! Data of a concrete kind of a running [FFmpeg] process performing a
//! re-streaming, that allows to spawn and re-spawn it at any time.
//!
//! [FFmpeg]: https://ffmpeg.org

use std::path::Path;
use std::process::ExitStatus;
use derive_more::From;
use tokio::{io, process::Command};
use url::Url;
use uuid::Uuid;
use nix::unistd::Pid;
use nix::sys::signal::{self, Signal};
use nix::libc::{kill, pid_t};

use ephyr_log::log;
use crate::{dvr, ffmpeg::{
    copy_restreamer::CopyRestreamer, mixing_restreamer::MixingRestreamer,
    transcoding_restreamer::TranscodingRestreamer, file_restreamer::FileRestreamer
}, file_manager, state::{self, State, Status}};
use std::result::Result::Err;

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
    Transcoding(TranscodingRestreamer),

    /// Mixing a live stream from one URL endpoint with additional live streams
    /// and re-streaming the result to another endpoint.
    Mixing(MixingRestreamer),

    /// Sourcing a video and audio from local file and streaming it to input
    /// endpoint.
    File(FileRestreamer),
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

    /// Creates a new [FFmpeg] process re-streaming a [`state::InputSrc`] to its
    /// [`state::Input`] endpoint.
    ///
    /// Returns [`None`] if a [FFmpeg] re-streaming process cannot not be
    /// created for the given [`state::Input`], or the later doesn't require it.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[must_use]
    pub fn from_input(
        input: &state::Input,
        endpoint: &state::InputEndpoint,
        key: &state::RestreamKey,
        is_playing_playlist: bool,
        files: &[file_manager::LocalFileInfo],
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
                let from_url = match input.src.as_ref()? {
                    state::InputSrc::Remote(remote) => {
                        remote.url.clone().into()
                    }
                    state::InputSrc::Failover(s) => {
                        s.inputs.iter().find_map(|i| {
                            i.endpoints.iter().find_map(|e| {
                                if e.is_rtmp() && e.status == Status::Online {
                                    Some(e.kind.rtmp_url(key, &i.key))
                                } else if i.enabled
                                    && e.is_file()
                                    && e.file_id.is_some()
                                    && files.iter().any(|f| {
                                    e.file_id.as_ref() == Some(&f.file_id)
                                        && f.state == file_manager::FileState::Local
                                })
                                {
                                    url::Url::from_file_path(
                                        file_root.join(
                                            e.file_id
                                                .as_ref()
                                                .unwrap_or(&"".to_string()),
                                        ),
                                    )
                                        .ok()
                                } else {
                                    None
                                }
                            })
                        })?
                    }
                };
                CopyRestreamer {
                    id: endpoint.id.into(),
                    from_url,
                    to_url: endpoint.kind.rtmp_url(key, &input.key),
                }
                .into()
            }

            state::InputEndpointKind::Hls => {
                if !input.is_ready_to_serve() {
                    return None;
                }
                TranscodingRestreamer {
                    id: endpoint.id.into(),
                    from_url: state::InputEndpointKind::Rtmp
                        .rtmp_url(key, &input.key),
                    to_url: endpoint.kind.rtmp_url(key, &input.key),
                    vcodec: Some("libx264".into()),
                    vprofile: Some("baseline".into()),
                    vpreset: Some("superfast".into()),
                    acodec: Some("libfdk_aac".into()),
                }
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
    pub fn from_playlist(
        playlist: &state::Playlist,
        restream_key: &state::RestreamKey,
        input_key: &state::InputKey,
        file_root: &Path,
    ) -> Self {
        let file_id =
            &playlist.currently_playing_file.as_ref().unwrap().file_id;
        let from_url = Url::from_file_path(file_root.join(file_id)).unwrap();
        Self::File(
            FileRestreamer {
                id: playlist.id.into(),
                from_url,
                to_url: Url::parse(&format!(
                    "rtmp://127.0.0.1:1935/{}/{}",
                    restream_key, input_key,
                ))
                    .unwrap(),
            }
                .into(),
        )
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
    /// # Errors
    ///
    /// This method doesn't return [`Ok`] as the running [FFmpeg] [`Command`] is
    /// aborted by dropping and is intended to never stop. If it returns, than
    /// an [`io::Error`] occurs and the [FFmpeg] [`Command`] cannot run.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[inline]
    pub(crate) async fn run_ffmpeg(&self, cmd: Command, mut kill_rx: tokio::sync::watch::Receiver<i32>) -> io::Result<()> {
        if let Self::Mixing(m) = self {
            m.run_ffmpeg(cmd, kill_rx).await
        } else {
            Self::run_ffmpeg_no_stdin(cmd, kill_rx).await
        }
    }

    /// Properly runs the given [FFmpeg] [`Command`] without writing to its
    /// STDIN and awaits its completion.
    ///
    /// # Errors
    ///
    /// This method doesn't return [`Ok`] as the running [FFmpeg] [`Command`] is
    /// aborted by dropping and is intended to never stop. If it returns, than
    /// an [`io::Error`] occurs and the [FFmpeg] [`Command`] cannot run.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    async fn run_ffmpeg_no_stdin(mut cmd: Command, mut kill_rx: tokio::sync::watch::Receiver<i32>) -> io::Result<()> {
        let process = cmd.spawn()?;
        let process_id = process.id().unwrap() as pid_t;

        let exit_signal = kill_rx.borrow_and_update().clone();
        if exit_signal != 0 {
            return Ok(());
        }

        tokio::select! {
            out = process.wait_with_output() => {
                let out = out?;
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "FFmpeg re-streamer stopped with exit code: {}\n{}",
                        out.status,
                        String::from_utf8_lossy(&out.stderr),
                    ),
                ))
            }

            _ = kill_rx.changed() => {
                log::info!("Killing ffmpeg with SIGTERM");
                signal::kill(Pid::from_raw(process_id), Signal::SIGTERM).unwrap();
                Ok(())
            }
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
                fn renew_input_status(
                    input: &mut state::Input,
                    status: Status,
                    my_id: state::EndpointId,
                ) -> bool {
                    if let Some(endpoint) =
                        input.endpoints.iter_mut().find(|e| e.id == my_id)
                    {
                        endpoint.status = status;
                        return true;
                    }

                    if let Some(state::InputSrc::Failover(s)) =
                        input.src.as_mut()
                    {
                        for i in &mut s.inputs {
                            if renew_input_status(i, status, my_id) {
                                return true;
                            }
                        }
                    }

                    false
                }

                if renew_input_status(&mut restream.input, status, self.id()) {
                    return;
                }
            }
        }
    }
}
