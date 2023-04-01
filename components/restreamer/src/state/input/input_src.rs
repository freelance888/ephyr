use std::{fmt::Display, mem, path::Path};

use derive_more::{Deref, Display, From, Into};
use juniper::{GraphQLObject, GraphQLScalar, GraphQLUnion};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use url::Url;

use crate::{
    spec,
    state::{Input, Label},
};

/// Source to pull a live stream by an `Input` from.
#[derive(
    Clone, Debug, Deserialize, Eq, From, GraphQLUnion, PartialEq, Serialize,
)]
#[serde(rename_all = "lowercase")]
pub enum InputSrc {
    /// Remote endpoint.
    Remote(RemoteInputSrc),

    /// Multiple local endpoints forming a failover source.
    Failover(FailoverInputSrc),
}

impl Display for InputSrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputSrc::Remote(src) => write!(f, "InputSrc::Remote({src})"),
            InputSrc::Failover(src) => write!(f, "InputSrc::Failover({src})"),
        }
    }
}

impl InputSrc {
    /// Creates a new [`InputSrc`] out of the given [`spec::v1::InputSrc`].
    #[inline]
    #[must_use]
    pub fn new(spec: spec::v1::InputSrc) -> Self {
        match spec {
            spec::v1::InputSrc::RemoteUrl(url) => {
                Self::Remote(RemoteInputSrc { url, label: None })
            }
            spec::v1::InputSrc::FailoverInputs(inputs) => {
                Self::Failover(FailoverInputSrc {
                    inputs: inputs.into_iter().map(Input::new).collect(),
                })
            }
        }
    }

    /// Applies the given [`spec::v1::InputSrc`] to this [`InputSrc`].
    ///
    /// Replaces all the [`FailoverInputSrc::inputs`] with new ones.
    pub fn apply(&mut self, new: spec::v1::InputSrc) {
        match (self, new) {
            (Self::Remote(old), spec::v1::InputSrc::RemoteUrl(new_url)) => {
                old.url = new_url;
            }
            (Self::Failover(src), spec::v1::InputSrc::FailoverInputs(news)) => {
                let mut olds = mem::replace(
                    &mut src.inputs,
                    Vec::with_capacity(news.len()),
                );
                for new in news {
                    if let Some(mut old) = olds
                        .iter()
                        .enumerate()
                        .find_map(|(n, o)| (o.key == new.key).then_some(n))
                        .map(|n| olds.swap_remove(n))
                    {
                        old.apply(new);
                        src.inputs.push(old);
                    } else {
                        src.inputs.push(Input::new(new));
                    }
                }
            }
            (old, new) => *old = Self::new(new),
        }
    }

    /// Exports this [`InputSrc`] as a [`spec::v1::InputSrc`].
    #[inline]
    #[must_use]
    pub fn export(&self) -> spec::v1::InputSrc {
        match self {
            Self::Remote(i) => spec::v1::InputSrc::RemoteUrl(i.url.clone()),
            Self::Failover(src) => spec::v1::InputSrc::FailoverInputs(
                src.inputs.iter().map(Input::export).collect(),
            ),
        }
    }
}

/// Remote upstream source to pull a live stream by an `Input` from.
#[derive(
    Clone, Debug, Deserialize, Eq, GraphQLObject, PartialEq, Serialize,
)]
pub struct RemoteInputSrc {
    /// URL of this `RemoteInputSrc`.
    pub url: InputSrcUrl,

    /// Label for this Endpoint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<Label>,
}

impl Display for RemoteInputSrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RemoteInputSrc({})", self.url.path())
    }
}

/// Failover source of multiple `Input`s to pull a live stream by an `Input`
/// from.
#[derive(
    Clone, Debug, Deserialize, Eq, GraphQLObject, PartialEq, Serialize,
)]
pub struct FailoverInputSrc {
    /// `Input`s forming this `FailoverInputSrc`.
    ///
    /// Failover is implemented by attempting to pull the first `Input` falling
    /// back to the second one, and so on. Once the first source is restored,
    /// we pool from it once again.
    pub inputs: Vec<Input>,
}

impl Display for FailoverInputSrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FailoverInputSrc({})", self.inputs.len())
    }
}

/// [`Url`] of a [`RemoteInputSrc`].
///
/// Only the following URLs are allowed at the moment:
/// - [RTMP] URL (starting with `rtmp://` or `rtmps://` scheme and having a
///   host);
/// - [HLS] URL (starting with `http://` or `https://` scheme, having a host,
///   and with `.m3u8` extension in its path).
///
/// [HLS]: https://en.wikipedia.org/wiki/HTTP_Live_Streaming
/// [RTMP]: https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol
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
pub struct InputSrcUrl(Url);

impl InputSrcUrl {
    /// Creates a new [`InputSrcUrl`] if the given [`Url`] is suitable for that.
    ///
    /// # Errors
    ///
    /// Returns the given [`Url`] back if it doesn't represent a valid
    /// [`InputSrcUrl`].
    #[inline]
    pub fn new(url: Url) -> Result<Self, Url> {
        if Self::validate(&url) {
            Ok(Self(url))
        } else {
            Err(url)
        }
    }

    /// Validates the given [`Url`] to represent a valid [`InputSrcUrl`].
    #[must_use]
    pub fn validate(url: &Url) -> bool {
        match url.scheme() {
            "rtmp" | "rtmps" => url.has_host(),
            "http" | "https" => {
                url.has_host()
                    && Path::new(url.path()).extension()
                        == Some("m3u8".as_ref())
            }
            _ => false,
        }
    }
}

impl<'de> Deserialize<'de> for InputSrcUrl {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(Url::deserialize(deserializer)?).map_err(|url| {
            D::Error::custom(format!("Not a valid RemoteInputSrc.url: {url}"))
        })
    }
}
