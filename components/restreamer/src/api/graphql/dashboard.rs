//! Dashboard [GraphQL] API providing application usage.
//!
//! [GraphQL]: https://graphql.com

use super::Context;
use crate::{
    api::graphql,
    state::{Client, ClientId},
};
use actix_web::http::StatusCode;
use futures::stream::BoxStream;
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
    #[graphql(arguments(client_id(description = "Ulr of remote client")))]
    fn add_client(
        client_id: ClientId,
        context: &Context,
    ) -> Result<Option<bool>, graphql::Error> {
        match context.state().add_client(&client_id) {
            Ok(_) => Ok(Some(true)),
            Err(e) => Err(graphql::Error::new("DUPLICATE_CLIENT")
                .status(StatusCode::CONFLICT)
                .message(&e)),
        }
    }

    /// Remove [`Client`]
    ///
    /// Returns [`None`] if there is no [`Client`] in this
    /// [`State`].
    #[graphql(arguments(client_id(description = "Ulr of remote client")))]
    fn remove_client(
        client_id: ClientId,
        context: &Context,
    ) -> Result<Option<bool>, graphql::Error> {
        match context.state().remove_client(&client_id) {
            Some(_) => Ok(Some(true)),
            None => Ok(None),
        }
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
}
