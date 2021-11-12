//! Statistics [GraphQL] API providing application usage.
//!
//! [GraphQL]: https://graphql.com

use juniper::{graphql_object, EmptyMutation, EmptySubscription, RootNode};

use super::Context;
use crate::state::ClientStatistics;
use std::fmt::Debug;

/// Schema of `Statistics` module.
pub type Schema = RootNode<
    'static,
    QueriesRoot,
    EmptyMutation<Context>,
    EmptySubscription<Context>,
>;

/// Constructs and returns new [`Schema`], ready for use.
#[inline]
#[must_use]
pub fn schema() -> Schema {
    Schema::new(QueriesRoot, EmptyMutation::new(), EmptySubscription::new())
}

/// Root of all [GraphQL queries][1] in the [`Schema`].
///
/// [1]: https://spec.graphql.org/June2018/#sec-Root-Operation-Types
#[derive(Clone, Copy, Debug)]
pub struct QueriesRoot;

#[graphql_object(name = "Query", context = Context)]
impl QueriesRoot {
    fn statistics(context: &Context) -> ClientStatistics {
        let public_ip = context.config().public_host.clone().unwrap();
        context.state().get_statistics(public_ip)
    }
}
