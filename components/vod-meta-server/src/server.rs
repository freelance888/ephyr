//! HTTP server of [VOD] meta.
//!
//! [VOD]: https://en.wikipedia.org/wiki/Video_on_demand

use std::{
    convert::TryInto as _, panic::AssertUnwindSafe, sync::Arc, time::Duration,
};

use actix_web::{
    delete, dev::ServiceRequest, error, get, middleware, put, web, App,
    FromRequest as _, HttpServer,
};
use actix_web_httpauth::{
    extractors::bearer::{self, BearerAuth},
    middleware::HttpAuthentication,
};
use ephyr_log::log;
use futures::{sink, FutureExt as _, StreamExt as _};
use serde::Deserialize;
use tokio::time;

use crate::{
    api::{nginx, vod},
    cli,
    util::display_panic,
    vod::{
        file,
        meta::{state, State},
    },
};

/// Runs [VOD] meta HTTP server.
///
/// # Errors
///
/// If running has failed and could not be performed. The appropriate error
/// is logged.
///
/// [VOD]: https://en.wikipedia.org/wiki/Video_on_demand
#[actix_web::main]
pub async fn run(opts: cli::Opts) -> Result<(), cli::Failure> {
    let request_max_size =
        opts.request_max_size.get_bytes().try_into().map_err(|e| {
            log::error!("Maximum request size has too big value: {}", e);
        })?;

    let state = state::Manager::try_new(&opts.state).await.map_err(|e| {
        log::error!("Failed to initialize vod::meta::State: {}", e);
    })?;
    state.refresh_playlists_positions().await.map_err(|e| {
        log::error!(
            "Failed to refresh vod::meta::State initial positions: {}",
            e,
        );
    })?;

    let cache =
        Arc::new(file::cache::Manager::try_new(opts.cache_dir).map_err(
            |e| log::error!("Failed to initialize vod::file::cache: {}", e),
        )?);

    drop(tokio::spawn(refill_state_with_cache_files(
        state.clone(),
        cache.clone(),
        Duration::from_secs(10),
    )));

    drop(tokio::spawn(refresh_initial_positions(
        state.clone(),
        Duration::from_secs(60),
    )));

    let auth_token_hash = AuthTokenHash(opts.auth_token_hash);

    let _server = HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .data(cache.clone())
            .wrap(middleware::Logger::default())
            .service(produce_meta)
            .service(show_playlist)
            .service(show_state)
            .app_data(bearer::Config::default().realm("Restricted area"))
            .app_data(auth_token_hash.clone())
            .app_data(web::Json::<vod::meta::Request>::configure(|cfg| {
                cfg.limit(request_max_size).error_handler(|err, _| {
                    error::ErrorBadRequest(format!(
                        "Invalid request body: {}",
                        err,
                    ))
                })
            }))
            .service(renew_state)
            .service(renew_playlist)
            .service(delete_playlist)
    })
    .bind((opts.http_ip, opts.http_port))
    .map_err(|e| log::error!("Failed to bind web server: {}", e))?
    .run()
    .await;

    Ok(())
}

/// Responses with the [`nginx-vod-module` mapping][1] containing the playlist
/// which should be played, starting from now and on.
///
/// [1]: https://github.com/kaltura/nginx-vod-module#mapping-response-format
#[get("/{location}/{playlist}/{filename}")]
async fn produce_meta(
    state: web::Data<state::Manager>,
    path: web::Path<(String, String, String)>,
) -> Result<web::Json<nginx::vod_module::mapping::Set>, error::Error> {
    let slug = state::PlaylistSlug::new(&path.1).ok_or_else(|| {
        error::ErrorBadRequest(format!("Invalid playlist slug '{}'", path.1))
    })?;

    Ok(web::Json(
        state
            .playlist(&slug)
            .await
            .ok_or_else(|| {
                error::ErrorNotFound(format!("Unknown playlist '{}'", slug))
            })?
            .schedule_nginx_vod_module_set(None, 5),
    ))
}

/// Displays the current whole `vod-meta` server [`State`].
#[get("/")]
async fn show_state(state: web::Data<state::Manager>) -> web::Json<State> {
    web::Json(state.state().await)
}

/// Displays the requested `vod-meta` server [`state::Playlist`].
#[get("/{playlist}")]
async fn show_playlist(
    state: web::Data<state::Manager>,
    slug: web::Path<state::PlaylistSlug>,
) -> Result<web::Json<state::Playlist>, error::Error> {
    Ok(web::Json(state.playlist(&slug.0).await.ok_or_else(
        || error::ErrorNotFound(format!("Unknown playlist '{}'", slug)),
    )?))
}

/// Renews the `vod-meta` server [`State`] with the new one provided in
/// [`vod::meta::Request`].
///
/// # Authorization
///
/// __Mandatory.__ The [`vod::meta::Request`] must be authorized with
/// [Bearer HTTP token][1], which value is verified against
/// [`cli::VodMetaOpts::auth_token_hash`].
///
/// [1]: https://tools.ietf.org/html/rfc6750#section-2.1
#[put("/", wrap = "HttpAuthentication::bearer(verify_auth_token)")]
async fn renew_state(
    state: web::Data<state::Manager>,
    cache: web::Data<Arc<file::cache::Manager>>,
    req: web::Json<vod::meta::Request>,
    mode: web::Query<Mode>,
) -> Result<&'static str, error::Error> {
    let mut new = State::parse_request(req.0)
        .await
        .map_err(error::ErrorBadRequest)?;

    for playlist in new.values_mut() {
        playlist
            .fill_with_cache_files(&cache)
            .await
            .map_err(error::ErrorInternalServerError)?;
    }

    state
        .set_state(new, None, mode.0.force, mode.0.dry_run)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok("Ok")
}

