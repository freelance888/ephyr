//! Dashboard [GraphQL] API providing application usage.
//!
//! [GraphQL]: https://graphql.com

use juniper::{graphql_object, RootNode, EmptySubscription};
use super::Context;
use crate::state::Client;
use actix_web::http::StatusCode;
use crate::api::graphql;

/// Schema of `Dashboard` app.
pub type Schema =
RootNode<'static, QueriesRoot, MutationsRoot, EmptySubscription<Context>>;

/// Constructs and returns new [`Schema`], ready for use.
#[inline]
#[must_use]
pub fn schema() -> Schema {
    Schema::new(QueriesRoot, MutationsRoot, EmptySubscription::new())
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
    #[graphql(arguments(ip_address(
        description = "IP address of remote host."
    )))]
    fn add_client(host: String, context: &Context) -> Result<Option<bool>, graphql::Error> {
        match context.state().add_client(host) {
            Ok(_) => Ok(Some(true)),
            Err(e) => Err(graphql::Error::new("DUPLICATE_CLIENT_IP")
                .status(StatusCode::CONFLICT)
                .message(&e))
        }
    }

    /// Remove [`Client`] by ip
    ///
    /// Returns [`None`] if there is no [`Client`] in this
    /// [`State`].
    #[graphql(arguments(ip_address(
        description = "IP address of remote host."
    )))]
    fn remove_client(host: String, context: &Context) -> Result<Option<bool>, graphql::Error> {
        match context.state().remove_client(host) {
            Some(_) => Ok(Some(true)),
            None => Ok(None)
        }
    }
}
