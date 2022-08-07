//! [TeamSpeak] audio capture definitions.
//!
//! [TeamSpeak]: https://teamspeak.com

use std::{
    collections::HashMap,
    fmt,
    fs::File,
    future::Future,
    io::Write,
    mem::ManuallyDrop,
    path::PathBuf,
    pin::Pin,
    str,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    task::{Context, Poll},
    time::Duration,
};
use unix_named_pipe;

use backoff::{future::retry_notify, ExponentialBackoff};
use byteorder::{BigEndian, ByteOrder as _};
use derive_more::{Display, Error};
use ephyr_log::log;
use futures::{
    future, ready, sink, FutureExt as _, Stream, StreamExt as _,
    TryFutureExt as _,
};
use once_cell::sync::Lazy;
use rand::Rng as _;
use tokio::{
    io::{self, AsyncRead},
    task::JoinHandle,
    time,
};
use tsclientlib::{DisconnectOptions, StreamItem};
use tsproto_packets::packets::AudioData;

pub use tsclientlib::{ConnectOptions as Config, Connection};

/// Handler responsible for decoding, tracking and mixing audio of all
/// [TeamSpeak] channel members.
///
/// [TeamSpeak]: https://teamspeak.com
pub type AudioHandler = tsclientlib::audio::AudioHandler<MemberId>;

/// Type of [TeamSpeak] channel member ID.
///
/// [TeamSpeak]: https://teamspeak.com
type MemberId = u16;

/// Audio input captured from [TeamSpeak] server.
///
/// It produces [PCM 32-bit floating-point big-endian][1] encoded
/// [`Input::CHANNELS`]-stereo audio samples (`f32be` format in [FFmpeg]'s
/// [notation][2]) with a constant [`Input::SAMPLE_RATE`].
///
/// [FFmpeg]: https://ffmpeg.org
/// [TeamSpeak]: https://teamspeak.com
/// [1]: https://wiki.multimedia.cx/index.php/PCM
/// [2]: https://trac.ffmpeg.org/wiki/audio%20types
pub struct Input {
    /// [`Config`] for establishing new [`Connection`] with.
    cfg: Config,

    /// Ticker that fires each [`Input::FREQUENCY_MILLIS`] and is used
    /// to determine when samples should be emitted.
    ticker: time::Interval,

    /// Audio frame (samples sequence of [`Input::FRAME_SIZE`]) being emitted
    /// on each [`Input::ticker`] tick.
    frame: Vec<f32>,

    /// Cursor indicating the position in [`Input::frame`] to start reading it
    /// from.
    cursor: usize,

    /// Handler responsible for decoding, tracking and mixing audio of all
    /// [TeamSpeak] channel members, for this [`Input`].
    ///
    /// [TeamSpeak]: https://teamspeak.com
    audio: Arc<Mutex<AudioHandler>>,

    /// Abort handle and waiter of the spawned [`AudioCapture`], which receives
    /// audio packets from [TeamSpeak] server and feeds them into the
    /// [`Input::audio`] handler.
    ///
    /// Abort handle is responsible for aborting [`AudioCapture`] execution.
    ///
    /// Waiter is responsible for awaiting [`AudioCapture`] to complete all its
    /// operations.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    conn: Option<(future::AbortHandle, JoinHandle<()>)>,

    /// Indicator whether the spawned [`AudioCapture`] is unable to recover from
    /// its last error, and so this [`Input`] should return an error too.
    is_conn_unrecoverable: Arc<AtomicBool>,

    pipe_file: PathBuf,
    ref_to_write_pipe: Option<File>,
}

impl Input {
    /// Sample rate that [`Input`] emits audio samples with.
    pub const SAMPLE_RATE: usize = 48000;

    /// Number of channels in stereo audion produced by [`Input`].
    pub const CHANNELS: usize = 2;

    /// Frequency (in milliseconds) that [`Input`] emits audio samples with.
    pub const FREQUENCY_MILLIS: usize = 20;

    /// Size (in samples) of a single frame emitted by [`Input`] each
    /// [`Input::FREQUENCY_MILLIS`].
    pub const FRAME_SIZE: usize =
        Self::SAMPLE_RATE / 1000 * Self::FREQUENCY_MILLIS * Self::CHANNELS;

