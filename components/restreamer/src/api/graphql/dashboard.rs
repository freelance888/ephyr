//! Dashboard [GraphQL] API providing application usage.
//!
//! [GraphQL]: https://graphql.com

use super::Context;
use crate::{
    api::graphql,
    broadcaster::DashboardCommand,
    console_logger::ConsoleMessage,
    state::{Client, ClientId},
};
use actix_web::http::StatusCode;
use futures::{stream::BoxStream, StreamExt};
use futures_signals::signal::SignalExt;
use juniper::{graphql_object, graphql_subscription, RootNode};

/// Schema of `Dashboard` app.
pub type Schema =
    RootNode<'static, QueriesRoot, MutationsRoot, SubscriptionsRoot>;

/// Constructs and returns new [`Schema`], ready for use.
#[inline]
#[must_use]
pub fn schema() -> Schema {
    Schema::new(QueriesRoot, MutationsRoot, SubscriptionsRoot)
}

/// Root of all [GraphQL queries][1] in the [`Schema`].
///
/// [1]: https://spec.graphql.org/June2018/#sec-Root-Operation-Types
#[derive(Clone, Copy, Debug)]
pub struct QueriesRoot;

#[graphql_object(name = "Query", context = Context)]
impl QueriesRoot {
    fn statistics(context: &Context) -> Vec<Client> {
        context.state().clients.lock_mut().clone()
    }
}

/// Root of all [GraphQL mutations][1] in the [`Schema`].
///
/// [1]: https://spec.graphql.org/June2018/#sec-Root-Operation-Types
#[derive(Clone, Copy, Debug)]
pub struct MutationsRoot;

#[graphql_object(name = "Mutation", context = Context)]
impl MutationsRoot {
    /// Add a new [`Client`]
    ///
    /// Returns [`graphql::Error`] if there is already [`Client`] in this
    /// [`State`].
    fn add_client(
        #[graphql(description = "Url of remote client")] client_id: ClientId,
        context: &Context,
    ) -> Result<Option<bool>, graphql::Error> {
        match context.state().add_client(&client_id) {
            Ok(()) => Ok(Some(true)),
            Err(e) => Err(graphql::Error::new("DUPLICATE_CLIENT")
                .status(StatusCode::CONFLICT)
                .message(&e)),
        }
    }

    /// Remove [`Client`]
    ///
    /// Returns [`None`] if there is no [`Client`] in this
    /// [`State`].
    fn remove_client(
        #[graphql(description = "Url of remote client")] client_id: ClientId,
        context: &Context,
    ) -> Result<Option<bool>, graphql::Error> {
        match context.state().remove_client(&client_id) {
            Some(()) => Ok(Some(true)),
            None => Ok(None),
        }
    }

    /// Remove all [`Client`]s from dashboard
    fn remove_all_clients(
        context: &Context,
    ) -> Result<Option<bool>, graphql::Error> {
        context.state().clients.lock_mut().clear();

        Ok(Some(true))
    }

    /// Remove all messages from console
    fn console_clear(
        context: &Context,
    ) -> Result<Option<bool>, graphql::Error> {
        context.state().console_log.lock_mut().clear();

        Ok(Some(true))
    }

    /// Start playing specific file on any of client
    fn broadcast_play_file(
        #[graphql(description = "Prefix of the file name to search")]
        name_prefix: String,
        context: &Context,
    ) -> Result<Option<bool>, graphql::Error> {
        let mut commands = context.state().dashboard_commands.lock_mut();
        commands.push(DashboardCommand::StartPlayingFile(name_prefix));

        Ok(Some(true))
    }

    /// Stop playing specific file on any of client
    fn broadcast_stop_playing_file(
        #[graphql(description = "Prefix of the file name to search")]
        name_prefix: String,
        context: &Context,
    ) -> Result<Option<bool>, graphql::Error> {
        let mut commands = context.state().dashboard_commands.lock_mut();
        commands.push(DashboardCommand::StopPlayingFile(name_prefix));

        Ok(Some(true))
    }

    /// Enables all `Output`s for all clients.
    fn enable_all_outputs_for_clients(
        context: &Context,
    ) -> Result<Option<bool>, graphql::Error> {
        let mut commands = context.state().dashboard_commands.lock_mut();
        commands.push(DashboardCommand::EnableAllOutputs());

        Ok(Some(true))
    }

    /// Disables all `Output`s for all clients.
    fn disable_all_outputs_for_clients(
        context: &Context,
    ) -> Result<Option<bool>, graphql::Error> {
        let mut commands = context.state().dashboard_commands.lock_mut();
        commands.push(DashboardCommand::DisableAllOutputs());

        Ok(Some(true))
    }
}

/// Root of all [GraphQL subscriptions][1] in the [`Schema`].
///
/// [1]: https://spec.graphql.org/June2018/#sec-Root-Operation-Types
#[derive(Clone, Copy, Debug)]
pub struct SubscriptionsRoot;

#[graphql_subscription(name = "Subscription", context = Context)]
impl SubscriptionsRoot {
    async fn statistics(context: &Context) -> BoxStream<'static, Vec<Client>> {
        context
            .state()
            .clients
            .signal_cloned()
            .dedupe_cloned()
            .to_stream()
            .boxed()
    }

    /// Subscribes to updates of `console_log` messages.
    async fn console_log(
        context: &Context,
    ) -> BoxStream<'static, Vec<ConsoleMessage>> {
        context
            .state()
            .console_log
            .signal_cloned()
            .dedupe_cloned()
            .to_stream()
            .boxed()
    }
}
