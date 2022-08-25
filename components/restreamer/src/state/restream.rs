use std::{borrow::Cow, mem};

use anyhow::anyhow;
use derive_more::{Deref, Display, From, Into};
use juniper::{GraphQLObject, GraphQLScalar};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{
    spec,
    state::{Input, Label, Output},
};

/// Re-stream of a live stream from one `Input` to many `Output`s.
#[derive(
    Clone, Debug, Deserialize, Eq, GraphQLObject, PartialEq, Serialize,
)]
pub struct Restream {
    /// Unique ID of this `Input`.
    ///
    /// Once assigned, it never changes.
    pub id: RestreamId,

    /// Unique key of this `Restream` identifying it, and used to form its
    /// endpoints URLs.
    pub key: RestreamKey,

    /// Optional label of this `Restream`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<Label>,

    /// `Input` that a live stream is received from.
    pub input: Input,

    /// `Output`s that a live stream is re-streamed to.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<Output>,
}

impl Restream {
    /// Creates a new [`Restream`] out of the given [`spec::v1::Restream`].
    #[inline]
    #[must_use]
    pub fn new(spec: spec::v1::Restream) -> Self {
        Self {
            id: RestreamId::random(),
            key: spec.key,
            label: spec.label,
            input: Input::new(spec.input),
            outputs: spec.outputs.into_iter().map(Output::new).collect(),
        }
    }

    /// Applies the given [`spec::v1::Restream`] to this [`Restream`].
    ///
    /// If `replace` is `true` then all the [`Restream::outputs`] will be
    /// replaced with new ones, otherwise new ones will be merged with already
    /// existing [`Restream::outputs`].
    pub fn apply(&mut self, new: spec::v1::Restream, replace: bool) {
        self.key = new.key;
        self.label = new.label;
        self.input.apply(new.input);
        if replace {
            let mut olds = mem::replace(
                &mut self.outputs,
                Vec::with_capacity(new.outputs.len()),
            );
            for new in new.outputs {
                if let Some(mut old) = olds
                    .iter()
                    .enumerate()
                    .find_map(|(n, o)| (o.dst == new.dst).then(|| n))
                    .map(|n| olds.swap_remove(n))
                {
                    old.apply(new, replace);
                    self.outputs.push(old);
                } else {
                    self.outputs.push(Output::new(new));
                }
            }
        } else {
            for new in new.outputs {
                if let Some(old) =
                    self.outputs.iter_mut().find(|o| o.dst == new.dst)
                {
                    old.apply(new, replace);
                } else {
                    self.outputs.push(Output::new(new));
                }
            }
        }
    }

    /// Exports this [`Restream`] as a [`spec::v1::Restream`].
    #[inline]
    #[must_use]
    pub fn export(&self) -> spec::v1::Restream {
        spec::v1::Restream {
            id: Some(self.id),
            key: self.key.clone(),
            label: self.label.clone(),
            input: self.input.export(),
            outputs: self.outputs.iter().map(Output::export).collect(),
        }
    }

    /// Returns an URL on a local [SRS] server of the endpoint representing a
    /// main [`Input`] in this [`Restream`].
    ///
    /// # Errors
    ///
    /// If not found any RTMP [`Input`] endpoint
    ///
    /// [SRS]: https://github.com/ossrs/srs
    pub fn main_input_rtmp_endpoint_url(&self) -> anyhow::Result<Url> {
        match self.input.endpoints.iter().find(|e| e.is_rtmp()) {
            Some(main) => Ok(main.kind.rtmp_url(&self.key, &self.input.key)),
            None => Err(anyhow!("Not found any RTMP endpoint")),
        }
    }
}

/// ID of a `Restream`.
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
pub struct RestreamId(Uuid);

impl RestreamId {
    /// Generates a new random [`RestreamId`].
    #[inline]
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Key of a [`Restream`] identifying it, and used to form its endpoints URLs.
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
pub struct RestreamKey(String);

impl RestreamKey {
    /// Creates a new [`RestreamKey`] if the given value meets its invariants.
    #[must_use]
    pub fn new<'s, S: Into<Cow<'s, str>>>(val: S) -> Option<Self> {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("^[a-z0-9_-]{1,20}$").unwrap());

        let val = val.into();
        (!val.is_empty() && REGEX.is_match(&val))
            .then(|| Self(val.into_owned()))
    }
}

impl<'de> Deserialize<'de> for RestreamKey {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(<Cow<'_, str>>::deserialize(deserializer)?)
            .ok_or_else(|| D::Error::custom("Not a valid Restream.key"))
    }
}

impl PartialEq<str> for RestreamKey {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}
