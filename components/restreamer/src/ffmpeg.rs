//! [FFmpeg]-based definitions and implementations.
//!
//! [FFmpeg]: https://ffmpeg.org

use std::{
    borrow::Cow,
    collections::HashMap,
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::Duration,
};

use derive_more::From;
use ephyr_log::{log, Drain as _};
use futures::{future, pin_mut, FutureExt as _, TryFutureExt as _};
use tokio::{io, process::Command, sync::Mutex, time};
use url::Url;
use uuid::Uuid;

use crate::{
    display_panic, dvr,
    state::{self, Delay, MixinId, MixinSrcUrl, State, Status, Volume},
    teamspeak,
};
use std::result::Result::Err;
use crate::types::DroppableAbortHandle;

/// Pool of [FFmpeg] processes performing re-streaming of a media traffic.
///
/// [FFmpeg]: https://ffmpeg.org
#[derive(Debug)]
pub struct RestreamersPool {
    /// Path to a [FFmpeg] binary used for spawning processes.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    ffmpeg_path: PathBuf,

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
    state: State,
}

impl RestreamersPool {
    /// Creates a new [`RestreamersPool`] out of the given parameters.
    #[inline]
    #[must_use]
    pub fn new<P: Into<PathBuf>>(ffmpeg_path: P, state: State) -> Self {
        Self {
            ffmpeg_path: ffmpeg_path.into(),
            pool: HashMap::new(),
            state,
        }
    }

    /// Adjusts this [`RestreamersPool`] to run [FFmpeg] re-streaming processes
    /// according to the given renewed [`state::Restream`]s.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    pub fn apply(&mut self, restreams: &[state::Restream]) {
        // The most often case is when one new FFmpeg process is added.
        let mut new_pool = HashMap::with_capacity(self.pool.len() + 1);

        for r in restreams {
            self.apply_input(&r.key, &r.input, &mut new_pool);

            if !r.input.enabled || !r.input.is_ready_to_serve() {
                continue;
            }

            let input_url = match r.main_input_rtmp_endpoint_url() {
                Ok(input_url) => input_url,
                Err(e) => {
                    log::error!(
                        "Failed to get main input RTMP endpoint: {}",
                        e
                    );
                    continue;
                }
            };
            for o in &r.outputs {
                let _ = self.apply_output(&input_url, o, &mut new_pool);
            }
        }

        self.pool = new_pool;
    }

    /// Traverses the given [`state::Input`] filling the `new_pool` with
    /// required [FFmpeg] re-streaming processes. Tries to preserve already
    /// running [FFmpeg] processes in its `pool` as much as possible.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    fn apply_input(
        &mut self,
        key: &state::RestreamKey,
        input: &state::Input,
        new_pool: &mut HashMap<Uuid, Restreamer>,
    ) {
        if let Some(state::InputSrc::Failover(s)) = &input.src {
            for i in &s.inputs {
                self.apply_input(key, i, new_pool);
            }
        }
        for endpoint in &input.endpoints {
            let _ = self.apply_input_endpoint(key, input, endpoint, new_pool);
        }
    }

    /// Inspects the given [`state::InputEndpoint`] filling the `new_pool` with
    /// a required [FFmpeg] re-streaming process. Tries to preserve already
    /// running [FFmpeg] processes in its `pool` as much as possible.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    fn apply_input_endpoint(
        &mut self,
        key: &state::RestreamKey,
        input: &state::Input,
        endpoint: &state::InputEndpoint,
        new_pool: &mut HashMap<Uuid, Restreamer>,
    ) -> Option<()> {
        let id = endpoint.id.into();

        let new_kind = RestreamerKind::from_input(input, endpoint, key)?;

        let process = self
            .pool
            .remove(&id)
            .and_then(|mut p| (!p.kind.needs_restart(&new_kind)).then(|| p))
            .unwrap_or_else(|| {
                Restreamer::run(
                    self.ffmpeg_path.clone(),
                    new_kind,
                    self.state.clone(),
                )
            });

        drop(new_pool.insert(id, process));
        Some(())
    }

    /// Inspects the given [`state::Output`] filling the `new_pool` with a
    /// required [FFmpeg] re-streaming process. Tries to preserve already
    /// running [FFmpeg] processes in its `pool` as much as possible.
    ///
    /// [FFmpeg]: https://ffmpeg.org
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

        let process = self
            .pool
            .remove(&id)
            .and_then(|mut p| (!p.kind.needs_restart(&new_kind)).then(|| p))
            .unwrap_or_else(|| {
                Restreamer::run(
                    self.ffmpeg_path.clone(),
                    new_kind,
                    self.state.clone(),
                )
            });

        drop(new_pool.insert(id, process));
        Some(())
    }
}

