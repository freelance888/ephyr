//! HTTP servers.

use std::{net::IpAddr, time::Duration};

use ephyr_log::log;
use futures::future;
use tokio::{fs, time};

use crate::{
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
    file_manager.check_files(state.restreams.get_cloned().iter());
    State::on_change("file_manager", &state.restreams, move |restreams| {
        file_manager.check_files(restreams.iter());
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
    use std::fmt;

    const MIX_ROUTE: &str = "/mix";
    const MIX_ROUTE_API: &str = "/api-mix";
    const STATISTICS_ROUTE_API: &str = "/api-statistics";
    const FULL_STREAM_ROUTE: &str = "/full-stream";
    const INDEX_FILE: &str = "index.html";

    pub mod public_dir {
        #![allow(clippy::must_use_candidate, unused_results)]
        #![doc(hidden)]

        include!(concat!(env!("OUT_DIR"), "/generated.rs"));
    }

    pub mod public_mix_dir {
        #![allow(clippy::must_use_candidate, unused_results)]
        #![doc(hidden)]

        include!(concat!(env!("OUT_DIR"), "/generated_mix.rs"));
    }

    pub mod public_dashboard_dir {
        #![allow(clippy::must_use_candidate, unused_results)]
        #![doc(hidden)]

        include!(concat!(env!("OUT_DIR"), "/generated_dashboard.rs"));
    }

    pub mod public_full_stream_dir {
        #![allow(clippy::must_use_candidate, unused_results)]
        #![doc(hidden)]

        include!(concat!(env!("OUT_DIR"), "/generated_full_stream.rs"));
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
            let root_dir_files = public_dir::generate();
            let mix_dir_files = public_mix_dir::generate();
            let dashboard_dir_files = public_dashboard_dir::generate();
            let full_stream_dir_files = public_full_stream_dir::generate();

            let mut app = App::new()
                .app_data(stored_cfg.clone())
                .app_data(state.clone())
                .app_data(
                    basic::Config::default().realm("Any login is allowed"),
                )
                .app_data(web::Data::new(api::graphql::client::schema()))
                .app_data(web::Data::new(api::graphql::mix::schema()))
                .app_data(web::Data::new(api::graphql::dashboard::schema()))
                .app_data(web::Data::new(api::graphql::statistics::schema()))
                .wrap(middleware::Logger::default())
                .wrap_fn(|req, srv| match authorize(req) {
                    Ok(req) => srv.call(req).left_future(),
                    Err(e) => future::err(e).right_future(),
                })
                .service(graphql_client)
                .service(graphql_mix)
                .service(graphql_statistics)
                .service(graphql_dashboard);
            if in_debug_mode {
                app = app
                    .service(playground_client)
                    .service(playground_mix)
                    .service(playground_statistics)
                    .service(playground_dashboard);
            }
            app.service(
                ResourceFiles::new(MIX_ROUTE, mix_dir_files)
                    .resolve_not_found_to(INDEX_FILE),
            )
            .service(
                ResourceFiles::new("/dashboard", dashboard_dir_files)
                    .resolve_not_found_to(INDEX_FILE),
            )
            .service(
                ResourceFiles::new(FULL_STREAM_ROUTE, full_stream_dir_files)
                    .resolve_not_found_to(INDEX_FILE)
            )
            .service(ResourceFiles::new("/", root_dir_files))
        })
        .bind((cfg.client_http_ip, cfg.client_http_port))
        .map_err(|e| log::error!("Failed to bind client HTTP server: {}", e))?
        .run()
        .await
        .map_err(|e| log::error!("Failed to run client HTTP server: {}", e))?)
    }

    /// List of schemes
    pub enum SchemaKind {
        /// Full schema
        Schema(web::Data<api::graphql::client::Schema>),

        /// Single output schema for mixing
        SchemaMix(web::Data<api::graphql::mix::Schema>),

        /// Dashboard schema
        SchemaDashboard(web::Data<api::graphql::dashboard::Schema>),

        /// Statistics schema
        SchemaStatistics(web::Data<api::graphql::statistics::Schema>),
    }

    impl fmt::Debug for SchemaKind {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "")
        }
    }

    /// Endpoint serving [`api::`graphql`::statistics`] application
    #[route("/api-statistics", method = "GET", method = "POST")]
    async fn graphql_statistics(
        req: HttpRequest,
        payload: web::Payload,
        schema: web::Data<api::graphql::statistics::Schema>,
    ) -> Result<HttpResponse, Error> {
        graphql(req, payload, SchemaKind::SchemaStatistics(schema)).await
    }

    /// Endpoint serving [`api::`graphql`::dashboard`] application
    #[route("/api-dashboard", method = "GET", method = "POST")]
    async fn graphql_dashboard(
        req: HttpRequest,
        payload: web::Payload,
        schema: web::Data<api::graphql::dashboard::Schema>,
    ) -> Result<HttpResponse, Error> {
        graphql(req, payload, SchemaKind::SchemaDashboard(schema)).await
    }

    /// Endpoint serving [`api::`graphql`::mix`] for single output
    /// application
    #[route("/api-mix", method = "GET", method = "POST")]
    async fn graphql_mix(
        req: HttpRequest,
        payload: web::Payload,
        schema: web::Data<api::graphql::mix::Schema>,
    ) -> Result<HttpResponse, Error> {
        graphql(req, payload, SchemaKind::SchemaMix(schema)).await
    }

    /// Endpoint serving [`api::`graphql`::client`] for main application
    #[route("/api", method = "GET", method = "POST")]
    async fn graphql_client(
        req: HttpRequest,
        payload: web::Payload,
        schema: web::Data<api::graphql::client::Schema>,
    ) -> Result<HttpResponse, Error> {
        graphql(req, payload, SchemaKind::Schema(schema)).await
    }

    async fn graphql(
        req: HttpRequest,
        payload: web::Payload,
        schema_kind: SchemaKind,
    ) -> Result<HttpResponse, Error> {
        let ctx = api::graphql::Context::new(req.clone());
        if req.head().upgrade() {
            let cfg = ConnectionConfig::new(ctx)
                .with_keep_alive_interval(Duration::from_secs(5));

            match schema_kind {
                SchemaKind::Schema(s) => {
                    subscriptions_handler(req, payload, s.into_inner(), cfg)
                        .await
                }
                SchemaKind::SchemaMix(s) => {
                    subscriptions_handler(req, payload, s.into_inner(), cfg)
                        .await
                }
                SchemaKind::SchemaDashboard(s) => {
                    subscriptions_handler(req, payload, s.into_inner(), cfg)
                        .await
                }
                SchemaKind::SchemaStatistics(s) => {
                    subscriptions_handler(req, payload, s.into_inner(), cfg)
                        .await
                }
            }
        } else {
            match schema_kind {
                SchemaKind::Schema(s) => {
                    graphql_handler(&s, &ctx, req, payload).await
                }
                SchemaKind::SchemaMix(s) => {
                    graphql_handler(&s, &ctx, req, payload).await
                }
                SchemaKind::SchemaDashboard(s) => {
                    graphql_handler(&s, &ctx, req, payload).await
                }
                SchemaKind::SchemaStatistics(s) => {
                    graphql_handler(&s, &ctx, req, payload).await
                }
            }
        }
    }

    /// Endpoint serving [GraphQL Playground][1] for exploring
    /// [`api::graphql::client`].
    ///
    /// [1]: https://github.com/graphql/graphql-playground
    #[get("/api/playground")]
    async fn playground_client() -> HttpResponse {
        playground().await
    }

    /// Endpoint serving [GraphQL Playground][1] for exploring
    /// [`api::graphql::mix`].
    ///
    /// [1]: https://github.com/graphql/graphql-playground
    #[get("/api-mix/playground")]
    async fn playground_mix() -> HttpResponse {
        playground().await
    }

    /// Endpoint serving [GraphQL Playground][1] for exploring
    /// [`api::graphql::dashboard`].
    ///
    /// [1]: https://github.com/graphql/graphql-playground
    #[get("/api-dashboard/playground")]
    async fn playground_dashboard() -> HttpResponse {
        playground().await
    }

    /// Endpoint serving [GraphQL Playground][1] for exploring
    /// [`api::graphql::statistics`].
    ///
    /// [1]: https://github.com/graphql/graphql-playground
    #[get("/api-statistics/playground")]
    async fn playground_statistics() -> HttpResponse {
        playground().await
    }

    #[allow(clippy::unused_async)]
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
        let route = req.uri().path();
        log::debug!("authorize URI PATH: {}", route);

        if route.starts_with(STATISTICS_ROUTE_API) {
            return Ok(req);
        }

        let is_mix_auth =
            route.starts_with(MIX_ROUTE) || route.starts_with(MIX_ROUTE_API);
        let settings = req.app_data::<State>().unwrap().settings.get_cloned();

        let hash = if is_mix_auth {
            settings.password_output_hash
        } else {
            settings.password_hash
        };

        let hash = match hash {
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
            "hls" => vec![InputEndpointKind::Hls],
            _ => vec![InputEndpointKind::Rtmp, InputEndpointKind::File],
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
            .find(|e| kind.contains(&e.kind))
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

/// Module which collects server statistics and updates them every second
pub mod statistics {
    use std::time::Duration;
    use systemstat::{Platform, System};
    use tokio::time;

    use crate::{cli::Failure, display_panic, state::ServerInfo, State};
    use ephyr_log::log;
    use futures::FutureExt;
    use std::panic::AssertUnwindSafe;

    /// Runs statistics monitoring
    ///
    /// # Panics
    /// Panic is captured to log. Could be panicked during getting server
    /// statistics.
    ///
    /// # Errors
    /// No return errors expected. Preserved return signature in order to
    /// run in `future::try_join3`
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    pub async fn run(state: State) -> Result<(), Failure> {
        // we use tx_last and rx_last to compute the delta
        // (send/receive bytes last second)
        let mut tx_last: f64 = 0.0;
        let mut rx_last: f64 = 0.0;

        let spawner = async move {
            loop {
                let state = &state;

                let _ = AssertUnwindSafe(async {
                    let sys = System::new();

                    let mut info = ServerInfo::default();

                    // Update cpu usage
                    match sys.cpu_load_aggregate() {
                        Ok(cpu) => {
                            // Need to wait some time to let the library compute
                            // CPU usage.
                            // Do not change delay time, since it is also used
                            // further to compute network statistics
                            // (bytes sent/received last second)
                            time::sleep(Duration::from_secs(1)).await;
                            let cpu = cpu.done().unwrap();

                            // in percents
                            info.update_cpu(Some(
                                f64::from(1.0 - cpu.idle) * 100.0,
                            ));
                        }
                        Err(x) => {
                            info.set_error(Some(x.to_string()));
                            log::error!("Statistics. CPU load: error: {}", x);
                        }
                    }

                    // Update ram usage
                    match sys.memory() {
                        Ok(mem) => {
                            // in megabytes
                            let mem_total = mem.total.as_u64() / 1024 / 1024;
                            // in megabytes
                            let mem_free = mem.free.as_u64() / 1024 / 1024;
                            info.update_ram(
                                Some(mem_total as f64),
                                Some(mem_free as f64),
                            );
                        }
                        Err(x) => {
                            info.set_error(Some(x.to_string()));
                            log::error!("Statistics. Memory: error: {}", x);
                        }
                    }

                    // Update network usage
                    match sys.networks() {
                        Ok(netifs) => {
                            // Sum up along network interfaces
                            let mut tx: f64 = 0.0;
                            let mut rx: f64 = 0.0;

                            // Note that the sum of sent/received bytes are
                            // computed among all the available network
                            // interfaces
                            for netif in netifs.values() {
                                if let Ok(netstats) =
                                    sys.network_stats(&netif.name)
                                {
                                    // in megabytes
                                    tx += netstats.tx_bytes.as_u64() as f64
                                        / 1024.0
                                        / 1024.0;
                                    // in megabytes
                                    rx += netstats.rx_bytes.as_u64() as f64
                                        / 1024.0
                                        / 1024.0;
                                }
                            }

                            // Compute delta
                            let tx_delta = tx - tx_last;
                            let rx_delta = rx - rx_last;

                            // Update server info
                            info.update_traffic_usage(
                                Some(tx_delta),
                                Some(rx_delta),
                            );

                            tx_last = tx;
                            rx_last = rx;
                        }
                        Err(x) => {
                            info.set_error(Some(x.to_string()));
                            log::error!("Statistics. Networks: error: {}", x);
                        }
                    }

                    *state.server_info.lock_mut() = info;
                })
                .catch_unwind()
                .await
                .map_err(|p| {
                    log::crit!(
                        "Panicked while getting server statistics {}",
                        display_panic(&p),
                    );
                });
            }
        };

        drop(tokio::spawn(spawner));

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
