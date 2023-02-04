mod input_endpoint;
mod input_src;

pub use self::{
    input_endpoint::{EndpointId, InputEndpoint, InputEndpointKind},
    input_src::{FailoverInputSrc, InputSrc, InputSrcUrl, RemoteInputSrc},
};

use std::{borrow::Cow, mem};

use derive_more::{Deref, Display, From, Into};
use juniper::{GraphQLObject, GraphQLScalar};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use crate::{serde::is_false, spec, state::Status};

/// Upstream source that a `Restream` receives a live stream from.
#[derive(
    Clone, Debug, Deserialize, Eq, GraphQLObject, PartialEq, Serialize,
)]
pub struct Input {
    /// Unique ID of this `Input`.
    ///
    /// Once assigned, it never changes.
    pub id: InputId,

    /// Key of this `Input` to expose its `InputEndpoint`s with for accepting
    /// and serving a live stream.
    pub key: InputKey,

    /// Endpoints of this `Input` serving a live stream for `Output`s and
    /// clients.
    pub endpoints: Vec<InputEndpoint>,

    /// Source to pull a live stream from.
    ///
    /// If specified, then this `Input` will pull a live stream from it (pull
    /// kind), otherwise this `Input` will await a live stream to be pushed
    /// (push kind).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<InputSrc>,

    /// Indicator whether this `Input` is enabled, so is allowed to receive a
    /// live stream from its upstream sources.
    #[serde(default, skip_serializing_if = "is_false")]
    pub enabled: bool,
}

impl Input {
    /// Creates a new [`Input`] out of the given [`spec::v1::Input`].
    #[must_use]
    pub fn new(spec: spec::v1::Input) -> Self {
        Self {
            id: InputId::random(),
            key: spec.key,
            endpoints: spec
                .endpoints
                .into_iter()
                .map(InputEndpoint::new)
                .collect(),
            src: spec.src.map(InputSrc::new),
            enabled: spec.enabled,
        }
    }

    /// Applies the given [`spec::v1::Input`] to this [`Input`].
    pub fn apply(&mut self, new: spec::v1::Input) {
        if self.key != new.key
            || !new.enabled
            || (self.src.is_none() && new.src.is_some())
            || (self.src.is_some() && new.src.is_none())
        {
            // SRS endpoints have changed, disabled, or push/pull type has been
            // switched, so we should kick the publisher and all the players.
            for e in &mut self.endpoints {
                e.srs_publisher_id = None;
                e.srs_player_ids.clear();
            }
        }

        self.key = new.key;
        // Temporary omit changing existing `enabled` value to avoid unexpected
        // breakages of ongoing re-streams.
        //self.enabled = new.enabled;

        let mut olds = mem::replace(
            &mut self.endpoints,
            Vec::with_capacity(new.endpoints.len()),
        );
        for new in new.endpoints {
            if let Some(mut old) = olds
                .iter()
                .enumerate()
                .find_map(|(n, o)| (o.kind == new.kind).then_some(n))
                .map(|n| olds.swap_remove(n))
            {
                old.apply(new);
                self.endpoints.push(old);
            } else {
                self.endpoints.push(InputEndpoint::new(new));
            }
        }

        match (self.src.as_mut(), new.src) {
            (Some(old), Some(new)) => old.apply(new),
            (None, Some(new)) => self.src = Some(InputSrc::new(new)),
            _ => self.src = None,
        }
    }

    /// Exports this [`Input`] as a [`spec::v1::Input`].
    #[must_use]
    pub fn export(&self) -> spec::v1::Input {
        spec::v1::Input {
            id: Some(self.id),
            key: self.key.clone(),
            endpoints: self
                .endpoints
                .iter()
                .map(InputEndpoint::export)
                .collect(),
            src: self.src.as_ref().map(InputSrc::export),
            enabled: self.enabled,
        }
    }

    /// Enables this [`Input`].
    ///
    /// Returns `false` if it has been enabled already.
    #[must_use]
    pub fn enable(&mut self) -> bool {
        let mut changed = !self.enabled;

        self.enabled = true;

        if let Some(InputSrc::Failover(s)) = self.src.as_mut() {
            for i in &mut s.inputs {
                changed |= i.enable();
            }
        }

        changed
    }

    /// Disables this [`Input`].
    ///
    /// Returns `false` if it has been disabled already.
    #[must_use]
    pub fn disable(&mut self) -> bool {
        let mut changed = self.enabled;

        self.enabled = false;

        for e in &mut self.endpoints {
            e.srs_publisher_id = None;
            e.srs_player_ids.clear();
            // Do not rely only on SRS to set status, as it sporadically races.
            e.status = Status::Offline;
        }

        if let Some(InputSrc::Failover(s)) = self.src.as_mut() {
            for i in &mut s.inputs {
                changed |= i.disable();
            }
        }

        changed
    }

    /// Lookups for an [`Input`] with the given `id` inside this [`Input`] or
    /// its [`FailoverInputSrc::inputs`].
    #[must_use]
    pub fn find_mut(&mut self, id: InputId) -> Option<&mut Self> {
        if self.id == id {
            return Some(self);
        }
        if let Some(InputSrc::Failover(s)) = &mut self.src {
            s.inputs.iter_mut().find_map(|i| i.find_mut(id))
        } else {
            None
        }
    }

    /// Lookups for an [`InputEndpoint`] with the given `id` inside
    /// [`Input`] or its [`FailoverInputSrc::inputs`].
    pub fn find_endpoint(
        &mut self,
        id: EndpointId,
    ) -> Option<&mut InputEndpoint> {
        if let Some(endpoint) = self.endpoints.iter_mut().find(|e| e.id == id) {
            return Some(endpoint);
        }

        if let Some(InputSrc::Failover(s)) = self.src.as_mut() {
            for i in &mut s.inputs {
                if let Some(endpoint) = i.find_endpoint(id) {
                    return Some(endpoint);
                }
            }
        }

        None
    }

    /// Indicates whether this [`Input`] is ready to serve a live stream for
    /// [`Output`]s.
    ///
    /// [`Input`]: crate::state::Input
    /// [`Output`]: crate::state::Output
    #[must_use]
    pub fn is_ready_to_serve(&self) -> bool {
        let mut is_online = self
            .endpoints
            .iter()
            .any(|e| e.is_rtmp() && e.status == Status::Online);

        if !is_online {
            if let Some(InputSrc::Failover(s)) = &self.src {
                is_online = s.inputs.iter().any(|i| {
                    i.endpoints
                        .iter()
                        .any(|e| e.is_rtmp() && e.status == Status::Online)
                });
            }
        }

        is_online
    }
}

/// ID of an `Input`.
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
pub struct InputId(Uuid);

impl InputId {
    /// Generates a new random [`InputId`].
    #[inline]
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Key of an [`Input`] used to form its endpoint URL.
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
pub struct InputKey(String);

impl InputKey {
    /// Creates a new [`InputKey`] if the given value meets its invariants.
    #[must_use]
    pub fn new<'s, S: Into<Cow<'s, str>>>(val: S) -> Option<Self> {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("^[a-z0-9_-]{1,20}$").unwrap());

        let val = val.into();
        (!val.is_empty() && REGEX.is_match(&val))
            .then(|| Self(val.into_owned()))
    }
}

impl<'de> Deserialize<'de> for InputKey {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(<Cow<'_, str>>::deserialize(deserializer)?)
            .ok_or_else(|| D::Error::custom("Not a valid Input.key"))
    }
}

impl PartialEq<str> for InputKey {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}
