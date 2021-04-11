//! HTTP servers.

use std::{net::IpAddr, time::Duration};

use ephyr_log::log;
use futures::future;
use tokio::{fs, time};

use crate::{
    cli::{Failure, Opts},
    dvr, ffmpeg, srs, teamspeak, State,
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
                    log::error!("Cannot detect server's public IP address")
                })?
                .to_string(),
        );
    }

    let ffmpeg_path =
        fs::canonicalize(&cfg.ffmpeg_path).await.map_err(|e| {
            log::error!("Failed to resolve FFmpeg binary path: {}", e)
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
            time::delay_for(Duration::from_secs(1)).await;
            dvr::Storage::global().cleanup(&restreams).await;
        },
    );

    let mut restreamers =
        ffmpeg::RestreamersPool::new(ffmpeg_path, state.clone());
    State::on_change("spawn_restreamers", &state.restreams, move |restreams| {
        restreamers.apply(&restreams);
        future::ready(())
    });

    future::try_join(
        self::client::run(&cfg, state.clone()),
        self::callback::run(&cfg, state),
    )
    .await?;

    drop(srs);
    // Wait for all the async `Drop`s to proceed well.
    teamspeak::finish_all_disconnects().await;

    Ok(())
}

/// Client HTTP server responding to client requests.
pub mod client {
    use std::time::Duration;

    use actix_service::Service as _;
    use actix_web::{
        dev::ServiceRequest, get, middleware, route, web, App, Error,
        HttpRequest, HttpResponse, HttpServer,
    };
    use actix_web_httpauth::extractors::{
        basic::{self, BasicAuth},
        AuthExtractor as _, AuthExtractorConfig, AuthenticationError,
    };
    use actix_web_static_files::ResourceFiles;
    use ephyr_log::log;
    use futures::{future, FutureExt as _};
    use juniper::http::playground::playground_source;
    use juniper_actix::{
        graphql_handler, subscriptions::subscriptions_handler,
    };
    use juniper_graphql_ws::ConnectionConfig;

    use crate::{
        api,
        cli::{Failure, Opts},
        State,
    };

    pub mod public_dir {
        #![allow(clippy::must_use_candidate, unused_results)]
        #![doc(hidden)]

        include!(concat!(env!("OUT_DIR"), "/generated.rs"));
    }

    /// Runs client HTTP server.
    ///
    /// Client HTTP server serves [`api::graphql::client`] on `/` endpoint.
    ///
    /// # Playground
    ///
    /// If [`cli::Opts::debug`] is specified then additionally serves
    /// [GraphQL Playground][2] on `/api/playground` endpoint with no
    /// authorization required.
    ///
    /// # Errors
    ///
    /// If [`HttpServer`] cannot run due to already used port, etc.
    /// The actual error is logged.
    ///
    /// [`cli::Opts::debug`]: crate::cli::Opts::debug
    /// [2]: https://github.com/graphql/graphql-playground
    pub async fn run(cfg: &Opts, state: State) -> Result<(), Failure> {
        let in_debug_mode = cfg.debug;

        let stored_cfg = cfg.clone();

        Ok(HttpServer::new(move || {
            let public_dir_files = public_dir::generate();
            let mut app = App::new()
                .app_data(stored_cfg.clone())
                .app_data(state.clone())
                .app_data(
                    basic::Config::default().realm("Any login is allowed"),
                )
                .data(api::graphql::client::schema())
                .wrap(middleware::Logger::default())
                .wrap_fn(|req, srv| match authorize(req) {
                    Ok(req) => srv.call(req).left_future(),
                    Err(e) => future::err(e).right_future(),
                })
                .service(graphql);
            if in_debug_mode {
                app = app.service(playground);
            }
            app.service(ResourceFiles::new("/", public_dir_files))
        })
        .bind((cfg.client_http_ip, cfg.client_http_port))
        .map_err(|e| log::error!("Failed to bind client HTTP server: {}", e))?
        .run()
        .await
        .map_err(|e| log::error!("Failed to run client HTTP server: {}", e))?)
    }