    /// Creates a new [`Input`] with the provided [`Config`].
    #[must_use]
    pub fn new<C: Into<Config>>(cfg: C, pipe_file: PathBuf) -> Self {
        let cfg = {
            use ephyr_log::Drain as _;

            let lgr = ephyr_log::logger();
            let is_debug = lgr.is_debug_enabled();
            let is_trace = lgr.is_trace_enabled();

            // TODO #6: Memoize TeamSpeak Identity and reuse.
            //      https://github.com/ALLATRA-IT/ephyr/issues/6
            let mut cfg = cfg
                .into()
                .logger(lgr)
                .log_commands(is_debug)
                .log_packets(is_trace);
            // TeamSpeak limits client names by 30 UTF-8 characters max. If the
            // provided name is longer, then we should truncate it to fit into
            // the requirement.
            if cfg.get_name().chars().count() > 30 {
                let n = cfg.get_name().chars().take(30).collect::<String>();
                cfg = cfg.name(n);
            }
            cfg
        };

        let lgr = ephyr_log::logger();
        Self {
            cfg,
            pipe_file,
            ref_to_write_pipe: None,
            ticker: time::interval(Duration::from_millis(
                Self::FREQUENCY_MILLIS as u64,
            )),
            frame: vec![0.0; Self::FRAME_SIZE],
            cursor: 0,
            audio: Arc::new(Mutex::new(AudioHandler::new(lgr))),
            conn: None,
            is_conn_unrecoverable: Arc::new(AtomicBool::default()),
        }
    }

    /// Spawns an [`AudioCapture`] associated with this [`Input`], retrying it
    /// endlessly with an [`ExponentialBackoff`] if it fails in a recoverable
    /// way.
    fn spawn_audio_capturing(&mut self) {
        let cfg = self.cfg.clone();
        let audio = self.audio.clone();
        let is_conn_unrecoverable = self.is_conn_unrecoverable.clone();

        let capturing = retry_notify(
            ExponentialBackoff {
                max_elapsed_time: None,
                ..ExponentialBackoff::default()
            },
            move || {
                AudioCapture::run(cfg.clone(), audio.clone())
                    .map_err(AudioCaptureError::into_backoff)
            },
            |err, dur| {
                log::error!(
                    "Backoff TeamSpeak server audio capturing for {} due to \
                     error: {}",
                    humantime::format_duration(dur),
                    err,
                );
            },
        )
        .map_err(move |e| {
            log::error!("Cannot capture audio from TeamSpeak server: {}", e);
            is_conn_unrecoverable.store(true, Ordering::SeqCst);
        });

        let (abort, on_abort) = future::AbortHandle::new_pair();
        let waiter = tokio::spawn(
            future::Abortable::new(capturing, on_abort).map(|_| ()),
        );

        self.conn = Some((abort, waiter));
    }
}

impl AsyncRead for Input {
    /// Emits audio frame of [`Input::FRAME_SIZE`] each
    /// [`Input::FREQUENCY_MILLIS`]. The frame contains mixed audio of all
    /// [TeamSpeak] channel members talking at the moment. If there is no
    /// talking members, the just a silence is emitted.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if self.conn.is_none() {
            self.spawn_audio_capturing();
            self.ref_to_write_pipe = Some(
                unix_named_pipe::open_write(&self.pipe_file)
                    .expect("could not create fifo"),
            );
        }
        if self.is_conn_unrecoverable.load(Ordering::SeqCst) {
            return Poll::Ready(Err(InputError::NoData.into()));
        }

        if self.cursor >= self.frame.len() {
            // `time::Interval` stream never returns `None`, so we can omit
            // checking it to be finished.
            let _ = ready!(Pin::new(&mut self.ticker).poll_next(cx));

            self.cursor = 0;
            self.frame.fill(0.0);
            drop(
                self.audio
                    .clone()
                    .lock()
                    .unwrap()
                    .fill_buffer(&mut self.frame),
            );
        }

        let cursor = self.cursor;

        // Detect how much samples we can mix and write into `dst`.
        let src_size = self.frame.len() - cursor;

        // `f32` takes 4 bytes in big endian, so we should fit in there.
        let dst_size = buf.len() / 4;
        if dst_size == 0 {
            return Poll::Ready(Err(InputError::TooSmallBuffer.into()));
        }

        let size = src_size.min(dst_size);
        let size_in_bytes = size * 4;

        BigEndian::write_f32_into(
            &self.frame[cursor..(cursor + size)],
            &mut buf[..size_in_bytes],
        );

        if let Some(ref mut pipe_to_write) = self.ref_to_write_pipe {
            let _ = pipe_to_write
                .write(&buf[..size_in_bytes])
                .expect("Failed write to the pipe.");
        }
        self.cursor += size;

        Poll::Ready(Ok(size_in_bytes))
    }
}

impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Input")
            .field("cfg", &self.cfg)
            .field("ticker", &self.ticker)
            .field("frame", &self.frame)
            .field("cursor", &self.cursor)
            .field("audio", &"Arc<Mutex<AudioHandler>>")
            .field("conn", &self.conn)
            .field("is_conn_unrecoverable", &self.is_conn_unrecoverable)
            .finish()
    }
}

impl Drop for Input {
    /// Spawns the [`Input::conn`] waiter to be fully awaited for ensuring
    /// normal disconnecting from [TeamSpeak] server.
    ///
    /// This is required, because disconnecting from [TeamSpeak] server by
    /// [`AudioCapture`] implies some handshake, and so awaited for completion.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    #[inline]
    fn drop(&mut self) {
        if let Some((conn, waiter)) = self.conn.take() {
            conn.abort();
            spawn_waiter(waiter);
        }
    }
}

/// Possible errors of reading [`Input`].
#[derive(Debug, Display, Error)]
pub enum InputError {
    /// No data can be received from [TeamSpeak] server.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    #[display(fmt = "Unable to receive data from TeamSpeak server")]
    NoData,

    /// Input buffer provided to read [`Input`] is too small to read any data.
    #[display(fmt = "Input buffer is too small")]
    TooSmallBuffer,
}

impl From<InputError> for io::Error {
    fn from(e: InputError) -> Self {
        use InputError as E;

        let kind = match e {
            E::NoData => io::ErrorKind::NotConnected,
            E::TooSmallBuffer => io::ErrorKind::InvalidData,
        };
        io::Error::new(kind, e)
    }
}

/// Listener of [TeamSpeak] channel, which captures audio packets of each
/// talking channel member and feeds them into an [`AudioHandler`] to be mixed.
///
/// [TeamSpeak]: https://teamspeak.com
#[allow(missing_debug_implementations)]
pub struct AudioCapture {
    /// Established [`Connection`] with [TeamSpeak] server.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    conn: ManuallyDrop<Connection>,

    /// Handler of audio packets received from [TeamSpeak] server.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    audio: Arc<Mutex<AudioHandler>>,
}

impl AudioCapture {
    /// Creates new [`AudioCapture`] from the given [`Connection`] and for
    /// the given [`AudioHandler`].
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(conn: Connection, audio: Arc<Mutex<AudioHandler>>) -> Self {
        audio.lock().unwrap().reset();
        Self {
            conn: ManuallyDrop::new(conn),
            audio,
        }
    }

    /// Generates a new random HWID (hardware identification string).
    ///
    /// # Panics
    ///
    /// No panics, because we guarantee to pass proper range to
    /// the [`hex::encode_to_slice`].
    #[must_use]
    pub fn new_hwid() -> String {
        const BYTES: usize = 16;
        const HEX_BYTES: usize = 2 * BYTES;

        let mut rng = rand::thread_rng();

        let mut first = [0_u8; HEX_BYTES];
        hex::encode_to_slice(&rng.gen::<[u8; BYTES]>(), &mut first).unwrap();

        let mut second = [0_u8; HEX_BYTES];
        hex::encode_to_slice(&rng.gen::<[u8; BYTES]>(), &mut second).unwrap();

        // This is totally safe, because hex-encoded data is guaranteed to be
        // a valid UTF-8 string.
        #[allow(unsafe_code)]
        unsafe {
            format!(
                "{},{}",
                str::from_utf8_unchecked(&first),
                str::from_utf8_unchecked(&second),
            )
        }
    }