/// Handle to a running [FFmpeg] process performing a re-streaming.
///
/// [FFmpeg]: https://ffmpeg.org
#[derive(Debug)]
pub struct Restreamer {
    /// Abort handle of a spawned [FFmpeg] process of this [`Restreamer`].
    ///
    /// [FFmpeg]: https://ffmpeg.org
    abort: DroppableAbortHandle,

    /// Kind of a spawned [FFmpeg] process describing the actual job it
    /// performs.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    kind: RestreamerKind,
}

impl Restreamer {
    /// Creates a new [`Restreamer`] spawning the actual [FFmpeg] process in
    /// background. Once this [`Restreamer`] is dropped, its [FFmpeg] process is
    /// aborted.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[must_use]
    pub fn run<P: AsRef<Path> + Send + 'static>(
        ffmpeg_path: P,
        kind: RestreamerKind,
        state: State,
    ) -> Self {
        let (kind_for_abort, state_for_abort) = (kind.clone(), state.clone());

        let kind_for_spawn = kind.clone();
        let (spawner, abort_handle) = future::abortable(async move {
            loop {
                let (kind, state) = (&kind_for_spawn, &state);

                let mut cmd = Command::new(ffmpeg_path.as_ref());

                let _ = AssertUnwindSafe(
                    async move {
                        kind.renew_status(Status::Initializing, state);

                        kind.setup_ffmpeg(
                            cmd.kill_on_drop(true)
                                .stdin(Stdio::null())
                                .stdout(Stdio::null())
                                .stderr(Stdio::piped()),
                            state,
                        )
                        .map_err(|e| {
                            log::error!(
                                "Failed to setup FFmpeg re-streamer: {}",
                                e,
                            );
                        })
                        .await?;

                        let running = kind.run_ffmpeg(cmd);
                        pin_mut!(running);

                        let set_online = async move {
                            time::delay_for(Duration::from_secs(5)).await;
                            kind.renew_status(Status::Online, state);
                            future::pending::<()>().await;
                            Ok(())
                        };
                        pin_mut!(set_online);

                        future::try_select(running, set_online)
                            .await
                            .map_err(|e| {
                                log::error!(
                                    "Failed to run FFmpeg re-streamer: {}",
                                    e.factor_first().0,
                                );
                            })
                            .map(|r| r.factor_first().0)
                    }
                    .unwrap_or_else(|_| {
                        kind.renew_status(Status::Offline, state);
                    }),
                )
                .catch_unwind()
                .await
                .map_err(|p| {
                    log::crit!(
                        "Panicked while spawning/observing FFmpeg \
                         re-streamer: {}",
                        display_panic(&p),
                    );
                });

                time::delay_for(Duration::from_secs(2)).await;
            }
        });

        // Spawn FFmpeg re-streamer as a child process.
        drop(tokio::spawn(spawner.map(move |_| {
            kind_for_abort.renew_status(Status::Offline, &state_for_abort);
        })));

        Self {
            abort: DroppableAbortHandle::new(abort_handle),
            kind,
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
    fn dst_url(output: &state::Output) -> Url {
        (output.dst.scheme() == "file")
            .then(|| dvr::Storage::global().file_url(output).unwrap())
            .unwrap_or_else(|| output.dst.clone().into())
    }

    /// Checks whether this [`Restreamer`] must be restarted, as cannot apply
    /// the new `actual` params on itself correctly, without interruptions.
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
    async fn setup_ffmpeg(
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
    /// # Errors
    ///
    /// This method doesn't return [`Ok`] as the running [FFmpeg] [`Command`] is
    /// aborted by dropping and is intended to never stop. If it returns, than
    /// an [`io::Error`] occurs and the [FFmpeg] [`Command`] cannot run.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[inline]
    async fn run_ffmpeg(&self, cmd: Command) -> io::Result<()> {
        if let Self::Mixing(m) = self {
            m.run_ffmpeg(cmd).await
        } else {
            Self::run_ffmpeg_no_stdin(cmd).await
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
    async fn run_ffmpeg_no_stdin(mut cmd: Command) -> io::Result<()> {
        let process = cmd.spawn()?;

        let out = process.wait_with_output().await?;

        Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "FFmpeg re-streamer stopped with exit code: {}\n{}",
                out.status,
                String::from_utf8_lossy(&out.stderr),
            ),
        ))
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

/// Kind of a [FFmpeg] re-streaming process that re-streams a live stream from
/// one URL endpoint to another one "as is", without performing any live stream
/// modifications, optionally transmuxing it to the destination format.
///
/// [FFmpeg]: https://ffmpeg.org
#[derive(Clone, Debug)]
pub struct CopyRestreamer {
    /// ID of an element in a [`State`] this [`CopyRestreamer`] process is
    /// related to.
    pub id: Uuid,

    /// [`Url`] to pull a live stream from.
    pub from_url: Url,

    /// [`Url`] to publish the pulled live stream onto.
    pub to_url: Url,
}

impl CopyRestreamer {
    /// Checks whether this [`CopyRestreamer`] process must be restarted, as
    /// cannot apply the new `actual` params on itself correctly, without
    /// interruptions.
    #[inline]
    #[must_use]
    pub fn needs_restart(&self, actual: &Self) -> bool {
        self.from_url != actual.from_url || self.to_url != actual.to_url
    }

    /// Properly setups the given [FFmpeg] [`Command`] for this
    /// [`CopyRestreamer`] before running it.
    ///
    /// # Errors
    ///
    /// If the given [FFmpeg] [`Command`] fails to be setup.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    async fn setup_ffmpeg(&self, cmd: &mut Command) -> io::Result<()> {
        let _ = match self.from_url.scheme() {
            "http" | "https"
                if Path::new(self.from_url.path()).extension()
                    == Some("m3u8".as_ref()) =>
            {
                cmd.arg("-re")
            }

            "rtmp" | "rtmps" => cmd,

            _ => unimplemented!(),
        }
        .args(&["-i", self.from_url.as_str()]);

        let _ = match self.to_url.scheme() {
            "file"
                if Path::new(self.to_url.path()).extension()
                    == Some("flv".as_ref()) =>
            {
                cmd.args(&["-c", "copy"])
                    .arg(dvr::new_file_path(&self.to_url).await?)
            }

            "icecast" => cmd
                .args(&["-c:a", "libmp3lame", "-b:a", "64k"])
                .args(&["-f", "mp3", "-content_type", "audio/mpeg"])
                .arg(self.to_url.as_str()),

            "rtmp" | "rtmps" => cmd
                .args(&["-c", "copy"])
                .args(&["-f", "flv"])
                .arg(self.to_url.as_str()),

            "srt" => cmd
                .args(&["-c", "copy"])
                .args(&["-strict", "-2", "-y", "-f", "mpegts"])
                .arg(self.to_url.as_str()),

            _ => unimplemented!(),
        };
        Ok(())
    }
}

/// Kind of a [FFmpeg] re-streaming process that re-streams a live stream from
/// one URL endpoint to another one transcoding it with desired settings, and
/// optionally transmuxing it to the destination format.
///
/// [FFmpeg]: https://ffmpeg.org
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscodingRestreamer {
    /// ID of an element in a [`State`] this [`TranscodingRestreamer`] process
    /// is related to.
    pub id: Uuid,

    /// [`Url`] to pull a live stream from.
    pub from_url: Url,

    /// [`Url`] to publish the transcoded live stream onto.
    pub to_url: Url,

    /// [FFmpeg video encoder][1] to encode the transcoded live stream with.
    ///
    /// [1]: https://ffmpeg.org/ffmpeg-codecs.html#Video-Encoders
    pub vcodec: Option<Cow<'static, str>>,

    /// [Preset] of the [`TranscodingRestreamer::vcodec`] if it has one.
    ///
    /// [Preset]: https://trac.ffmpeg.org/wiki/Encode/H.264#Preset
    pub vpreset: Option<Cow<'static, str>>,

    /// [Profile] of the [`TranscodingRestreamer::vcodec`] if it has one.
    ///
    /// [Profile]: https://trac.ffmpeg.org/wiki/Encode/H.264#Profile
    pub vprofile: Option<Cow<'static, str>>,

    /// [FFmpeg audio encoder][1] to encode the transcoded live stream with.
    ///
    /// [1]: https://ffmpeg.org/ffmpeg-codecs.html#Audio-Encoders
    pub acodec: Option<Cow<'static, str>>,
}

impl TranscodingRestreamer {
    /// Checks whether this [`TranscodingRestreamer`] process must be restarted,
    /// as cannot apply the new `actual` params on itself correctly, without
    /// interruptions.
    #[inline]
    #[must_use]
    pub fn needs_restart(&self, actual: &Self) -> bool {
        self != actual
    }

    /// Properly setups the given [FFmpeg] [`Command`] for this
    /// [`TranscodingRestreamer`] before running it.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    fn setup_ffmpeg(&self, cmd: &mut Command) {
        let _ = cmd.args(&["-i", self.from_url.as_str()]);

        if let Some(val) = self.vcodec.as_ref() {
            let _ = cmd.args(&["-c:v", val]);
        }
        if let Some(val) = self.vpreset.as_ref() {
            let _ = cmd.args(&["-preset", val]);
        }
        if let Some(val) = self.vprofile.as_ref() {
            let _ = cmd.args(&["-profile:v", val]);
        }

        if let Some(val) = self.acodec.as_ref() {
            let _ = cmd.args(&["-c:a", val]);
        }

        let _ = match self.to_url.scheme() {
            "rtmp" | "rtmps" => cmd.args(&["-f", "flv"]),
            _ => unimplemented!(),
        }
        .arg(self.to_url.as_str());
    }
}

/// Kind of a [FFmpeg] re-streaming process that mixes a live stream from one
/// URL endpoint with some additional live streams and re-streams the result to
/// another endpoint.
///
/// [FFmpeg]: https://ffmpeg.org
#[derive(Clone, Debug)]
pub struct MixingRestreamer {
    /// ID of an element in a [`State`] this [`MixingRestreamer`] process is
    /// related to.
    pub id: Uuid,

    /// [`Url`] to pull a live stream from.
    pub from_url: Url,

    /// [`Url`] to publish the mixed live stream onto.
    pub to_url: Url,

    /// [`Volume`] rate to mix an audio of the original pulled live stream with.
    pub orig_volume: Volume,

    /// [ZeroMQ] port of a spawned [FFmpeg] process listening to a real-time
    /// filter updates of the original pulled live stream during mixing process.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    /// [ZeroMQ]: https://zeromq.org
    pub orig_zmq_port: u16,

    /// Additional live streams to be mixed with the original one before being
    /// re-streamed to the [`MixingRestreamer::to_url`].
    pub mixins: Vec<Mixin>,
}

impl MixingRestreamer {
    /// Creates a new [`MixingRestreamer`] out of the given [`state::Output`].
    ///
    /// `prev` value may be specified to consume already initialized resources,
    /// which are unwanted to be re-created.
    #[must_use]
    pub fn new(
        output: &state::Output,
        from_url: &Url,
        mut prev: Option<&RestreamerKind>,
    ) -> Self {
        let prev = prev.as_mut().and_then(|kind| {
            if let RestreamerKind::Mixing(r) = kind {
                Some(&r.mixins)
            } else {
                None
            }
        });
        Self {
            id: output.id.into(),
            from_url: from_url.clone(),
            to_url: RestreamerKind::dst_url(output),
            orig_volume: output.volume,
            orig_zmq_port: new_unique_zmq_port(),
            mixins: output
                .mixins
                .iter()
                .map(|m| {
                    Mixin::new(
                        m,
                        output.label.as_ref(),
                        prev.and_then(|p| p.iter().find(|p| p.id == m.id)),
                    )
                })
                .collect(),
        }
    }

    /// Checks whether this [`MixingRestreamer`] process must be restarted, as
    /// cannot apply the new `actual` params on itself correctly, without
    /// interruptions.
    #[inline]
    #[must_use]
    pub fn needs_restart(&mut self, actual: &Self) -> bool {
        if self.from_url != actual.from_url
            || self.to_url != actual.to_url
            || self.mixins.len() != actual.mixins.len()
        {
            return true;
        }

        for (curr, actual) in self.mixins.iter().zip(actual.mixins.iter()) {
            if curr.needs_restart(actual) {
                return true;
            }
        }

        if self.orig_volume != actual.orig_volume {
            self.orig_volume = actual.orig_volume;
            tune_volume(self.id, self.orig_zmq_port, self.orig_volume);
        }
        for (curr, actual) in self.mixins.iter_mut().zip(actual.mixins.iter()) {
            if curr.volume != actual.volume {
                curr.volume = actual.volume;
                tune_volume(curr.id.into(), curr.zmq_port, curr.volume);
            }
        }

        false
    }

    /// Properly setups the given [FFmpeg] [`Command`] for this
    /// [`MixingRestreamer`] before running it.
    ///
    /// The specified [`State`] is used to retrieve up-to-date [`Volume`]s, as
    /// their changes don't trigger re-creation of the whole [FFmpeg]
    /// re-streaming process.
    ///
    /// # Errors
    ///
    /// If the given [FFmpeg] [`Command`] fails to be setup.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[allow(clippy::too_many_lines)]
    async fn setup_ffmpeg(
        &self,
        cmd: &mut Command,
        state: &State,
    ) -> io::Result<()> {
        let my_id = self.id.into();

        // We need up-to-date values of `Volume` here, right from the `State`,
        // as they won't be updated in a closured `self` value.
        let output =
            state.restreams.lock_ref().iter().find_map(|r| {
                r.outputs.iter().find(|o| o.id == my_id).cloned()
            });

        if ephyr_log::logger().is_debug_enabled() {
            let _ = cmd.stderr(Stdio::inherit()).args(&["-loglevel", "debug"]);
        } else {
            let _ = cmd.stderr(Stdio::null());
        }

        if self.mixins.iter().any(|m| m.stdin.is_some()) {
            let _ = cmd.stdin(Stdio::piped());
        }

        let orig_volume =
            output.as_ref().map_or(self.orig_volume, |o| o.volume);

        // WARNING: The filters order matters here!
        let mut filter_complex = Vec::with_capacity(self.mixins.len() + 1);
        filter_complex.push(format!(
            "[0:a]\
               volume@{orig_id}={volume},\
               aresample=48000,\
               azmq=bind_address=tcp\\\\\\://127.0.0.1\\\\\\:{port}\
             [{orig_id}]",
            orig_id = self.id,
            volume = orig_volume.display_as_fraction(),
            port = self.orig_zmq_port,
        ));
        let _ = cmd.args(&["-i", self.from_url.as_str()]);

        for (n, mixin) in self.mixins.iter().enumerate() {
            let mut extra_filters = String::new();

            let _ = match mixin.url.scheme() {
                "ts" => {
                    extra_filters.push_str("aresample=async=1,");
                    cmd.args(&["-thread_queue_size", "512"])
                        .args(&["-f", "f32be"])
                        .args(&["-sample_rate", "48000"])
                        .args(&["-channels", "2"])
                        .args(&["-use_wallclock_as_timestamps", "true"])
                        .args(&["-i", "pipe:0"])
                }

                "http" | "https"
                    if Path::new(mixin.url.path()).extension()
                        == Some("mp3".as_ref()) =>
                {
                    extra_filters.push_str("aresample=48000,");
                    cmd.args(&["-i", mixin.url.as_str()])
                }

                _ => unimplemented!(),
            };

            if !mixin.delay.is_zero() {
                extra_filters.push_str(&format!(
                    "adelay=delays={}:all=1,",
                    mixin.delay.as_millis(),
                ));
            }

            let volume = output
                .as_ref()
                .and_then(|o| {
                    o.mixins
                        .iter()
                        .find_map(|m| (m.id == mixin.id).then(|| m.volume))
                })
                .unwrap_or(mixin.volume);

            // WARNING: The filters order matters here!
            filter_complex.push(format!(
                "[{num}:a]\
                   volume@{mixin_id}={volume},\
                   {extra_filters}\
                   azmq=bind_address=tcp\\\\\\://127.0.0.1\\\\\\:{port}\
                 [{mixin_id}]",
                num = n + 1,
                mixin_id = mixin.id,
                volume = volume.display_as_fraction(),
                extra_filters = extra_filters,
                port = mixin.zmq_port,
            ));
        }

        filter_complex.push(format!(
            "[{orig_id}][{mixin_ids}]amix=inputs={count}:duration=longest[out]",
            orig_id = self.id,
            mixin_ids = self
                .mixins
                .iter()
                .map(|m| m.id.to_string())
                .collect::<Vec<_>>()
                .join("]["),
            count = self.mixins.len() + 1,
        ));
        let _ = cmd
            .args(&["-filter_complex", &filter_complex.join(";")])
            .args(&["-map", "[out]"])
            .args(&["-max_muxing_queue_size", "50000000"]);

        let _ = match self.to_url.scheme() {
            "file"
                if Path::new(self.to_url.path()).extension()
                    == Some("flv".as_ref()) =>
            {
                cmd.args(&["-map", "0:v"])
                    .args(&["-c:a", "libfdk_aac", "-c:v", "copy", "-shortest"])
                    .arg(dvr::new_file_path(&self.to_url).await?)
            }

            "icecast" => cmd
                .args(&["-c:a", "libmp3lame", "-b:a", "64k"])
                .args(&["-f", "mp3", "-content_type", "audio/mpeg"])
                .arg(self.to_url.as_str()),

            "rtmp" | "rtmps" => cmd
                .args(&["-map", "0:v"])
                .args(&["-c:a", "libfdk_aac", "-c:v", "copy", "-shortest"])
                .args(&["-f", "flv"])
                .arg(self.to_url.as_str()),

            "srt" => cmd
                .args(&["-map", "0:v"])
                .args(&["-c:a", "libfdk_aac", "-c:v", "copy", "-shortest"])
                .args(&["-strict", "-2", "-y", "-f", "mpegts"])
                .arg(self.to_url.as_str()),

            _ => unimplemented!(),
        };
        Ok(())
    }

    /// Runs the given [FFmpeg] [`Command`] by feeding to its STDIN the captured
    /// [`Mixin`] (if required), and awaits its completion.
    ///
    /// # Errors
    ///
    /// This method doesn't return [`Ok`] as the running [FFmpeg] [`Command`] is
    /// aborted by dropping and is intended to never stop. If it returns, than
    /// an [`io::Error`] occurs and the [FFmpeg] [`Command`] cannot run.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    /// [TeamSpeak]: https://teamspeak.com
    async fn run_ffmpeg(&self, mut cmd: Command) -> io::Result<()> {
        if let Some(m) = self.mixins.iter().find_map(|m| m.stdin.as_ref()) {
            let process = cmd.spawn()?;

            let ffmpeg_stdin = &mut process.stdin.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "FFmpeg's STDIN hasn't been captured",
                )
            })?;

            let mut src = m.lock().await;
            let _ = io::copy(&mut *src, ffmpeg_stdin).await.map_err(|e| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    format!("Failed to write into FFmpeg's STDIN: {}", e),
                )
            })?;

            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "FFmpeg re-streamer stopped unexpectedly",
            ))
        } else {
            RestreamerKind::run_ffmpeg_no_stdin(cmd).await
        }
    }
}