    /// Endpoint serving [`api::graphql::client`] directly.
    ///
    /// # Errors
    ///
    /// If GraphQL operation execution errors or fails.
    #[route("/api", method = "GET", method = "POST")]
    async fn graphql(
        req: HttpRequest,
        payload: web::Payload,
        schema: web::Data<api::graphql::client::Schema>,
    ) -> Result<HttpResponse, Error> {
        let ctx = api::graphql::Context::new(req.clone());
        if req.head().upgrade() {
            let cfg = ConnectionConfig::new(ctx)
                .with_keep_alive_interval(Duration::from_secs(5));
            subscriptions_handler(req, payload, schema.into_inner(), cfg).await
        } else {
            graphql_handler(&schema, &ctx, req, payload).await
        }
    }

    /// Endpoint serving [GraphQL Playground][1] for exploring
    /// [`api::graphql::client`].
    ///
    /// [1]: https://github.com/graphql/graphql-playground
    #[get("/api/playground")]
    async fn playground() -> HttpResponse {
        // Constructs API URL relatively to the current HTTP request's scheme
        // and authority.
        let html = playground_source("__API_URL__", None).replace(
            "'__API_URL__'",
            r"document.URL.replace(/\/playground$/, '')",
        );
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)
    }

    /// Performs [`HttpRequest`] [Basic authorization][1] as middleware against
    /// [`State::password_hash`]. Doesn't consider username anyhow.
    ///
    /// No-op if [`State::password_hash`] is [`None`].
    ///
    /// [1]: https://en.wikipedia.org/wiki/Basic_access_authentication
    fn authorize(req: ServiceRequest) -> Result<ServiceRequest, Error> {
        let hash = match req
            .app_data::<State>()
            .unwrap()
            .settings
            .get_cloned()
            .password_hash
        {
            Some(h) => h,
            None => return Ok(req),
        };

        let err = || {
            AuthenticationError::new(
                req.app_data::<basic::Config>()
                    .unwrap()
                    .clone()
                    .into_inner(),
            )
        };

        let auth = BasicAuth::from_service_request(&req).into_inner()?;
        let pass = auth.password().ok_or_else(err)?;
        if argon2::verify_encoded(hash.as_str(), pass.as_bytes()) != Ok(true) {
            return Err(err().into());
        }

        Ok(req)
    }
}

/// Callback HTTP server responding to [SRS] HTTP callbacks.
///
/// [SRS]: https://github.com/ossrs/srs
pub mod callback {
    use actix_web::{error, middleware, post, web, App, Error, HttpServer};
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
    /// [1]: https://github.com/ossrs/srs/wiki/v3_EN_HTTPCallback
    pub async fn run(cfg: &Opts, state: State) -> Result<(), Failure> {
        Ok(HttpServer::new(move || {
            App::new()
                .data(state.clone())
                .wrap(middleware::Logger::default())
                .service(on_callback)
        })
        .bind((cfg.callback_http_ip, cfg.callback_http_port))
        .map_err(|e| log::error!("Failed to bind callback HTTP server: {}", e))?
        .run()
        .await
        .map_err(|e| {
            log::error!("Failed to run callback HTTP server: {}", e)
        })?)
    }

    /// Endpoint serving the whole [HTTP Callback API][1] for [SRS].
    ///
    /// # Errors
    ///
    /// If [SRS] HTTP callback doesn't succeed.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    /// [1]: https://github.com/ossrs/srs/wiki/v3_EN_HTTPCallback
    #[post("/")]
    async fn on_callback(
        req: web::Json<callback::Request>,
        state: web::Data<State>,
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

            if endpoint.srs_publisher_id.as_ref().map(|id| **id)
                != Some(req.client_id)
            {
                endpoint.srs_publisher_id = Some(req.client_id.into());
            }

            endpoint.status = Status::Online;
        } else {
            // `srs::ClientId` kicks the client when `Drop`ped, so we should be
            // careful here to not accidentally kick the client by creating a
            // temporary binding.
            if !endpoint.srs_player_ids.contains(&req.client_id) {
                let _ = endpoint.srs_player_ids.insert(req.client_id.into());
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
            let _ = endpoint.srs_player_ids.insert(req.client_id.into());
        }
        Ok(())
    }
}

/// Tries to detect public IP address of the machine where this application
/// runs.
///
/// See [`public_ip`] crate for details.
pub async fn detect_public_ip() -> Option<IpAddr> {
    use public_ip::{dns, http, BoxToResolver, ToResolver as _};

    public_ip::resolve_address(
        vec![
            BoxToResolver::new(dns::OPENDNS_RESOLVER),
            BoxToResolver::new(http::HTTP_IPIFY_ORG_RESOLVER),
        ]
        .to_resolver(),
    )
    .await
}
