//! Callback HTTP server responding to [SRS] HTTP callbacks.
//!
//! [SRS]: https://github.com/ossrs/srs
use std::panic::AssertUnwindSafe;

use actix_web::{
    error, middleware, post, web, web::Data, App, Error, HttpServer,
};
use futures::{FutureExt, TryFutureExt};
use tap::Tap;

use crate::{
    cli::{Failure, Opts},
    display_panic,
    state::{EndpointId, Input, InputEndpointKind, InputSrc, State, Status},
    stream_probe::stream_probe,
};
use ephyr_log::{
    tracing,
    tracing::{instrument, Instrument},
};
use srs_client::{SrsCallbackEvent, SrsCallbackReq};

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
#[instrument(name = "srs_callback", skip_all,
fields(% cfg.callback_http_port, % cfg.callback_http_ip)
)]
pub async fn run(cfg: &Opts, state: State) -> Result<(), Failure> {
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .wrap(middleware::Logger::default())
            .service(on_callback)
    })
    .bind((cfg.callback_http_ip, cfg.callback_http_port))
    .map_err(|e| tracing::error!(%e, "Failed to bind callback HTTP server"))?
    .run()
    .in_current_span()
    .await
    .map_err(|e| {
        tracing::error!(%e, "Failed to run callback HTTP server");
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
#[instrument(name = "srs_callback", skip_all,
fields(
action = % req.action,
client = % req.ip,
input = & req.app_stream())
)]
async fn on_callback(
    req: web::Json<SrsCallbackReq>,
    state: Data<State>,
) -> Result<&'static str, Error> {
    match req.action {
        SrsCallbackEvent::OnConnect => on_connect(&req, &state),
        SrsCallbackEvent::OnPublish => on_start(&req, &state, true),
        SrsCallbackEvent::OnUnpublish => on_stop(&req, &state, true),
        SrsCallbackEvent::OnPlay => on_start(&req, &state, false),
        SrsCallbackEvent::OnStop => on_stop(&req, &state, false),
        SrsCallbackEvent::OnHls => on_hls(&req, &state),
        SrsCallbackEvent::OnDvr => Ok(()),
    }
    .map(|()| "0")
}

/// Handles [`SrsCallbackEvent::OnConnect`].
///
/// Only checks whether the appropriate [`state::Restream`] exists and its
/// [`Input`] is enabled.
///
/// # Errors
///
/// If [`SrsCallbackReq::app`] matches no existing [`state::Restream`].
///
/// [`state::Restream`]: crate::state::Restream
#[instrument(err, skip_all)]
fn on_connect(req: &SrsCallbackReq, state: &State) -> Result<(), Error> {
    state
        .restreams
        .get_cloned()
        .iter()
        .find(|r| r.input.enabled && r.key == *req.app)
        .tap(|r| {
            if let Some(r) = r {
                tracing::info!(actor = %r.id, "Connection established");
            }
        })
        .ok_or_else(|| {
            error::ErrorNotFound(format!("App `{}` doesn't exist", req.app))
        })
        .map(|_| ())
}

/// Handles [`SrsCallbackEvent::OnPublish`] and [`SrsCallbackEvent::OnPlay`].
///
/// Updates the appropriate [`state::Restream`]'s [`InputEndpoint`] to
/// [`Status::Online`] (if [`SrsCallbackEvent::OnPublish`]) and remembers the
/// connected [SRS] client.
///
/// # Errors
///
/// - If [`SrsCallbackReq::vhost`], [`SrsCallbackReq::app`] or
///   [`SrsCallbackReq::stream`] matches no existing enabled
///   [`InputEndpoint`].
/// - If [`InputEndpoint`] is not allowed to be published by external
///   client.
///
/// [`InputEndpoint`]: crate::state::InputEndpoint
/// [`state::Restream`]: crate::state::Restream
///
/// [SRS]: https://github.com/ossrs/srs
#[instrument(err, skip_all)]
fn on_start(
    req: &SrsCallbackReq,
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
            return input.enabled.then_some(input);
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
        .ok_or_else(|| {
            error::ErrorNotFound(format!("App `{}` doesn't exist", req.app))
        })?;

    let input = lookup_input(&mut restream.input, stream).ok_or_else(|| {
        error::ErrorNotFound(format!("Stream `{stream}` doesn't exist"))
    })?;

    let endpoint = input
        .endpoints
        .iter_mut()
        .find(|e| e.kind == kind)
        .ok_or_else(|| error::ErrorForbidden("Such `vhost` is not allowed"))?;

    if publishing {
        if !req.ip.is_loopback() && (input.src.is_some() || !endpoint.is_rtmp())
        {
            return Err(error::ErrorNotFound(format!(
                "Stream `{stream}` doesn't exist"
            )));
        }

        let publisher_id = match endpoint.srs_publisher_id.clone() {
            Some(id) => id.get_value(),
            None => None,
        };

        if publisher_id != Some(req.client_id.clone()) {
            endpoint.srs_publisher_id = Some(req.client_id.clone().into());
        }

        endpoint.status = Status::Online;

        let url = InputEndpointKind::get_rtmp_url(
            &restream.key,
            &input.key,
            InputEndpointKind::Rtmp,
        );
        if !url.to_string().contains("playback") {
            endpoint.stream_stat = None;
            update_stream_info(endpoint.id, url.to_string(), state.clone());
        }
        tracing::info!(actor = %endpoint.id, "Publishing started");
    } else {
        // `srs::ClientId` kicks the client when `Drop`ped, so we should be
        // careful here to not accidentally kick the client by creating a
        // temporary binding.
        if !endpoint.srs_player_ids.contains(&req.client_id) {
            _ = endpoint.srs_player_ids.insert(req.client_id.clone().into());
        }
        tracing::info!(actor = %endpoint.id, "Playing stopped");
    }
    Ok(())
}