/// Additional live stream for mixing in a [`MixingRestreamer`].
#[derive(Clone, Debug)]
pub struct Mixin {
    /// ID of a [`state::Mixin`] represented by this [`Mixin`].
    pub id: MixinId,

    /// [`Url`] to pull an additional live stream from for mixing.
    pub url: MixinSrcUrl,

    /// [`Delay`] to mix this [`Mixin`]'s live stream with.
    pub delay: Delay,

    /// [`Volume`] rate to mix an audio of this [`Mixin`]'s live stream with.
    pub volume: Volume,

    /// [ZeroMQ] port of a spawned [FFmpeg] process listening to a real-time
    /// filter updates of this [`Mixin`]'s live stream during mixing process.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    /// [ZeroMQ]: https://zeromq.org
    pub zmq_port: u16,

    /// Actual live audio stream captured from the [TeamSpeak] server.
    ///
    /// If present, it should be fed into [FFmpeg]'s STDIN.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    /// [TeamSpeak]: https://teamspeak.com
    stdin: Option<Arc<Mutex<teamspeak::Input>>>,
}

impl Mixin {
    /// Creates a new [`Mixin`] out of the given [`state::Mixin`].
    ///
    /// `prev` value may be specified to consume already initialized resources,
    /// which are unwanted to be re-created.
    ///
    /// Optional `label` may be used to identify this [`Mixin`] in a [TeamSpeak]
    /// channel.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    #[allow(clippy::non_ascii_literal)]
    #[must_use]
    pub fn new(
        state: &state::Mixin,
        label: Option<&state::Label>,
        prev: Option<&Mixin>,
    ) -> Self {
        let stdin = (state.src.scheme() == "ts")
            .then(|| {
                prev.and_then(|m| m.stdin.clone()).or_else(|| {
                    let mut host = Cow::Borrowed(state.src.host_str()?);
                    if let Some(port) = state.src.port() {
                        host = Cow::Owned(format!("{}:{}", host, port));
                    }

                    let channel = state.src.path().trim_start_matches('/');

                    let name = state
                        .src
                        .query_pairs()
                        .find_map(|(k, v)| {
                            (k == "name").then(|| v.into_owned())
                        })
                        .or_else(|| label.map(|l| format!("🤖 {}", l)))
                        .unwrap_or_else(|| format!("🤖 {}", state.id));

                    Some(Arc::new(Mutex::new(teamspeak::Input::new(
                        teamspeak::Connection::build(host.into_owned())
                            .channel(channel.to_owned())
                            .name(name),
                    ))))
                })
            })
            .flatten();

        Self {
            id: state.id,
            url: state.src.clone(),
            delay: state.delay,
            volume: state.volume,
            zmq_port: new_unique_zmq_port(),
            stdin,
        }
    }

