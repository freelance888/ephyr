use std::collections::HashSet;

use derive_more::{Display, From, Into};
use juniper::{GraphQLEnum, GraphQLObject, GraphQLScalar};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{
    file_manager::FileId,
    spec, srs,
    state::{InputKey, Label, RestreamKey, Status},
    stream_statistics::StreamStatistics,
};

/// Endpoint of an `Input` serving a live stream for `Output`s and clients.
#[derive(
    Clone, Debug, Deserialize, Eq, GraphQLObject, PartialEq, Serialize,
)]
pub struct InputEndpoint {
    /// Unique ID of this `InputEndpoint`.
    ///
    /// Once assigned, it never changes.
    pub id: EndpointId,

    /// Kind of this `InputEndpoint`.
    pub kind: InputEndpointKind,

    /// User defined label for each Endpoint
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<Label>,

    /// If the endpoint is of type FILE, then this contains
    /// the file ID that is in the [`State::files`]
    ///
    /// [`State::files`]: crate::state::State::files
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<FileId>,

    /// `Status` of this `InputEndpoint` indicating whether it actually serves a
    /// live stream ready to be consumed by `Output`s and clients.
    #[serde(skip)]
    pub status: Status,

    /// ID of [SRS] client who publishes a live stream to this [`InputEndpoint`]
    /// (either an external client or a local process).
    ///
    /// [SRS]: https://github.com/ossrs/srs
    #[graphql(skip)]
    #[serde(skip)]
    pub srs_publisher_id: Option<srs::ClientId>,

    /// IDs of [SRS] clients who play a live stream from this [`InputEndpoint`]
    /// (either an external clients or a local processes).
    ///
    /// [SRS]: https://github.com/ossrs/srs
    #[graphql(skip)]
    #[serde(skip)]
    pub srs_player_ids: HashSet<srs::ClientId>,

    /// Corresponding stream info
    pub stream_stat: Option<StreamStatistics>,
}

impl InputEndpoint {
    /// Creates a new [`InputEndpoint`] out of the given
    /// [`spec::v1::InputEndpoint`].
    #[inline]
    #[must_use]
    pub fn new(spec: spec::v1::InputEndpoint) -> Self {
        Self {
            id: EndpointId::random(),
            kind: spec.kind,
            status: Status::Offline,
            file_id: spec.file_id,
            label: spec.label,
            srs_publisher_id: None,
            srs_player_ids: HashSet::new(),
            stream_stat: None,
        }
    }

    /// Applies the given [`spec::v1::InputEndpoint`] to
    /// this [`InputEndpoint`].
    ///
    /// If `label` is not `None` than we already
    /// set the value and the update is not required.
    #[inline]
    pub fn apply(&mut self, new: spec::v1::InputEndpoint) {
        self.kind = new.kind;
        if new.label.is_some() {
            self.label = new.label;
        };
    }

    /// Exports this [`InputEndpoint`] as a [`spec::v1::InputEndpoint`].
    #[inline]
    #[must_use]
    pub fn export(&self) -> spec::v1::InputEndpoint {
        spec::v1::InputEndpoint {
            kind: self.kind,
            label: self.label.clone(),
            file_id: self.file_id.clone(),
        }
    }

    /// Indicates whether this [`InputEndpoint`] is an
    /// [`InputEndpointKind::Rtmp`].
    #[inline]
    #[must_use]
    pub fn is_rtmp(&self) -> bool {
        matches!(self.kind, InputEndpointKind::Rtmp)
    }

    /// Indicates whether this [`InputEndpoint`] is an
    /// [`InputEndpointKind::Rtmp`].
    #[inline]
    #[must_use]
    pub fn is_file(&self) -> bool {
        matches!(self.kind, InputEndpointKind::File)
    }
}

/// Possible kinds of an `InputEndpoint`.
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    Eq,
    From,
    GraphQLEnum,
    Hash,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "lowercase")]
pub enum InputEndpointKind {
    /// [RTMP] endpoint.
    ///
    /// Can accept a live stream and serve it for playing.
    ///
    /// [RTMP]: https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol
    #[display(fmt = "RTMP")]
    Rtmp,

    /// [HLS] endpoint.
    ///
    /// Only serves a live stream for playing and is not able to accept one.
    ///
    /// [HLS]: https://en.wikipedia.org/wiki/HTTP_Live_Streaming
    #[display(fmt = "HLS")]
    Hls,

    /// File input.
    #[display(fmt = "FILE")]
    File,
}

impl InputEndpointKind {
    /// Returns RTMP URL on a local [SRS] server of this [`InputEndpointKind`]
    /// for the given `restream` and `input`.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    #[must_use]
    pub fn rtmp_url(self, restream: &RestreamKey, input: &InputKey) -> Url {
        Self::get_rtmp_url(restream, input, self)
    }

    /// Create RTMP URL for specific [`RestreamKey`] and [`InputKey`]
    /// and [`InputEndpointKind`]
    ///
    /// # Panics
    /// No panics, because [`RestreamKey`] and [`InputKey`] are validated.
    #[must_use]
    pub fn get_rtmp_url(
        restream: &RestreamKey,
        input: &InputKey,
        kind: InputEndpointKind,
    ) -> Url {
        Url::parse(&format!(
            "rtmp://127.0.0.1:1935/{}{}/{}",
            restream,
            match kind {
                InputEndpointKind::Rtmp | InputEndpointKind::File => "",
                InputEndpointKind::Hls => "?vhost=hls",
            },
            input,
        ))
        .unwrap()
    }
}

/// ID of an `InputEndpoint`.
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
pub struct EndpointId(Uuid);

impl EndpointId {
    /// Generates a new random [`EndpointId`].
    #[inline]
    #[must_use]
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}
