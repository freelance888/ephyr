//! Client HTTP server responding to client requests.
use std::time::Duration;

use actix_service::Service as _;
use actix_web::{
    dev::ServiceRequest, get, middleware, route, web, App, Error, HttpRequest,
    HttpResponse, HttpServer,
};
use actix_web_httpauth::extractors::{
    basic::{self, BasicAuth},
    AuthExtractor as _, AuthExtractorConfig, AuthenticationError,
};
use actix_web_static_files::ResourceFiles;
use ephyr_log::log;
use futures::{future, FutureExt as _};
use juniper::http::playground::playground_source;
use juniper_actix::{graphql_handler, subscriptions::subscriptions_handler};
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

        let mut app = App::new()
            .app_data(stored_cfg.clone())
            .app_data(state.clone())
            .app_data(basic::Config::default().realm("Any login is allowed"))
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
        .service(ResourceFiles::new("/", root_dir_files))
    })
    .bind((cfg.client_http_ip, cfg.client_http_port))
    .map_err(|e| log::error!("Failed to bind client HTTP server: {e}"))?
    .run()
    .await
    .map_err(|e| log::error!("Failed to run client HTTP server: {e}"))?)
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
                subscriptions_handler(req, payload, s.into_inner(), cfg).await
            }
            SchemaKind::SchemaMix(s) => {
                subscriptions_handler(req, payload, s.into_inner(), cfg).await
            }
            SchemaKind::SchemaDashboard(s) => {
                subscriptions_handler(req, payload, s.into_inner(), cfg).await
            }
            SchemaKind::SchemaStatistics(s) => {
                subscriptions_handler(req, payload, s.into_inner(), cfg).await
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
