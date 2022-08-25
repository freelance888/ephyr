use std::{mem, path::Path};

use derive_more::{Deref, Display, From, Into};
use juniper::{GraphQLObject, GraphQLScalar};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use url::Url;
use uuid::Uuid;

use crate::state::{Label, Mixin, Status, Volume};
use crate::{serde::is_false, spec};

/// Downstream destination that a `Restream` re-streams a live stream to.
#[derive(
    Clone, Debug, Deserialize, Eq, GraphQLObject, PartialEq, Serialize,
)]
pub struct Output {
    /// Unique ID of this `Output`.
    ///
    /// Once assigned, it never changes.
    pub id: OutputId,

    /// Downstream URL to re-stream a live stream onto.
    ///
    /// At the moment only [RTMP] and [Icecast] are supported.
    ///
    /// [Icecast]: https://icecast.org
    /// [RTMP]: https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol
    pub dst: OutputDstUrl,

    /// Optional label of this `Output`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<Label>,

    /// Url of stream preview.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<Url>,

    /// Volume rate of this `Output`'s audio tracks when mixed with
    /// `Output.mixins`.
    ///
    /// Has no effect when there is no `Output.mixins`.
    #[serde(default, skip_serializing_if = "Volume::is_origin")]
    pub volume: Volume,

    /// `Mixin`s to mix this `Output` with before re-streaming it to its
    /// downstream destination.
    ///
    /// If empty, then no mixing is performed and re-streaming is as cheap as
    /// possible (just copies bytes "as is").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mixins: Vec<Mixin>,

    /// Indicator whether this `Output` is enabled, so is allowed to perform a
    /// live stream re-streaming to its downstream destination.
    #[serde(default, skip_serializing_if = "is_false")]
    pub enabled: bool,

    /// `Status` of this `Output` indicating whether it actually re-streams a
    /// live stream to its downstream destination.
    #[serde(skip)]
    pub status: Status,
}

impl Output {
    /// Creates a new [`Output`] out of the given [`spec::v1::Output`].
    #[inline]
    #[must_use]
    pub fn new(spec: spec::v1::Output) -> Self {
        Self {
            id: OutputId::random(),
            dst: spec.dst,
            label: spec.label,
            preview_url: spec.preview_url,
            volume: Volume::new(&spec.volume),
            mixins: spec.mixins.into_iter().map(Mixin::new).collect(),
            enabled: spec.enabled,
            status: Status::Offline,
        }
    }

    /// Applies the given [`spec::v1::Output`] to this [`Output`].
    ///
    /// If `replace` is `true` then all the [`Output::mixins`] will be replaced
    /// with new ones, otherwise new ones will be merged with already existing
    /// [`Output::mixins`].
    pub fn apply(&mut self, new: spec::v1::Output, replace: bool) {
        self.dst = new.dst;
        self.label = new.label;
        self.preview_url = new.preview_url;
        self.volume = Volume::new(&new.volume);
        // Temporary omit changing existing `enabled` value to avoid unexpected
        // breakages of ongoing re-streams.
        //self.enabled = new.enabled;
        if replace {
            let mut olds = mem::replace(
                &mut self.mixins,
                Vec::with_capacity(new.mixins.len()),
            );
            for new in new.mixins {
                if let Some(mut old) = olds
                    .iter()
                    .enumerate()
                    .find_map(|(n, o)| (o.src == new.src).then(|| n))
                    .map(|n| olds.swap_remove(n))
                {
                    old.apply(new);
                    self.mixins.push(old);
                } else {
                    self.mixins.push(Mixin::new(new));
                }
            }
        } else {
            for new in new.mixins {
                if let Some(old) =
                    self.mixins.iter_mut().find(|o| o.src == new.src)
                {
                    old.apply(new);
                } else {
                    self.mixins.push(Mixin::new(new));
                }
            }
        }
    }

    /// Exports this [`Output`] as a [`spec::v1::Output`].
    #[inline]
    #[must_use]
    pub fn export(&self) -> spec::v1::Output {
        spec::v1::Output {
            id: Some(self.id),
            dst: self.dst.clone(),
            label: self.label.clone(),
            preview_url: self.preview_url.clone(),
            volume: self.volume.export(),
            mixins: self.mixins.iter().map(Mixin::export).collect(),
            enabled: self.enabled,
        }
    }
}

/// ID of an `Output`.
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
pub struct OutputId(Uuid);

impl OutputId {
    /// Generates a new random [`OutputId`].
    #[inline]
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}

/// [`Url`] of an [`Output::dst`].
///
/// Only the following URLs are allowed at the moment:
/// - [RTMP] URL (starting with `rtmp://` or `rtmps://` scheme and having a
///   host);
/// - [SRT] URL (starting with `srt://` scheme and having a host);
/// - [Icecast] URL (starting with `icecast://` scheme and having a host);
/// - [FLV] file URL (starting with `file:///` scheme, without host and
///   subdirectories, and with `.flv` extension in its path).
///
/// [FLV]: https://en.wikipedia.org/wiki/Flash_Video
/// [Icecast]: https://icecast.org
/// [RTMP]: https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol
/// [SRT]: https://en.wikipedia.org/wiki/Secure_Reliable_Transport
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
pub struct OutputDstUrl(Url);

impl OutputDstUrl {
    /// Creates a new [`OutputDstUrl`] if the given [`Url`] is suitable for
    /// that.
    ///
    /// # Errors
    ///
    /// Returns the given [`Url`] back if it doesn't represent a valid
    /// [`OutputDstUrl`].
    #[inline]
    pub fn new(url: Url) -> Result<Self, Url> {
        if Self::validate(&url) {
            Ok(Self(url))
        } else {
            Err(url)
        }
    }

    /// Validates the given [`Url`] to represent a valid [`OutputDstUrl`].
    #[must_use]
    pub fn validate(url: &Url) -> bool {
        match url.scheme() {
            "icecast" | "rtmp" | "rtmps" | "srt" => url.has_host(),
            "file" => {
                let path = Path::new(url.path());
                !url.has_host()
                    && path.is_absolute()
                    && path.extension() == Some("flv".as_ref())
                    && path.parent() == Some("/".as_ref())
                    && !url.path().contains("/../")
            }
            _ => false,
        }
    }
}

impl<'de> Deserialize<'de> for OutputDstUrl {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(Url::deserialize(deserializer)?).map_err(|url| {
            D::Error::custom(format!("Not a valid Output.src URL: {}", url))
        })
    }
}