    /// Checks whether this [`Mixin`]'s [FFmpeg] process must be restarted, as
    /// cannot apply the new `actual` params on itself correctly, without
    /// interruptions.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[inline]
    #[must_use]
    pub fn needs_restart(&self, actual: &Self) -> bool {
        self.url != actual.url || self.delay != actual.delay
    }
}

/// Generates a new port for a [ZeroMQ] listener, which is highly unlikely to be
/// used already.
///
/// [ZeroMQ]: https://zeromq.org
#[must_use]
fn new_unique_zmq_port() -> u16 {
    use std::{
        convert,
        sync::atomic::{AtomicU16, Ordering},
    };

    static LATEST_PORT: AtomicU16 = AtomicU16::new(20000);

    LATEST_PORT
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |p| {
            Some(p.checked_add(1).unwrap_or(20000))
        })
        .unwrap_or_else(convert::identity)
}

/// Tunes [`Volume`] of the specified [FFmpeg] `track` by updating the `volume`
/// [FFmpeg] filter in real-time via [ZeroMQ] protocol.
///
/// [FFmpeg]: https://ffmpeg.org
/// [ZeroMQ]: https://zeromq.org
fn tune_volume(track: Uuid, port: u16, volume: Volume) {
    use zeromq::{BlockingRecv as _, BlockingSend as _, Socket as _};

    drop(tokio::spawn(
        AssertUnwindSafe(async move {
            let addr = format!("tcp://127.0.0.1:{}", port);

            let mut socket = zeromq::ReqSocket::new();
            socket.connect(&addr).await.map_err(|e| {
                log::error!(
                    "Failed to establish ZeroMQ connection with {} : {}",
                    addr,
                    e,
                );
            })?;

            socket
                .send(
                    format!(
                        "volume@{} volume {}",
                        track,
                        volume.display_as_fraction(),
                    )
                    .into(),
                )
                .await
                .map_err(|e| {
                    log::error!(
                        "Failed to send ZeroMQ message to {} : {}",
                        addr,
                        e,
                    );
                })?;

            let resp = socket.recv().await.map_err(|e| {
                log::error!(
                    "Failed to receive ZeroMQ response from {} : {}",
                    addr,
                    e,
                );
            })?;

            if resp.data.as_ref() != "0 Success".as_bytes() {
                log::error!(
                    "Received invalid ZeroMQ response from {} : {}",
                    addr,
                    std::str::from_utf8(&*resp.data).map_or_else(
                        |_| Cow::Owned(format!("{:?}", &*resp.data)),
                        Cow::Borrowed,
                    ),
                );
            }

            <Result<_, ()>>::Ok(())
        })
        .catch_unwind()
        .map_err(|p| {
            log::crit!(
                "Panicked while sending ZeroMQ message: {}",
                display_panic(&p),
            );
        }),
    ));
}