    /// Creates a new [`AudioCapture`] using the given [`Config`] for the given
    /// [`AudioHandler`] and awaits its completion.
    ///
    /// Generates new HWID (hardware identification string) to uniquely
    /// distinguish this [`AudioCapture`] for [TeamSpeak] server.
    ///
    /// # Errors
    ///
    /// - If receiving audio from [TeamSpeak] server fails;
    /// - If processing received audio packets with [`AudioHandler`] fails.
    ///
    /// See [`AudioCaptureError`] for details.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    pub async fn run(
        cfg: Config,
        audio: Arc<Mutex<AudioHandler>>,
    ) -> Result<(), AudioCaptureError> {
        log::debug!(
            "Connecting to TeamSpeak server: {:?}/{:?}",
            cfg.get_address(),
            cfg.get_channel()
        );
        let conn = cfg
            .hardware_id(Self::new_hwid())
            .connect()
            .map_err(AudioCaptureError::InitializationFailed)?;
        AudioCapture::new(conn, audio).await
    }
}

impl Future for AudioCapture {
    type Output = Result<(), AudioCaptureError>;

    /// Processes [`AudioCapture::conn`] lifecycle and feeds all received audio
    /// packets into the [`AudioHandler`].
    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        use tsclientlib::audio::Error as AE;
        use AudioCaptureError as E;

        loop {
            let audio_packet =
                match ready!(Pin::new(&mut self.conn.events()).poll_next(cx))
                    .ok_or_else(|| E::UnexpectedFinish)?
                    .map_err(E::ConnectionFailed)?
                {
                    StreamItem::Audio(packet) => packet,
                    _ => continue,
                };

            let member_id = match audio_packet.data().data() {
                AudioData::S2C { from, .. }
                | AudioData::S2CWhisper { from, .. } => *from,
                _ => return Poll::Ready(Err(E::UnexpectedC2sPacket)),
            };

            if let Err(e) = self
                .audio
                .lock()
                .unwrap()
                .handle_packet(member_id, audio_packet)
            {
                if !matches!(
                    e,
                    AE::QueueFull | AE::TooLate { .. } | AE::Duplicate(_),
                ) {
                    return Poll::Ready(Err(E::DecodingFailed(e)));
                }
                log::warn!("Drop audio packet from TeamSpeak server: {}", e);
            }
        }
    }
}

impl Drop for AudioCapture {
    /// Spawns the [`AudioCapture::conn`] waiter to be fully drained, so
    /// disconnecting from [TeamSpeak] server normally.
    ///
    /// This is required, because disconnecting from [TeamSpeak] server implies
    /// some handshake.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    #[inline]
    fn drop(&mut self) {
        // This is totally safe, because `self.conn` field is guaranteed to be
        // never used again later, so `ManuallyDrop` won't be touched again.
        #[allow(unsafe_code)]
        spawn_disconnect(unsafe { ManuallyDrop::take(&mut self.conn) });
    }
}

/// Possible errors of capturing audio from [TeamSpeak] server.
///
/// [TeamSpeak]: https://teamspeak.com
#[derive(Debug, Display, Error)]
pub enum AudioCaptureError {
    /// Initializing [`Connection`] with [TeamSpeak] server failed.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    #[display(
        fmt = "Initializing connection with TeamSpeak server failed: {}",
        _0
    )]
    InitializationFailed(tsclientlib::Error),

    /// Establishing connection with [TeamSpeak] server failed.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    #[display(fmt = "Connecting to TeamSpeak server failed: {}", _0)]
    ConnectionFailed(tsclientlib::Error),

    /// Receiving packets from [TeamSpeak] server finished unexpectedly.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    #[display(
        fmt = "Receiving packets from TeamSpeak server finished unexpectedly"
    )]
    UnexpectedFinish,

    /// Received from [TeamSpeak] server C2S (client-to-server) audio packet,
    /// while only S2C (server-to-client) audio packets are allowed.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    #[display(
        fmt = "Received C2S audio packet, while only S2C packets are allowed"
    )]
    UnexpectedC2sPacket,

    /// Failed to decode audio packet received from [TeamSpeak] server.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    #[display(fmt = "Failed to decode audio packet: {}", _0)]
    DecodingFailed(tsclientlib::audio::Error),
}

