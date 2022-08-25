//! Additional source for an `Output` to be mixed with before re-streaming to
//! the destination.
mod delay;
mod mixin_src_url;
mod volume;

pub use self::{
    delay::Delay,
    mixin_src_url::MixinSrcUrl,
    volume::{Volume, VolumeLevel},
};

use crate::{serde::is_false, spec, state::Status};
use derive_more::{Display, From, Into};
use juniper::{GraphQLObject, GraphQLScalar};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Additional source for an `Output` to be mixed with before re-streaming to
/// the destination.
#[derive(
    Clone, Debug, Deserialize, Eq, GraphQLObject, PartialEq, Serialize,
)]
pub struct Mixin {
    /// Unique ID of this `Mixin`.
    ///
    /// Once assigned, it never changes.
    pub id: MixinId,

    /// URL of the source to be mixed with an `Output`.
    ///
    /// At the moment, only [TeamSpeak] is supported.
    ///
    /// [TeamSpeak]: https://teamspeak.com
    pub src: MixinSrcUrl,

    /// Volume rate of this `Mixin`'s audio tracks to mix them with.
    #[serde(default, skip_serializing_if = "Volume::is_origin")]
    pub volume: Volume,

    /// Delay that this `Mixin` should wait before being mixed with an `Output`.
    ///
    /// Very useful to fix de-synchronization issues and correct timings between
    /// a `Mixin` and its `Output`.
    #[serde(default, skip_serializing_if = "Delay::is_zero")]
    pub delay: Delay,

    /// `Status` of this `Mixin` indicating whether it provides an actual media
    /// stream to be mixed with its `Output`.
    #[serde(skip)]
    pub status: Status,

    /// Side-chain audio of `Output` with this `Mixin`.
    ///
    /// Helps to automatically control audio level of `Mixin`
    /// based on level of `Output`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub sidechain: bool,
}

impl Mixin {
    /// Creates a new [`Mixin`] out of the given [`spec::v1::Mixin`].
    #[inline]
    #[must_use]
    pub fn new(spec: spec::v1::Mixin) -> Self {
        Self {
            id: MixinId::random(),
            src: spec.src,
            volume: Volume::new(&spec.volume),
            delay: spec.delay,
            status: Status::Offline,
            sidechain: spec.sidechain,
        }
    }

    /// Applies the given [`spec::v1::Mixin`] to this [`Mixin`].
    #[inline]
    pub fn apply(&mut self, new: spec::v1::Mixin) {
        self.src = new.src;
        self.volume = Volume::new(&new.volume);
        self.delay = new.delay;
        self.sidechain = new.sidechain;
    }

    /// Exports this [`Mixin`] as a [`spec::v1::Mixin`].
    #[inline]
    #[must_use]
    pub fn export(&self) -> spec::v1::Mixin {
        spec::v1::Mixin {
            src: self.src.clone(),
            volume: self.volume.export(),
            delay: self.delay,
            sidechain: self.sidechain,
        }
    }
}

/// ID of a `Mixin`.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    Eq,
    From,
    GraphQLScalar,
    Into,
    PartialEq,
    Serialize,
)]
#[graphql(transparent)]
pub struct MixinId(Uuid);

impl MixinId {
    /// Generates a new random [`MixinId`].
    #[inline]
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}
