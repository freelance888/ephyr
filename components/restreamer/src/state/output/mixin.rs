//! Additional source for an `Output` to be mixed with before re-streaming to
//! the destination.

use crate::{
    serde::is_false,
    spec,
    state::{output::Volume, Status},
};
use derive_more::{Deref, Display, From, Into};
use juniper::{
    GraphQLObject, GraphQLScalar, InputValue, ParseScalarResult,
    ParseScalarValue, ScalarToken, ScalarValue, Value,
};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use std::{convert::TryInto, path::Path, time::Duration};
use url::Url;
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

/// [`Url`] of a [`Mixin::src`].
///
/// Only the following URLs are allowed at the moment:
/// - [TeamSpeak] URL (starting with `ts://` scheme and having a host);
/// - [MP3] HTTP URL (starting with `http://` or `https://` scheme, having a
///   host and `.mp3` extension in its path).
///
/// [MP3]: https://en.wikipedia.org/wiki/MP3
/// [TeamSpeak]: https://teamspeak.com
#[derive(
    Clone,
    Debug,
    Deref,
    Display,
    Eq,
    Hash,
    Into,
    PartialEq,
    Serialize,
    GraphQLScalar,
)]
#[graphql(transparent)]
pub struct MixinSrcUrl(Url);

impl MixinSrcUrl {
    /// Creates a new [`MixinSrcUrl`] if the given [`Url`] is suitable for that.
    ///
    /// # Errors
    ///
    /// Returns the given [`Url`] back if it doesn't represent a valid
    /// [`MixinSrcUrl`].
    #[inline]
    pub fn new(url: Url) -> Result<Self, Url> {
        if Self::validate(&url) {
            Ok(Self(url))
        } else {
            Err(url)
        }
    }

    /// Validates the given [`Url`] to represent a valid [`MixinSrcUrl`].
    #[must_use]
    pub fn validate(url: &Url) -> bool {
        url.has_host()
            && match url.scheme() {
                "ts" => true,
                "http" | "https" => {
                    Path::new(url.path()).extension() == Some("mp3".as_ref())
                }
                _ => false,
            }
    }
}

impl<'de> Deserialize<'de> for MixinSrcUrl {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(Url::deserialize(deserializer)?).map_err(|url| {
            D::Error::custom(format!("Not a valid Mixin.src URL: {url}"))
        })
    }
}

/// Delay of a [`Mixin`] being mixed with an [`Output`].
///
/// [`Mixin`]: crate::state::Mixin
/// [`Output`]: crate::state::Output
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    GraphQLScalar,
)]
#[graphql(with = Self)]
pub struct Delay(#[serde(with = "serde_humantime")] Duration);

impl Delay {
    /// Creates a new [`Delay`] out of the given milliseconds.
    #[inline]
    #[must_use]
    pub fn from_millis<N: TryInto<u64>>(millis: N) -> Option<Self> {
        millis
            .try_into()
            .ok()
            .map(|m| Self(Duration::from_millis(m)))
    }

    /// Returns milliseconds of this [`Delay`].
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn as_millis(&self) -> i32 {
        self.0.as_millis().try_into().unwrap()
    }

    /// Indicates whether this [`Delay`] introduces no actual delay.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == Duration::default()
    }

    #[allow(clippy::wrong_self_convention)]
    fn to_output<S: ScalarValue>(&self) -> Value<S> {
        Value::scalar(self.as_millis())
    }

    fn from_input<S>(v: &InputValue<S>) -> Result<Self, String>
    where
        S: ScalarValue,
    {
        let v = v
            .as_scalar()
            .and_then(ScalarValue::as_int)
            .and_then(Self::from_millis);
        match v {
            None => Err("test".to_string()),
            Some(d) => Ok(d),
        }
    }

    fn parse_token<S>(value: ScalarToken<'_>) -> ParseScalarResult<S>
    where
        S: ScalarValue,
    {
        <String as ParseScalarValue<S>>::from_str(value)
    }
}
