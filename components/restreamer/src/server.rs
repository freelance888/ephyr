//! HTTP servers.

pub mod client;
pub mod srs_callback;
pub mod statistics;

use std::{net::IpAddr, time::Duration};

use ephyr_log::log;
use futures::future;
use gst_client::GstClient;
use tokio::{fs, time};
use url::Url;

use crate::{
    cli::{Failure, Opts},
    client_stat, dvr, ffmpeg, restreamer, srs, teamspeak, State,
};

/// Initializes and runs all application's HTTP servers.
///
/// # Errors
///
/// If some [`HttpServer`] cannot run due to already used port, etc.
/// The actual error is witten to logs.
///
/// [`HttpServer`]: actix_web::HttpServer
#[actix_web::main]
pub async fn run(mut cfg: Opts) -> Result<(), Failure> {
    if cfg.public_host.is_none() {
        cfg.public_host = Some(
            detect_public_ip()
                .await
                .ok_or_else(|| {
                    log::error!("Cannot detect server's public IP address");
                })?
                .to_string(),
        );
    }

    // let ffmpeg_path =
    //     fs::canonicalize(&cfg.ffmpeg_path).await.map_err(|e| {
    //         log::error!("Failed to resolve FFmpeg binary path: {}", e);
    //     })?;

    // Use Url only for validation as issues with thread safety
    let gstd_uri = Url::parse(&format!(
        "http://{}:{}",
        cfg.gstd_http_ip, cfg.gstd_http_port
    ))
    .map_err(|e| log::error!("Failed to parse GStD URL: {}", e))?;

    let state = State::try_new(&cfg.state_path)
        .await
        .map_err(|e| log::error!("Failed to initialize server state: {}", e))?;

    let srs = srs::Server::try_new(
        &cfg.srs_path,
        &srs::Config {
            callback_port: cfg.callback_http_port,
            http_server_dir: cfg.srs_http_dir.clone().into(),
            log_level: cfg.verbose.map(Into::into).unwrap_or_default(),
        },
    )
    .await
    .map_err(|e| log::error!("Failed to initialize SRS server: {}", e))?;
    State::on_change(
        "cleanup_dvr_files",
        &state.restreams,
        |restreams| async move {
            // Wait for all the re-streaming processes to release DVR files.
            time::sleep(Duration::from_secs(1)).await;
            dvr::Storage::global().cleanup(&restreams).await;
        },
    );

    // let mut ffmpeg_restreamers =
    //     ffmpeg::RestreamersPool::new(ffmpeg_path, state.clone());
    // State::on_change("spawn_restreamers", &state.restreams, move |restreams| {
    //     ffmpeg_restreamers.apply(&restreams);
    //     future::ready(())
    // });

    let client = GstClient::from(gstd_uri);
    // TODO: add ping to GStD
    let mut gstd_restreamers =
        restreamer::RestreamersPool::new(client, state.clone());
    State::on_change("spawn_restreamers", &state.restreams, move |restreams| {
        gstd_restreamers.apply(&restreams);
        future::ready(())
    });

    let mut client_jobs = client_stat::ClientJobsPool::new(state.clone());
    State::on_change("spawn_client_jobs", &state.clients, move |clients| {
        client_jobs.apply(&clients);
        future::ready(())
    });

    future::try_join3(
        self::client::run(&cfg, state.clone()),
        self::statistics::run(state.clone()),
        self::srs_callback::run(&cfg, state),
    )
    .await?;

    drop(srs);
    // Wait for all the async `Drop`s to proceed well.
    teamspeak::finish_all_disconnects().await;

    Ok(())
}

/// Tries to detect public IP address of the machine where this application
/// runs.
///
/// See [`public_ip`] crate for details.
pub async fn detect_public_ip() -> Option<IpAddr> {
    public_ip::addr().await
}
