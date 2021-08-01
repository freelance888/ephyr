//! Client [GraphQL] API providing application usage.
//!
//! [GraphQL]: https://graphql.com

use futures::stream::BoxStream;
use futures_signals::signal::SignalExt as _;
use juniper::{graphql_object, graphql_subscription, RootNode};
use url::Url;

use crate::{
    api::graphql,
    Spec,
    state::{
        Delay, InputEndpointKind, InputId, InputKey, InputSrcUrl, Label,
        MixinId, MixinSrcUrl, Output, OutputDstUrl, OutputId, PasswordKind,
        Restream, RestreamId, RestreamKey, Volume
    },
};

use super::Context;

/// Schema of [`api::graphql::mix`] for single output app.
///
/// [`api::graphql::mix`]: graphql::mix
pub type SchemaMix =
RootNode<'static, QueriesRootMix, MutationsRootMix, SubscriptionsRootMix>;

/// Constructs and returns new [`SchemaMix`], ready for use.
#[inline]
#[must_use]
pub fn schema_mix() -> SchemaMix {
    SchemaOut::new(QueriesRootMix, MutationsRootMix, SubscriptionsRootMix)
}

/// Root of all [GraphQL mutations][1] in the [`SchemaMix`].
///
/// [1]: https://spec.graphql.org/June2018/#sec-Root-Operation-Types
#[derive(Clone, Copy, Debug)]
pub struct MutationsRootMix;

#[graphql_object(name = "Mutation", context = Context)]
impl MutationsRootMix {

    /// Tunes a `Volume` rate of the specified `Output` or one of its `Mixin`s.
    ///
    /// ### Result
    ///
    /// Returns `true` if a `Volume` rate has been changed, `false` if it has
    /// the same value already, or `null` if the specified `Output` or `Mixin`
    /// doesn't exist.
    #[graphql(arguments(
        restream_id(description = "ID of the `Restream` to tune the \
                                   `Output` in."),
        output_id(description = "ID of the tuned `Output`."),
        mixin_id(description = "Optional ID of the tuned `Mixin`.\
                                \n\n\
                                If set, then tunes the `Mixin` rather than \
                                the `Output`."),
        volume(description = "Volume rate in percents to be set."),
    ))]
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
    /// `Output`.
    ///
    /// ### Result
    ///
    /// Returns `true` if a `Delay` has been changed, `false` if it has the same
    /// value already, or `null` if the specified `Output` or `Mixin` doesn't
    /// exist.
    #[graphql(arguments(
        restream_id(description = "ID of the `Restream` to tune the the \
                                   `Mixin` in."),
        output_id(description = "ID of the `Output` of the tuned `Mixin`."),
        mixin_id(description = "ID of the tuned `Mixin`."),
        delay(description = "Number of milliseconds to delay the `Mixin` \
                             before mix it into its `Output`."),
    ))]
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

/// Root of all [GraphQL queries][1] in the [`SchemaMix`].
///
/// [1]: https://spec.graphql.org/June2018/#sec-Root-Operation-Types
#[derive(Clone, Copy, Debug)]
pub struct QueriesRootMix;

#[graphql_object(name = "Query", context = Context)]
impl QueriesRootMix {

    /// Returns output for specified restream by output_id.
    fn output(restream_id: RestreamId, output_id: OutputId, context: &Context) -> Output {
        context.state().get_output(restream_id, output_id)
    }
}

/// Root of all [GraphQL subscriptions][1] in the [`SchemaMix`].
///
/// [1]: https://spec.graphql.org/June2018/#sec-Root-Operation-Types
#[derive(Clone, Copy, Debug)]
pub struct SubscriptionsRootMix;

#[graphql_subscription(name = "Subscription", context = Context)]
impl SubscriptionsRootMix {
    /// Subscribes to updates of title of this server.
    async fn title(context: &Context) -> BoxStream<'static, Option<String>> {
        context
            .state()
            .settings
            .signal_cloned()
            .dedupe_cloned()
            .map(move |h| h.title)
            .to_stream()
            .boxed()
    }

    /// Returns output for specified restream by output_id.
    async fn output(restream_id: RestreamId, output_id: OutputId, context: &Context) -> BoxStream<'static, Output> {
        context.state().restreams
            .signal_cloned()
            .dedupe_cloned()
            .map(move |restreams| {
                restreams
                    .into_iter()
                    .find(|r| r.id == restream_id).unwrap()
                    .outputs
                    .into_iter()
                    .find(|o| o.id == output_id).unwrap()
            })
            .to_stream()
            .boxed()
    }
}
