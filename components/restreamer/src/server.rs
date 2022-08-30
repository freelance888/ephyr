//! HTTP servers.

pub mod client;
pub mod statistics;

use std::{net::IpAddr, time::Duration};

use ephyr_log::log;
use futures::future;
use tokio::{fs, time};

use crate::{
    cli::{Failure, Opts},
    client_stat, dvr, ffmpeg, srs, teamspeak, State,
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

    let ffmpeg_path =
        fs::canonicalize(&cfg.ffmpeg_path).await.map_err(|e| {
            log::error!("Failed to resolve FFmpeg binary path: {}", e);
        })?;

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

    let mut restreamers =
        ffmpeg::RestreamersPool::new(ffmpeg_path, state.clone());
    State::on_change("spawn_restreamers", &state.restreams, move |restreams| {
        restreamers.apply(&restreams);
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
        self::callback::run(&cfg, state),
    )
    .await?;

    drop(srs);
    // Wait for all the async `Drop`s to proceed well.
    teamspeak::finish_all_disconnects().await;

    Ok(())
}

/// Callback HTTP server responding to [SRS] HTTP callbacks.
///
/// [SRS]: https://github.com/ossrs/srs
pub mod callback {
    use actix_web::{
        error, middleware, post, web, web::Data, App, Error, HttpServer,
    };
    use ephyr_log::log;

    use crate::{
        api::srs::callback,
        cli::{Failure, Opts},
        state::{Input, InputEndpointKind, InputSrc, State, Status},
    };

    /// Runs HTTP server for exposing [SRS] [HTTP Callback API][1] on `/`
    /// endpoint for responding to [SRS] HTTP callbacks.
    ///
    /// # Errors
    ///
    /// If [`HttpServer`] cannot run due to already used port, etc.
    /// The actual error is logged.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    /// [1]: https://github.com/ossrs/srs/wiki/v4_EN_HTTPCallback
    pub async fn run(cfg: &Opts, state: State) -> Result<(), Failure> {
        Ok(HttpServer::new(move || {
            App::new()
                .app_data(Data::new(state.clone()))
                .wrap(middleware::Logger::default())
                .service(on_callback)
        })
        .bind((cfg.callback_http_ip, cfg.callback_http_port))
        .map_err(|e| log::error!("Failed to bind callback HTTP server: {}", e))?
        .run()
        .await
        .map_err(|e| {
            log::error!("Failed to run callback HTTP server: {}", e);
        })?)
    }

    /// Endpoint serving the whole [HTTP Callback API][1] for [SRS].
    ///
    /// # Errors
    ///
    /// If [SRS] HTTP callback doesn't succeed.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    /// [1]: https://github.com/ossrs/srs/wiki/v4_EN_HTTPCallback
    #[allow(clippy::unused_async)]
    #[post("/")]
    async fn on_callback(
        req: web::Json<callback::Request>,
        state: Data<State>,
    ) -> Result<&'static str, Error> {
        match req.action {
            callback::Event::OnConnect => on_connect(&req, &*state),
            callback::Event::OnPublish => on_start(&req, &*state, true),
            callback::Event::OnUnpublish => on_stop(&req, &*state, true),
            callback::Event::OnPlay => on_start(&req, &*state, false),
            callback::Event::OnStop => on_stop(&req, &*state, false),
            callback::Event::OnHls => on_hls(&req, &*state),
        }
        .map(|_| "0")
    }

    /// Handles [`callback::Event::OnConnect`].
    ///
    /// Only checks whether the appropriate [`state::Restream`] exists and its
    /// [`Input`] is enabled.
    ///
    /// # Errors
    ///
    /// If [`callback::Request::app`] matches no existing [`state::Restream`].
    ///
    /// [`state::Restream`]: crate::state::Restream
    fn on_connect(req: &callback::Request, state: &State) -> Result<(), Error> {
        state
            .restreams
            .get_cloned()
            .iter()
            .find(|r| r.input.enabled && r.key == *req.app)
            .ok_or_else(|| error::ErrorNotFound("Such `app` doesn't exist"))
            .map(|_| ())
    }

    /// Handles [`callback::Event::OnPublish`] and [`callback::Event::OnPlay`].
    ///
    /// Updates the appropriate [`state::Restream`]'s [`InputEndpoint`] to
    /// [`Status::Online`] (if [`callback::Event::OnPublish`]) and remembers the
    /// connected [SRS] client.
    ///
    /// # Errors
    ///
    /// - If [`callback::Request::vhost`], [`callback::Request::app`] or
    ///   [`callback::Request::stream`] matches no existing enabled
    ///   [`InputEndpoint`].
    /// - If [`InputEndpoint`] is not allowed to be published by external
    ///   client.
    ///
    /// [`InputEndpoint`]: crate::state::InputEndpoint
    /// [`state::Restream`]: crate::state::Restream
    ///
    /// [SRS]: https://github.com/ossrs/srs
    fn on_start(
        req: &callback::Request,
        state: &State,
        publishing: bool,
    ) -> Result<(), Error> {
        /// Traverses the given [`Input`] and all its [`Input::srcs`] looking
        /// for the one matching the specified `stream` and being enabled.
        #[must_use]
        fn lookup_input<'i>(
            input: &'i mut Input,
            stream: &str,
        ) -> Option<&'i mut Input> {
            if input.key == *stream {
                return input.enabled.then(|| input);
            }
            if let Some(InputSrc::Failover(s)) = input.src.as_mut() {
                s.inputs.iter_mut().find_map(|i| lookup_input(i, stream))
            } else {
                None
            }
        }

        let stream = req.stream.as_deref().unwrap_or_default();
        let kind = match req.vhost.as_str() {
            "hls" => InputEndpointKind::Hls,
            _ => InputEndpointKind::Rtmp,
        };

        let mut restreams = state.restreams.lock_mut();
        let restream = restreams
            .iter_mut()
            .find(|r| r.input.enabled && r.key == *req.app)
            .ok_or_else(|| error::ErrorNotFound("Such `app` doesn't exist"))?;

        let input =
            lookup_input(&mut restream.input, stream).ok_or_else(|| {
                error::ErrorNotFound("Such `stream` doesn't exist")
            })?;

        let endpoint = input
            .endpoints
            .iter_mut()
            .find(|e| e.kind == kind)
            .ok_or_else(|| {
                error::ErrorForbidden("Such `vhost` is not allowed")
            })?;

        if publishing {
            if !req.ip.is_loopback()
                && (input.src.is_some() || !endpoint.is_rtmp())
            {
                return Err(error::ErrorForbidden(
                    "Such `stream` is allowed only locally",
                ));
            }

            let publisher_id = match endpoint.srs_publisher_id.clone() {
                Some(id) => id.get_value(),
                None => None,
            };

            if publisher_id != Some(req.client_id.clone()) {
                endpoint.srs_publisher_id = Some(req.client_id.clone().into());
            }

            endpoint.status = Status::Online;
        } else {
            // `srs::ClientId` kicks the client when `Drop`ped, so we should be
            // careful here to not accidentally kick the client by creating a
            // temporary binding.
            if !endpoint.srs_player_ids.contains(&req.client_id) {
                let _ = endpoint
                    .srs_player_ids
                    .insert(req.client_id.clone().into());
            }
        }
        Ok(())
    }

    /// Handles [`callback::Event::OnUnpublish`].
    ///
    /// Updates the appropriate [`state::Restream`]'s [`InputEndpoint`] to
    /// [`Status::Offline`].
    ///
    /// # Errors
    ///
    /// If [`callback::Request::vhost`], [`callback::Request::app`] or
    /// [`callback::Request::stream`] matches no existing [`InputEndpoint`].
    ///
    /// [`InputEndpoint`]: crate::state::InputEndpoint
    /// [`state::Restream`]: crate::state::Restream
    fn on_stop(
        req: &callback::Request,
        state: &State,
        publishing: bool,
    ) -> Result<(), Error> {
        /// Traverses the given [`Input`] and all its [`Input::srcs`] looking
        /// for the one matching the specified `stream`.
        #[must_use]
        fn lookup_input<'i>(
            input: &'i mut Input,
            stream: &str,
        ) -> Option<&'i mut Input> {
            if input.key == *stream {
                return Some(input);
            }
            if let Some(InputSrc::Failover(s)) = input.src.as_mut() {
                s.inputs.iter_mut().find_map(|i| lookup_input(i, stream))
            } else {
                None
            }
        }

        let stream = req.stream.as_deref().unwrap_or_default();
        let kind = match req.vhost.as_str() {
            "hls" => InputEndpointKind::Hls,
            _ => InputEndpointKind::Rtmp,
        };

        let mut restreams = state.restreams.lock_mut();
        let restream = restreams
            .iter_mut()
            .find(|r| r.key == *req.app)
            .ok_or_else(|| error::ErrorNotFound("Such `app` doesn't exist"))?;

        let input =
            lookup_input(&mut restream.input, stream).ok_or_else(|| {
                error::ErrorNotFound("Such `stream` doesn't exist")
            })?;

        let endpoint = input
            .endpoints
            .iter_mut()
            .find(|e| e.kind == kind)
            .ok_or_else(|| {
                error::ErrorForbidden("Such `vhost` is not allowed")
            })?;

        if publishing {
            endpoint.srs_publisher_id = None;
            endpoint.status = Status::Offline;
        } else {
            let _ = endpoint.srs_player_ids.remove(&req.client_id);
        }
        Ok(())
    }

    /// Handles [`callback::Event::OnHls`].
    ///
    /// Checks whether the appropriate [`state::Restream`] with an
    /// [`InputEndpointKind::Hls`] exists and its [`Input`] is enabled.
    ///
    /// # Errors
    ///
    /// If [`callback::Request::vhost`], [`callback::Request::app`] or
    /// [`callback::Request::stream`] matches no existing [`InputEndpoint`]
    /// of [`InputEndpointKind::Hls`].
    ///
    /// [`InputEndpoint`]: crate::state::InputEndpoint
    /// [`state::Restream`]: crate::state::Restream
    fn on_hls(req: &callback::Request, state: &State) -> Result<(), Error> {
        /// Traverses the given [`Input`] and all its [`Input::srcs`] looking
        /// for the one matching the specified `stream` and being enabled.
        #[must_use]
        fn lookup_input<'i>(
            input: &'i mut Input,
            stream: &str,
        ) -> Option<&'i mut Input> {
            if input.key == *stream {
                return input.enabled.then(|| input);
            }
            if let Some(InputSrc::Failover(s)) = input.src.as_mut() {
                s.inputs.iter_mut().find_map(|i| lookup_input(i, stream))
            } else {
                None
            }
        }

        let stream = req.stream.as_deref().unwrap_or_default();
        let kind = (req.vhost.as_str() == "hls")
            .then(|| InputEndpointKind::Hls)
            .ok_or_else(|| {
                error::ErrorForbidden("Such `vhost` is not allowed")
            })?;

        let mut restreams = state.restreams.lock_mut();
        let restream = restreams
            .iter_mut()
            .find(|r| r.input.enabled && r.key == *req.app)
            .ok_or_else(|| error::ErrorNotFound("Such `app` doesn't exist"))?;

        let endpoint = lookup_input(&mut restream.input, stream)
            .ok_or_else(|| error::ErrorNotFound("Such `stream` doesn't exist"))?
            .endpoints
            .iter_mut()
            .find(|e| e.kind == kind)
            .ok_or_else(|| {
                error::ErrorNotFound("Such `stream` doesn't exist")
            })?;

        if endpoint.status != Status::Online {
            return Err(error::ErrorImATeapot("Not ready to serve"));
        }

        // `srs::ClientId` kicks the client when `Drop`ped, so we should be
        // careful here to not accidentally kick the client by creating a
        // temporary binding.
        if !endpoint.srs_player_ids.contains(&req.client_id) {
            let _ =
                endpoint.srs_player_ids.insert(req.client_id.clone().into());
        }
        Ok(())
    }
}

/// Tries to detect public IP address of the machine where this application
/// runs.
///
/// See [`public_ip`] crate for details.
pub async fn detect_public_ip() -> Option<IpAddr> {
    public_ip::addr().await
}