/// Handles [`SrsCallbackEvent::OnUnpublish`].
///
/// Updates the appropriate [`state::Restream`]'s [`InputEndpoint`] to
/// [`Status::Offline`].
///
/// # Errors
///
/// If [`SrsCallbackReq::vhost`], [`SrsCallbackReq::app`] or
/// [`SrsCallbackReq::stream`] matches no existing [`InputEndpoint`].
///
/// [`InputEndpoint`]: crate::state::InputEndpoint
/// [`state::Restream`]: crate::state::Restream
#[instrument(err, skip_all)]
fn on_stop(
    req: &SrsCallbackReq,
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
        .ok_or_else(|| {
            error::ErrorNotFound(format!("App {} doesn't exist", req.app))
        })?;

    let input = lookup_input(&mut restream.input, stream).ok_or_else(|| {
        error::ErrorNotFound(format!("Stream `{stream}` doesn't exist",))
    })?;

    let endpoint = input
        .endpoints
        .iter_mut()
        .find(|e| e.kind == kind)
        .ok_or_else(|| {
            error::ErrorForbidden(format!(
                "Vhost `{}` is not allowed",
                req.vhost
            ))
        })?;

    if publishing {
        endpoint.srs_publisher_id = None;
        endpoint.status = Status::Offline;
        tracing::info!(actor = %endpoint.id, "Publishing stopped");
    } else {
        _ = endpoint.srs_player_ids.remove(&req.client_id);
        tracing::info!(actor = %endpoint.id, "Playing stopped");
    }
    Ok(())
}

/// Handles [`SrsCallbackEvent::OnHls`].
///
/// Checks whether the appropriate [`state::Restream`] with an
/// [`InputEndpointKind::Hls`] exists and its [`Input`] is enabled.
///
/// # Errors
///
/// If [`SrsCallbackReq::vhost`], [`SrsCallbackReq::app`] or
/// [`SrsCallbackReq::stream`] matches no existing [`InputEndpoint`]
/// of [`InputEndpointKind::Hls`].
///
/// [`InputEndpoint`]: crate::state::InputEndpoint
/// [`state::Restream`]: crate::state::Restream
#[instrument(err, skip_all)]
fn on_hls(req: &SrsCallbackReq, state: &State) -> Result<(), Error> {
    /// Traverses the given [`Input`] and all its [`Input::srcs`] looking
    /// for the one matching the specified `stream` and being enabled.
    #[must_use]
    fn lookup_input<'i>(
        input: &'i mut Input,
        stream: &str,
    ) -> Option<&'i mut Input> {
        if input.key == *stream {
            return input.enabled.then_some(input);
        }
        if let Some(InputSrc::Failover(s)) = input.src.as_mut() {
            s.inputs.iter_mut().find_map(|i| lookup_input(i, stream))
        } else {
            None
        }
    }

    let stream = req.stream.as_deref().unwrap_or_default();
    let kind = (req.vhost.as_str() == "hls")
        .then_some(InputEndpointKind::Hls)
        .ok_or_else(|| {
            error::ErrorForbidden(format!(
                "Vhost `{}` is not allowed",
                req.vhost
            ))
        })?;

    let mut restreams = state.restreams.lock_mut();
    let restream = restreams
        .iter_mut()
        .find(|r| r.input.enabled && r.key == *req.app)
        .ok_or_else(|| {
            error::ErrorNotFound(format!("App `{}` doesn't exist", req.app))
        })?;

    let endpoint = lookup_input(&mut restream.input, stream)
        .ok_or_else(|| {
            error::ErrorNotFound(format!("Stream `{stream}` doesn't exist"))
        })?
        .endpoints
        .iter_mut()
        .find(|e| e.kind == kind)
        .ok_or_else(|| {
            error::ErrorNotFound(format!("Stream `{stream}` doesn't exist"))
        })?;

    if endpoint.status != Status::Online {
        return Err(error::ErrorImATeapot("Not ready to serve"));
    }

    // `srs::ClientId` kicks the client when `Drop`ped, so we should be
    // careful here to not accidentally kick the client by creating a
    // temporary binding.
    if !endpoint.srs_player_ids.contains(&req.client_id) {
        _ = endpoint.srs_player_ids.insert(req.client_id.clone().into());
    }
    Ok(())
}

#[instrument(skip_all)]
fn update_stream_info(id: EndpointId, url: String, state: State) {
    drop(
        tokio::spawn(
            AssertUnwindSafe(
                async move {
                    let result = stream_probe(url).await;
                    state
                        .set_stream_info(id, result)
                        .unwrap_or_else(|e| tracing::error!(%e));
                }
                .in_current_span(),
            )
            .catch_unwind()
            .map_err(move |p| {
                tracing::error!(
                    e = display_panic(&p),
                    "Can not fetch stream info",
                );
            }),
        )
        .in_current_span(),
    );
}
