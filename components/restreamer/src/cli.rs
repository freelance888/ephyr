//! CLI (command line interface).

use std::{fmt, net::IpAddr, path::PathBuf, str::FromStr as _};

use anyhow::anyhow;
use ephyr_log::slog;
use structopt::StructOpt;

/// CLI (command line interface) of the re-streamer server.
#[derive(Clone, Debug, StructOpt)]
#[structopt(about = "RTMP re-streamer server")]
pub struct Opts {
    /// Debug mode of the server.
    #[structopt(short, long, help = "Enables debug mode")]
    pub debug: bool,

    /// IP address for the server to listen client HTTP requests on.
    #[structopt(
        long,
        env = "EPHYR_RESTREAMER_CLIENT_HTTP_IP",
        default_value = "0.0.0.0",
        help = "IP to listen client HTTP on",
        long_help = "IP address for the server to listen client HTTP requests \
                     on"
    )]
    pub client_http_ip: IpAddr,

    /// Port for the server to listen client HTTP requests on.
    #[structopt(
        long,
        env = "EPHYR_RESTREAMER_CLIENT_HTTP_PORT",
        default_value = "80",
        help = "Port to listen client HTTP on",
        long_help = "Port for the server to listen client HTTP requests on"
    )]
    pub client_http_port: u16,

    /// IP address for the server to listen RTMP callback HTTP requests on.
    #[structopt(
        long,
        env = "EPHYR_RESTREAMER_CALLBACK_HTTP_IP",
        default_value = "127.0.0.1",
        help = "IP to listen callback HTTP on",
        long_help = "IP address for the server to listen RTMP callback HTTP \
                     requests on"
    )]
    pub callback_http_ip: IpAddr,

    /// Port for the server to listen RTMP callback HTTP requests on.
    #[structopt(
        long,
        env = "EPHYR_RESTREAMER_CALLBACK_HTTP_PORT",
        default_value = "8081",
        help = "Port to listen callback HTTP on",
        long_help = "Port for the server to listen RTMP callback HTTP requests \
                     on"
    )]
    pub callback_http_port: u16,

    /// Path to a file to persist the server's state in.
    #[structopt(
        short,
        long,
        env = "EPHYR_RESTREAMER_STATE_PATH",
        default_value = "state.json",
        help = "Path to a file to persist state in",
        long_help = "Path to a file to persist the server's state in"
    )]
    pub state_path: PathBuf,

    /// Path to [SRS] installation directory.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    #[structopt(
        long,
        env = "EPHYR_RESTREAMER_SRS_PATH",
        default_value = "/usr/local/srs",
        help = "Path to SRS dir",
        long_help = "Path to SRS installation directory"
    )]
    pub srs_path: PathBuf,

    /// Path to the directory where [SRS] serves public files from via HTTP
    /// (HLS chunks, etc).
    ///
    /// Relative path will use [`Opts::srs_path`] as its base path, not the
    /// current working directory.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    #[structopt(
        long,
        env = "EPHYR_RESTREAMER_SRS_HTTP_DIR",
        default_value = "/var/www/srs",
        help = "Path to SRS public HTTP files",
        long_help = "Path to the directory where SRS serves public files from \
                     via HTTP (HLS chunks, etc).\
                     \n\n\
                     Relative path will use --srs-path as its base path, not \
                     the current working directory."
    )]
    pub srs_http_dir: PathBuf,

    /// Path to [FFmpeg] binary.
    ///
    /// [FFmpeg]: https://ffmpeg.org
    #[structopt(
        short,
        long,
        env = "FFMPEG_PATH",
        default_value = "/usr/local/bin/ffmpeg",
        help = "Path to FFmpeg binary",
        long_help = "Path to FFmpeg binary"
    )]
    pub ffmpeg_path: PathBuf,

    /// Host to access the re-streamer server in public networks.
    ///
    /// If [`None`], then it will be auto-detected.
    #[structopt(
        long,
        env = "EPHYR_RESTREAMER_PUBLIC_HOST",
        help = "Public host to access the server",
        long_help = "Host to access the server in public networks \
                     (auto-detects by default)"
    )]
    pub public_host: Option<String>,

    /// Verbosity level of the server logs.
    #[structopt(
        short,
        long,
        parse(try_from_str = Self::parse_log_level),
        help = "Logs verbosity level: \
                OFF | CRIT | ERRO | WARN | INFO | DEBG | TRCE"
    )]
    pub verbose: Option<slog::Level>,

    /// Verbosity level of the server logs.
    #[structopt(
    long,
    env = "EPHYR_RESTREAMER_VIDEO_FILE_ROOT",
    default_value = "/tmp/ephyr",
    help = "Path where video files will be stored",
    long_help = "Here the video files that can be streamed to the output will be downloaded."
    )]
    pub file_root: PathBuf,
}

impl Opts {
    /// Parses CLI [`Opts`] from command line arguments.
    ///
    /// Prints the error message and quits the program in case of failure.
    #[inline]
    #[must_use]
    pub fn from_args() -> Self {
        <Self as StructOpt>::from_args()
    }

    /// Parses [`slog::Level`] from the given string.
    ///
    /// This function is required, because [`slog::Level`]'s [`FromStr`]
    /// implementation returns `()`, which is not [`Display`] as [`StructOpt`]
    /// requires.
    ///
    /// # Errors
    ///
    /// If [`slog::Level`] failed to parse from the string.
    ///
    /// [`Display`]: std::fmt::Display
    /// [`FromStr`]: std::str::FromStr
    pub fn parse_log_level(lvl: &str) -> Result<slog::Level, anyhow::Error> {
        #[allow(clippy::map_err_ignore)]
        slog::Level::from_str(lvl).map_err(|_| {
            anyhow!(
                "'{}' is invalid verbosity level, allowed levels are: \
                 OFF | CRIT | ERRO | WARN | INFO | DEBG | TRCE",
                lvl,
            )
        })
    }
}

/// Error type indicating non-zero process exit code.
pub struct Failure;

impl fmt::Debug for Failure {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

impl From<()> for Failure {
    #[inline]
    fn from(_: ()) -> Self {
        Self
    }
}
