//! [`Url`] of a [`Mixin::src`].
use derive_more::{Deref, Display, Into};
use juniper::GraphQLScalar;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use std::path::Path;
use url::Url;

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
            D::Error::custom(format!("Not a valid Mixin.src URL: {}", url))
        })
    }
}
