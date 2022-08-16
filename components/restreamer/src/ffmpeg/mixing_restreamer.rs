//! Kind of a [FFmpeg] re-streaming process that mixes a live stream from one
//! URL endpoint with some additional live streams and re-streams the result to
//! another endpoint.
//!
//! [FFmpeg]: https://ffmpeg.org

use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::Write as _,
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
};

use ephyr_log::{log, Drain as _};
use futures::{FutureExt as _, TryFutureExt as _};
use interprocess::os::unix::fifo_file::create_fifo;
use tokio::{io, process::Command, sync::watch, sync::Mutex};
use url::Url;
use uuid::Uuid;

use crate::{
    display_panic, dvr,
    ffmpeg::util::{
        kill_ffmpeg_process_by_sigterm, wraps_ffmpeg_process_output_with_result,
    },
    ffmpeg::RestreamerKind,
    state::{self, Delay, MixinId, MixinSrcUrl, State, Volume},
    teamspeak,
};
use tokio::fs::File;
use tsclientlib::Identity;

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
            orig_volume: output.volume.clone(),
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
            self.orig_volume = actual.orig_volume.clone();
            tune_volume(self.id, self.orig_zmq_port, self.orig_volume.clone());
        }
        for (curr, actual) in self.mixins.iter_mut().zip(actual.mixins.iter()) {
            if curr.volume != actual.volume {
                curr.volume = actual.volume.clone();
                tune_volume(curr.id.into(), curr.zmq_port, curr.volume.clone());
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
    pub(crate) async fn setup_ffmpeg(
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

        let orig_volume = output
            .as_ref()
            .map_or(self.orig_volume.clone(), |o| o.volume.clone());

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
                        .arg("-i")
                        .arg(mixin.get_fifo_path())
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
                let _ = write!(
                    extra_filters,
                    "adelay=delays={}:all=1,",
                    mixin.delay.as_millis()
                );
            }

            let volume = output
                .as_ref()
                .and_then(|o| {
                    o.mixins.iter().find_map(|m| {
                        (m.id == mixin.id).then(|| m.volume.clone())
                    })
                })
                .unwrap_or_else(|| mixin.volume.clone());

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

        let mut orig_id = self.id.to_string();
        let mut mixin_ids = self
            .mixins
            .iter()
            .map(|m| m.id.to_string())
            .collect::<Vec<_>>();

        // Activate `sidechain` filter if required
        if let Some(sidechain_mixin) = self.mixins.iter().find(|m| m.sidechain)
        {
            let sidechain_mixin_id = sidechain_mixin.id.to_string();
            // Sidechain is mixing Origin Audio and selected Mixin Audio
            filter_complex.push(format!(
                "[{sidechain_mixin_id}]asplit=2[sc][mix];\
                 [{orig_id}][sc]sidechaincompress=\
                                    level_in=2\
                                    :threshold=0.01\
                                    :ratio=10\
                                    :attack=10\
                                    :release=1500[compr]"
            ));
            // Replace Mixin Id for sidechain with `mix` value
            if let Some(elem) =
                mixin_ids.iter_mut().find(|x| **x == sidechain_mixin_id)
            {
                "mix".clone_into(elem);
            };

            // Replace Origin Audio Id with side-chained version
            orig_id = "compr".to_string();
        };

        filter_complex.push(format!(
            "[{orig_id}][{mixin_ids}]amix=inputs={count}:duration=longest[out]",
            orig_id = orig_id,
            mixin_ids = mixin_ids.join("]["),
            count = self.mixins.len() + 1,
        ));

        log::debug!("FFmpeg FILTER COMPLEX: {:?}", &filter_complex.join(";"));
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
        log::debug!("FFmpeg CMD: {:?}", &cmd);
        Ok(())
    }

    /// Runs the given [FFmpeg] [`Command`] by providing input FIFO files and
    /// feeding [`Mixin`]s into them (if required), and awaits its completion.
    ///
    /// The FIFO files are created before starting [`Command`] and separate
    /// tasks are created for each file to feed the [`Mixin`] data into them.
    /// After the [`Command`] finishes FIFO files are deleted.
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
    /// [TeamSpeak]: https://teamspeak.com
    pub(crate) async fn run_ffmpeg_with_mixins(
        &self,
        mut cmd: Command,
        mut kill_rx: watch::Receiver<i32>,
    ) -> io::Result<()> {
        // FIFO should be exists before start of FFmpeg process
        self.create_mixins_fifo()?;
        // FFmpeg should start reading FIFO before writing started
        let process = cmd.spawn()?;

        let exit_signal = *kill_rx.borrow_and_update();
        if exit_signal != 0 {
            return Ok(());
        }

        let kill_task = kill_ffmpeg_process_by_sigterm(process.id(), kill_rx);
        self.start_fed_mixins_fifo();
        // Wait for process to finish or get killed
        let out = process.wait_with_output().await?;
        kill_task.abort();
        self.remove_mixins_fifo();

        wraps_ffmpeg_process_output_with_result(&out)
    }

    /// Creates [FIFO] files for [`Mixin`]s.
    ///
    /// # Errors
    ///
    /// If [FIFI] file failed to create.
    /// We need it because [FFmpeg] cannot start if no [FIFO] file.
    ///
    /// [FIFO]: https://www.unix.com/man-page/linux/7/fifo/
    fn create_mixins_fifo(&self) -> io::Result<()> {
        for m in &self.mixins {
            if !m.get_fifo_path().exists() {
                create_fifo(m.get_fifo_path(), 0o777)?;
            }
        }
        Ok(())
    }

    /// Remove [FIFO] files for [`Mixin`]s.
    ///
    /// We don't really care if file was really deleted so no error.
    ///
    /// [FIFO]: https://www.unix.com/man-page/linux/7/fifo/
    fn remove_mixins_fifo(&self) {
        for m in &self.mixins {
            if m.get_fifo_path().exists() {
                let _ = std::fs::remove_file(m.get_fifo_path())
                    .map_err(|e| log::error!("Failed to remove FIFO: {}", e));
            }
        }
    }

    /// Copy data from [`Mixin.stdin`] to [FIFO].
    /// Each data copying is operated in separate thread.
    ///
    /// [FIFO]: https://www.unix.com/man-page/linux/7/fifo/
    fn start_fed_mixins_fifo(&self) {
        async fn run_copy(
            input: Arc<Mutex<teamspeak::Input>>,
            fifo_path: PathBuf,
        ) -> io::Result<()> {
            let mut src = input.lock().await;
            log::debug!("Connecting to FIFO: {:?}", &fifo_path);
            let mut file = File::create(&fifo_path).await?;

            let _ = io::copy(&mut *src, &mut file).await.map_err(|e| {
                log::error!("Failed to write into FIFO: {}", e);
            });
            Ok(())
        }

        for m in &self.mixins {
            if let Some(i) = m.stdin.as_ref() {
                drop(tokio::spawn(run_copy(Arc::clone(i), m.get_fifo_path())));
            }
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

    /// Apply [sidechain] audio filter of this [`Mixin`]'s with live stream.
    ///
    /// [sidechain]: https://ffmpeg.org/ffmpeg-filters.html#sidechaincompress
    pub sidechain: bool,

    /// [ZeroMQ] port of a spawned [FFmpeg] process listening to a real-time
    /// filter updates of this [`Mixin`]'s live stream during mixing process.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    /// [ZeroMQ]: https://zeromq.org
    pub zmq_port: u16,

    /// Actual live audio stream captured from the [TeamSpeak] server.
    ///
    /// If present, it should be fed into [FIFO].
    ///
    /// [TeamSpeak]: https://teamspeak.com
    /// [FIFO]: https://www.unix.com/man-page/linux/7/fifo/
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

                    let query: HashMap<String, String> =
                        state.src.query_pairs().into_owned().collect();
                    let name = query
                        .get("name")
                        .cloned()
                        .or_else(|| label.map(|l| format!("🤖 {}", l)))
                        .unwrap_or_else(|| format!("🤖 {}", state.id));
                    let identity = query.get("identity").map_or_else(
                        Identity::create,
                        |v| {
                            Identity::new_from_str(v).unwrap_or_else(|e| {
                                log::error!(
                                    "Failed to create identity `{}`\
                                    \n\t with error: {}",
                                    &v,
                                    &e
                                );
                                Identity::create()
                            })
                        },
                    );

                    Some(Arc::new(Mutex::new(teamspeak::Input::new(
                        teamspeak::Connection::build(host.into_owned())
                            .channel(channel.to_owned())
                            .name(name)
                            .identity(identity),
                    ))))
                })
            })
            .flatten();

        Self {
            id: state.id,
            url: state.src.clone(),
            delay: state.delay,
            sidechain: state.sidechain,
            volume: state.volume.clone(),
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
        self.url != actual.url
            || self.delay != actual.delay
            || self.sidechain != actual.sidechain
    }

    /// [FIFO] path where stream captures from the [TeamSpeak] server.
    ///
    /// Should be fed into [FFmpeg]'s as file input.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    /// [TeamSpeak]: https://teamspeak.com
    /// [FIFO]: https://www.unix.com/man-page/linux/7/fifo/
    #[inline]
    #[must_use]
    pub fn get_fifo_path(&self) -> PathBuf {
        std::env::temp_dir().join(format!("ephyr_mixin_{}.pipe", self.id))
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
    use zeromq::{Socket as _, SocketRecv as _, SocketSend as _};

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

            let data = resp.into_vec().pop().unwrap();
            if data.as_ref() != "0 Success".as_bytes() {
                log::error!(
                    "Received invalid ZeroMQ response from {} : {}",
                    addr,
                    std::str::from_utf8(&data).map_or_else(
                        |_| Cow::Owned(format!("{:?}", &data)),
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
