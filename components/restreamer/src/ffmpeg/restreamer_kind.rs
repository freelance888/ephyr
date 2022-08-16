//! Data of a concrete kind of a running [FFmpeg] process performing a
//! re-streaming, that allows to spawn and re-spawn it at any time.
//!
//! [FFmpeg]: https://ffmpeg.org

use derive_more::From;
use tokio::{io, process::Command, sync::watch};
use url::Url;
use uuid::Uuid;

use crate::{
    dvr,
    ffmpeg::{
        copy_restreamer::CopyRestreamer,
        mixing_restreamer::MixingRestreamer,
        restreamer::RestreamerStatus,
        transcoding_restreamer::TranscodingRestreamer,
        util::{
            kill_ffmpeg_process_by_sigterm,
            wraps_ffmpeg_process_output_with_result,
        },
    },
    state::{self, State, Status},
};

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
    ) -> Option<Self> {
        if !input.enabled {
            return None;
        }

        Some(match endpoint.kind {
            state::InputEndpointKind::Rtmp => {
                let from_url = match input.src.as_ref()? {
                    state::InputSrc::Remote(remote) => {
                        remote.url.clone().into()
                    }
                    state::InputSrc::Failover(s) => {
                        s.inputs.iter().find_map(|i| {
                            i.endpoints.iter().find_map(|e| {
                                (e.is_rtmp() && e.status == Status::Online)
                                    .then(|| e.kind.rtmp_url(key, &i.key))
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
        })
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
        };
        Ok(())
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
    #[inline]
    pub(crate) async fn run_ffmpeg(
        &self,
        cmd: Command,
        kill_rx: watch::Receiver<RestreamerStatus>,
    ) -> io::Result<()> {
        if let Self::Mixing(m) = self {
            m.run_ffmpeg_with_mixins(cmd, kill_rx).await
        } else {
            Self::run_standard_ffmpeg(cmd, kill_rx).await
        }
    }

    /// Properly runs the given [FFmpeg] [`Command`] without writing to
    /// FIFO and awaits its completion.
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
    async fn run_standard_ffmpeg(
        mut cmd: Command,
        mut kill_rx: watch::Receiver<RestreamerStatus>,
    ) -> io::Result<()> {
        let process = cmd.spawn()?;

        if *kill_rx.borrow_and_update() == RestreamerStatus::Finished {
            return Ok(());
        }

        let kill_task = kill_ffmpeg_process_by_sigterm(process.id(), kill_rx);
        let out = process.wait_with_output().await?;
        kill_task.abort();

        wraps_ffmpeg_process_output_with_result(&out)
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
