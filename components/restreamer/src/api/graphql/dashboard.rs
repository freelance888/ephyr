//! Dashboard [GraphQL] API providing application usage.
//!
//! [GraphQL]: https://graphql.com

use juniper::{graphql_object, RootNode, EmptySubscription};
use super::Context;
use crate::state::Client;
use actix_web::http::StatusCode;
use crate::api::graphql;
use std::net::IpAddr;
use std::str::FromStr;

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
    /// Adds a new [`Client`] by ip
    fn add_client(ip_address: String, context: &Context) -> Result<Option<bool>, graphql::Error> {

        let ip = match IpAddr::from_str(ip_address.as_str()) {
            Ok(address) => address,
            Err(e) => return Err(graphql::Error::new("STRING_IS_NOT_IP_ADDRESS")
                .status(StatusCode::BAD_REQUEST)
                .message(&e))
        };

        match context.state().add_client(ip) {
            Ok(_) => Ok(Some(true)),
            Err(e) => Err(graphql::Error::new("DUPLICATE_CLIENT_IP")
                .status(StatusCode::CONFLICT)
                .message(&e))
        }
    }
}