/// Renews the single [`state::Playlist`] in `vod-meta` server [`State`] with
/// the new one provided in [`vod::meta::Playlist`] request.
///
/// # Idempotent
///
/// If there is no such [`state::Playlist`], then it will be created. Otherwise,
/// it will be updated.
///
/// # Authorization
///
/// __Mandatory.__ The [`vod::meta::Playlist`] request must be authorized with
/// [Bearer HTTP token][1], which value is verified against
/// [`cli::VodMetaOpts::auth_token_hash`].
///
/// [1]: https://tools.ietf.org/html/rfc6750#section-2.1
#[put("/{playlist}", wrap = "HttpAuthentication::bearer(verify_auth_token)")]
async fn renew_playlist(
    state: web::Data<state::Manager>,
    cache: web::Data<Arc<file::cache::Manager>>,
    slug: web::Path<state::PlaylistSlug>,
    req: web::Json<vod::meta::Playlist>,
    mode: web::Query<Mode>,
) -> Result<&'static str, error::Error> {
    let mut playlist = state::Playlist::parse_request(slug.0, req.0)
        .await
        .map_err(error::ErrorBadRequest)?;

    playlist
        .fill_with_cache_files(&cache)
        .await
        .map_err(error::ErrorInternalServerError)?;

    state
        .set_playlist(playlist, mode.0.force, mode.0.dry_run)
        .await
        .map_err(error::ErrorConflict)?;

    Ok("Ok")
}

/// Removes the single [`state::Playlist`] from `vod-meta` server [`State`]
/// identified by its [`state::Playlist::slug`].
///
/// # Idempotent
///
/// If there is no such [`state::Playlist`] then no-op.
///
/// # Authorization
///
/// __Mandatory.__ The request must be authorized with [Bearer HTTP token][1],
/// which value is verified against [`cli::VodMetaOpts::auth_token_hash`].
///
/// [1]: https://tools.ietf.org/html/rfc6750#section-2.1
#[delete("/{playlist}", wrap = "HttpAuthentication::bearer(verify_auth_token)")]
async fn delete_playlist(
    state: web::Data<state::Manager>,
    slug: web::Path<state::PlaylistSlug>,
) -> Result<&'static str, error::Error> {
    state
        .delete_playlist(&slug.0)
        .await
        .map_err(error::ErrorConflict)?;
    Ok("Ok")
}

/// Runs job, which periodically (with the given `period`) refills the given
/// `state` with information about files available in the given `cache`.
async fn refill_state_with_cache_files(
    state: state::Manager,
    cache: Arc<file::cache::Manager>,
    period: Duration,
) {
    async fn refill(
        state: state::Manager,
        cache: Arc<file::cache::Manager>,
    ) -> Result<(), anyhow::Error> {
        let (mut curr, ver) = state.state_and_version().await;
        for playlist in curr.values_mut() {
            playlist.fill_with_cache_files(&cache).await?;
        }
        state.set_state(curr, Some(ver), true, false).await?;
        Ok(())
    }

    let _ = time::interval(period)
        .then(move |_| {
            log::debug!(
                "Refilling vod::meta::State with vod::file::cache triggered",
            );
            let (state, cache) = (state.clone(), cache.clone());
            async move {
                AssertUnwindSafe(refill(state, cache))
                    .catch_unwind()
                    .await
                    .map_err(|p| {
                        log::error!(
                            "Panicked while refilling vod::meta::State with \
                             vod::file::cache: {}",
                            display_panic(&p),
                        );
                    })?
                    .map_err(|e| {
                        log::error!(
                            "Failed to refill vod::meta::State with \
                             vod::file::cache: {}",
                            e,
                        );
                    })
            }
        })
        .map(Ok)
        .forward(sink::drain())
        .await;
}

/// Runs job, which periodically (with the given `period`) refreshes
/// [`state::Playlist::initial`] positions in the given `state`.
async fn refresh_initial_positions(state: state::Manager, period: Duration) {
    let _ = time::interval(period)
        .then(move |_| {
            log::debug!(
                "Refreshing vod::meta::state::Playlist::initial positions",
            );
            let state = state.clone();
            async move {
                AssertUnwindSafe(state.refresh_playlists_positions())
                    .catch_unwind()
                    .await
                    .map_err(|p| {
                        log::error!(
                            "Panicked while refreshing vod::meta::State \
                             initial positions: {}",
                            display_panic(&p),
                        );
                    })
                    .unwrap_err();
            }
        })
        .map(Ok)
        .forward(sink::drain())
        .await;
}

/// Helper wrapper for extracting [`cli::VodMetaOpts::auth_token_hash`] in
/// [`actix_web`] handlers.
#[derive(Clone, Debug)]
struct AuthTokenHash(String);

async fn verify_auth_token(
    req: ServiceRequest,
    auth: BearerAuth,
) -> Result<ServiceRequest, error::Error> {
    let token_hash = req.app_data::<AuthTokenHash>().unwrap().0.clone();

    let is_ok = web::block(move || {
        argon2::verify_encoded(&token_hash, auth.token().as_bytes())
    })
    .await
    .map_err(error::ErrorInternalServerError)?;
    if !is_ok {
        return Err(error::ErrorUnauthorized("Invalid Bearer token provided"));
    }

    Ok(req)
}

/// Parameters configuring the mode for applying new [`State`].
#[derive(Clone, Copy, Debug, Deserialize)]
struct Mode {
    /// Indicator whether [`state::Playlist`]s should be updated regardless
    /// its broken playbacks.
    #[serde(default)]
    force: bool,

    /// Indicator whether [`state::Playlist`]s should be checked and verified
    /// without applying any real changes to existing [`State`].
    #[serde(default)]
    dry_run: bool,
}
