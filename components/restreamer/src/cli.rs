//! CLI (command line interface).

use clap::Parser;
use ephyr_log::tracing;
use std::{fmt, net::IpAddr, path::PathBuf, str::FromStr as _};

/// CLI (command line interface) of the re-streamer server.
#[derive(Clone, Debug, Parser)]
#[command(about = "RTMP re-streamer server")]
pub struct Opts {
    /// Debug mode of the server.
    #[arg(short, long, help = "Enables debug mode")]
    pub debug: bool,

    /// IP address for the server to listen client HTTP requests on.
    #[arg(
        long,
        env = "EPHYR_RESTREAMER_CLIENT_HTTP_IP",
        default_value = "0.0.0.0",
        help = "IP to listen client HTTP on",
        long_help = "IP address for the server to listen client HTTP requests \
                     on"
    )]
    pub client_http_ip: IpAddr,

    /// Port for the server to listen client HTTP requests on.
    #[arg(
        long,
        env = "EPHYR_RESTREAMER_CLIENT_HTTP_PORT",
        default_value = "80",
        help = "Port to listen client HTTP on",
        long_help = "Port for the server to listen client HTTP requests on"
    )]
    pub client_http_port: u16,

    /// IP address for the server to listen RTMP callback HTTP requests on.
    #[arg(
        long,
        env = "EPHYR_RESTREAMER_CALLBACK_HTTP_IP",
        default_value = "127.0.0.1",
        help = "IP to listen callback HTTP on",
        long_help = "IP address for the server to listen RTMP callback HTTP \
                     requests on"
    )]
    pub callback_http_ip: IpAddr,

    /// Port for the server to listen RTMP callback HTTP requests on.
    #[arg(
        long,
        env = "EPHYR_RESTREAMER_CALLBACK_HTTP_PORT",
        default_value = "8081",
        help = "Port to listen callback HTTP on",
        long_help = "Port for the server to listen RTMP callback HTTP requests \
                     on"
    )]
    pub callback_http_port: u16,

    /// Path to a file to persist the server's state in.
    #[arg(
        short,
        long,
        env = "EPHYR_RESTREAMER_STATE_PATH",
        default_value = "./state.json",
        help = "Path to a file to persist state in",
        long_help = "Path to a file to persist the server's state in"
    )]
    pub state_path: PathBuf,

    /// Path to [SRS] installation directory.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    #[arg(
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
    #[arg(
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
    #[arg(
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
    #[arg(
        long,
        env = "EPHYR_RESTREAMER_PUBLIC_HOST",
        help = "Public host to access the server",
        long_help = "Host to access the server in public networks \
                     (auto-detects by default)"
    )]
    pub public_host: Option<String>,

    /// Verbosity level of the server logs.
    #[arg(
        short,
        long,
        value_parser(tracing::Level::from_str),
        help = "Logs verbosity level: INFO | DEBUG | TRACE"
    )]
    pub verbose: Option<tracing::Level>,

    /// Logs format for displaying.
    #[arg(
        short,
        long,
        env = "EPHYR_RESTREAMER_LOG_FORMAT",
        help = "Logs format: JSON | COMPACT"
    )]
    pub log_format: Option<ephyr_log::LogFormat>,

    /// Path for local video files.
    #[arg(
        long,
        env = "EPHYR_RESTREAMER_VIDEO_FILE_ROOT",
        default_value = "/tmp/ephyr",
        help = "Path where video files will be stored",
        long_help = "Here the video files that can be streamed to the output \
                     will be downloaded."
    )]
    pub file_root: PathBuf,

    /// IP address of [OpenTelemetry] collector server to send logs to.
    ///
    /// [OpenTelemetry]: https://OpenTelemetry.io
    #[arg(
        long,
        env = "EPHYR_RESTREAMER_OTLP_COLLECTOR_IP",
        help = "IP of OTLP collector to send traces",
        long_help = "Uses for aggregation of traces for OTLP collector"
    )]
    pub otlp_collector_ip: Option<IpAddr>,

    /// Port of [OpenTelemetry] collector server to send logs to.
    ///
    /// In our case as we send data with gRPC so port is typically `4317`.
    ///
    /// [OpenTelemetry]: https://OpenTelemetry.io
    #[arg(
        long,
        env = "EPHYR_RESTREAMER_OTLP_COLLECTOR_PORT",
        help = "Port of OTLP collector to send traces",
        long_help = "Uses for aggregation of traces for OTLP collector"
    )]
    pub otlp_collector_port: Option<u16>,

    /// Service name to collect traces to [OpenTelemetry] collector.
    ///
    /// [OpenTelemetry]: https://OpenTelemetry.io
    #[arg(
        long,
        env = "EPHYR_RESTREAMER_SERVICE_NAME",
        default_value = "ephyr-restreamer",
        help = "Service name to collect traces to OTLP collector",
        long_help = "Uses for aggregation of traces for OTLP collector"
    )]
    pub service_name: String,
}

impl Opts {
    /// Parses CLI [`Opts`] from command line arguments.
    ///
    /// Prints the error message and quits the program in case of failure.
    #[inline]
    #[must_use]
    pub fn from_args() -> Self {
        <Self as Parser>::parse()
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
