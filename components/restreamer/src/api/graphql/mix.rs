//! Client [GraphQL] API providing application usage.
//!
//! [GraphQL]: https://graphql.com

use futures::stream::BoxStream;
use futures_signals::signal::SignalExt as _;
use juniper::{graphql_object, graphql_subscription, RootNode};

use crate::{
    state::{
        Delay, MixinId, Output, OutputId, RestreamId, Volume
    },
};

use super::Context;

/// Schema of [`api::graphql::mix`] for single output app.
///
/// [`api::graphql::mix`]: graphql::mix
pub type Schema =
RootNode<'static, QueriesRoot, MutationsRoot, SubscriptionsRoot>;

/// Constructs and returns new [`Schema`], ready for use.
#[inline]
#[must_use]
pub fn schema() -> Schema {
    Schema::new(QueriesRoot, MutationsRoot, SubscriptionsRoot)
}

/// Root of all [GraphQL mutations][1] in the [`Schema`].
///
/// [1]: https://spec.graphql.org/June2018/#sec-Root-Operation-Types
#[derive(Clone, Copy, Debug)]
pub struct MutationsRoot;

#[graphql_object(name = "Mutation", context = Context)]
impl MutationsRoot {

    /// Tunes a `Volume` rate of the specified `Output` or one of its `Mixin`s.
    fn tune_volume(
        restream_id: RestreamId,
        output_id: OutputId,
        mixin_id: Option<MixinId>,
        volume: Volume,
        context: &Context,
    ) -> Option<bool> {
        context
            .state()
            .tune_volume(restream_id, output_id, mixin_id, volume)
    }

    /// Tunes a `Delay` of the specified `Mixin` before mix it into its
    fn tune_delay(
        restream_id: RestreamId,
        output_id: OutputId,
        mixin_id: MixinId,
        delay: Delay,
        context: &Context,
    ) -> Option<bool> {
        context
            .state()
            .tune_delay(restream_id, output_id, mixin_id, delay)
    }
}

/// Root of all [GraphQL queries][1] in the [`Schema`].
///
/// [1]: https://spec.graphql.org/June2018/#sec-Root-Operation-Types
#[derive(Clone, Copy, Debug)]
pub struct QueriesRoot;

#[graphql_object(name = "Query", context = Context)]
impl QueriesRoot {

    /// Returns output for specified restream by output_id.
    fn output(restream_id: RestreamId, output_id: OutputId, context: &Context) -> Option<Output> {
        context.state().get_output(restream_id, output_id)
    }
}

/// Root of all [GraphQL subscriptions][1] in the [`Schema`].
///
/// [1]: https://spec.graphql.org/June2018/#sec-Root-Operation-Types
#[derive(Clone, Copy, Debug)]
pub struct SubscriptionsRoot;

#[graphql_subscription(name = "Subscription", context = Context)]
impl SubscriptionsRoot {
    /// Returns output for specified restream by output_id.
    async fn output(restream_id: RestreamId, output_id: OutputId, context: &Context) -> BoxStream<'static, Option<Output>> {
        context.state().restreams
            .signal_cloned()
            .dedupe_cloned()
            .map(move |restreams| {
                Some(restreams
                    .into_iter()
                    .find(|r| r.id == restream_id)?
                    .outputs
                    .into_iter()
                    .find(|o| o.id == output_id)?)
            })
            .to_stream()
            .boxed()
    }
}
