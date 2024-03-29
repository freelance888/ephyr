//! HTTP servers.

pub mod client;
pub mod periodic_tasks;
pub mod srs_callback;

use std::{net::IpAddr, time::Duration};

use ephyr_log::{tracing, TelemetryConfig};
use futures::future;
use tokio::{fs, time};

use crate::{
    broadcaster::Broadcaster,
    cli::{Failure, Opts},
    client_stat, dvr, ffmpeg,
    file_manager::FileManager,
    srs, teamspeak, State,
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
    TelemetryConfig::new(cfg.verbose)
        .otlp_endpoint(cfg.otlp_collector_ip, cfg.otlp_collector_port)
        .service_name(cfg.service_name.clone())
        .log_format(cfg.log_format.clone())
        .init();

    if cfg.public_host.is_none() {
        cfg.public_host = Some(
            detect_public_ip()
                .await
                .ok_or_else(|| {
                    tracing::error!("Cannot detect server's public IP address");
                })?
                .to_string(),
        );
    }
    tracing::info!(
        "Public host: http://{}:{}",
        cfg.public_host.as_deref().unwrap(),
        cfg.client_http_port
    );

    let ffmpeg_path =
        fs::canonicalize(&cfg.ffmpeg_path).await.map_err(|e| {
            tracing::error!("Failed to resolve FFmpeg binary path: {e}");
        })?;

    let state = State::try_new(&cfg.state_path).await.map_err(|e| {
        tracing::error!("Failed to initialize server state: {e}");
    })?;

    let srs = srs::Server::try_new(
        &cfg.srs_path,
        &srs::Config {
            callback_port: cfg.callback_http_port,
            http_server_dir: cfg.srs_http_dir.clone().into(),
            log_level: cfg.verbose.map(Into::into).unwrap_or_default(),
        },
    )
    .await
    .map_err(|e| tracing::error!("Failed to initialize SRS server: {e}"))?;
    State::on_change(
        "cleanup_dvr_files",
        &state.restreams,
        |restreams| async move {
            // Wait for all the re-streaming processes to release DVR files.
            time::sleep(Duration::from_secs(1)).await;
            dvr::Storage::global().cleanup(&restreams).await;
        },
    );

    let mut restreamers = ffmpeg::RestreamersPool::new(
        ffmpeg_path,
        state.clone(),
        cfg.file_root.clone(),
    );
    State::on_change("spawn_restreamers", &state.restreams, move |restreams| {
        restreamers.apply(&restreams);
        future::ready(())
    });
    let file_manager = FileManager::new(&cfg, state.clone());
    file_manager.check_files();
    State::on_change("handle_fm_commands", &state.file_commands, move |_| {
        file_manager.handle_commands();
        future::ready(())
    });

    let mut client_jobs = client_stat::ClientJobsPool::new(state.clone());
    State::on_change("spawn_client_jobs", &state.clients, move |clients| {
        client_jobs.start_statistics_loop(&clients);
        future::ready(())
    });

    let mut broadcaster = Broadcaster::new(state.clone());
    State::on_change(
        "handle_dashboard_commands",
        &state.dashboard_commands,
        move |_| {
            broadcaster.handle_commands();
            future::ready(())
        },
    );

    future::try_join3(
        self::client::run(&cfg, state.clone()),
        self::periodic_tasks::run(state.clone()),
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