impl AudioCaptureError {
    /// Wraps this [`AudioCaptureError`] into a [`backoff::Error`] carefully
    /// distinguishing transient and permanent errors.
    #[must_use]
    pub fn into_backoff(self) -> backoff::Error<Self> {
        use tsclientlib::audio::Error as E;

        let is_permanent = match &self {
            Self::InitializationFailed(_) => true,
            Self::ConnectionFailed(_)
            | Self::UnexpectedFinish
            | Self::UnexpectedC2sPacket => false,
            Self::DecodingFailed(err) => {
                matches!(err, E::CreateDecoder(_) | E::UnsupportedCodec(_))
            }
        };
        if is_permanent {
            backoff::Error::Permanent(self)
        } else {
            backoff::Error::Transient(self)
        }
    }
}

/// Collection of [`JoinHandle`]s being awaited for completion at the moment.
///
/// See [`finish_all_disconnects`]'s documentation for details.
#[allow(clippy::type_complexity)]
static IN_PROGRESS_DISCONNECTS: Lazy<Arc<Mutex<HashMap<u64, JoinHandle<()>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Registers the given [`JoinHandle`] and tracks its completion via
/// [`IN_PROGRESS_DISCONNECTS`].
///
/// All disconnects can be awaited for completion via
/// [`finish_all_disconnects()`].
#[allow(clippy::missing_panics_doc)]
fn spawn_waiter(waiter: JoinHandle<()>) {
    let mut disconnects = IN_PROGRESS_DISCONNECTS.lock().unwrap();

    let id = loop {
        let id = rand::thread_rng().gen::<u64>();
        if !disconnects.contains_key(&id) {
            break id;
        }
    };

    drop(disconnects.insert(id, waiter));
}

/// [`tokio::spawn`]s disconnection of the given [`Connection`] and tracks its
/// completion via [`IN_PROGRESS_DISCONNECTS`].
///
/// All disconnects can be awaited for completion via
/// [`finish_all_disconnects()`].
fn spawn_disconnect(mut conn: Connection) {
    let mut disconnects = IN_PROGRESS_DISCONNECTS.lock().unwrap();

    let id = loop {
        let id = rand::thread_rng().gen::<u64>();
        if !disconnects.contains_key(&id) {
            break id;
        }
    };

    drop(
        disconnects.insert(
            id,
            tokio::spawn(
                async move {
                    // First, we should check whether `Connection` is
                    // established at all.
                    let _ = conn.get_state()?;

                    // Then initiate disconnection by sending an appropriate
                    // packet.
                    conn.disconnect(DisconnectOptions::default())?;
                    // And wait until it's done.
                    let _ = conn.events().map(Ok).forward(sink::drain()).await;

                    Ok(())
                }
                .map(
                    move |_: Result<_, tsclientlib::Error>| {
                        // Finally, we should remove this disconnect from the
                        // collection whenever it succeeds or errors. Otherwise,
                        // we could stuck on shutdown waiting eternally.
                        drop(
                            IN_PROGRESS_DISCONNECTS.lock().unwrap().remove(&id),
                        );
                    },
                ),
            ),
        ),
    );
}

/// Awaits for all disconnections from [TeamSpeak] servers happening at the
/// moment to be completed.
///
/// Call this function __before__ shutting down the [`tokio::runtime`],
/// otherwise disconnects won't proceed normally.
///
/// This is required due to [`tokio::runtime`] [doesn't wait][1] all
/// [`tokio::spawn`]ed tasks to be fully processed when shutting down.
///
/// [TeamSpeak]: https://teamspeak.com
/// [1]: https://github.com/tokio-rs/tokio/issues/2053
#[allow(clippy::missing_panics_doc)]
pub async fn finish_all_disconnects() {
    let disconnects = {
        IN_PROGRESS_DISCONNECTS
            .lock()
            .unwrap()
            .drain()
            .map(|(_, hndl)| hndl)
            .collect::<Vec<_>>()
    };

    drop(future::join_all(disconnects).await);
}
