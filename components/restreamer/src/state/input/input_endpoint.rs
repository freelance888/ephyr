use std::collections::HashSet;

use derive_more::{Display, From, Into};
use juniper::{GraphQLEnum, GraphQLObject, GraphQLScalar};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::state::client_statistics::StreamStatistics;
use crate::state::StreamInfo;
use crate::{
    spec, srs,
    state::{InputKey, Label, RestreamKey, Status},
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

    /// Corresponding stream info from SRS
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_stat: Option<StreamStatistics>,

    /// History of stream changes. It's used for calculation of FPS value of [`StreamStatistics`]
    #[graphql(skip)]
    #[serde(skip)]
    stream_history: Option<Vec<StreamInfo>>,
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
            label: spec.label,
            srs_publisher_id: None,
            srs_player_ids: HashSet::new(),
            stream_stat: None,
            stream_history: Some(vec![]),
        }
    }

    /// Updates statistics for video and audio parameters from SRS stream
    pub fn update_stream_statistics(&mut self, srs_steam: StreamInfo) {
        let prev_fps = match &self.stream_stat {
            Some(s) => s.fps,
            None => 0,
        };

        let fps = match &mut self.stream_history {
            Some(h) if h.len() > 10 => {
                let srs_stream0 = &h[0];
                // Calculates FPS value
                let result = ((srs_steam.frames - srs_stream0.frames) * 1000)
                    as u64
                    / (srs_steam.live_ms - srs_stream0.live_ms);

                self.stream_history = Some(vec![]);
                // This is safe because we don't expect too big numbers.
                // But even in case of overflow it will return 0 and not fail
                result as i32
            }
            Some(h) => {
                h.push(srs_steam.clone());
                prev_fps
            }
            None => {
                self.stream_history = Some(vec![]);
                prev_fps
            }
        };

        self.stream_stat = Some(StreamStatistics {
            fps,
            kbps: srs_steam.kbps.recv_30s,
            width: srs_steam.video.width,
            height: srs_steam.video.height,
            video_codec: srs_steam.video.codec,
            audio_channel: srs_steam.audio.channel,
            audio_codec: srs_steam.audio.codec,
            audio_sample_rate: srs_steam.audio.sample_rate,
        });

        println!("NAME: {}, FPS: {}", srs_steam.name, fps);
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
        }
    }

    /// Indicates whether this [`InputEndpoint`] is an
    /// [`InputEndpointKind::Rtmp`].
    #[inline]
    #[must_use]
    pub fn is_rtmp(&self) -> bool {
        matches!(self.kind, InputEndpointKind::Rtmp)
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
}

impl InputEndpointKind {
    /// Returns RTMP URL on a local [SRS] server of this [`InputEndpointKind`]
    /// for the given `restream` and `input`.
    ///
    /// # Panics
    /// No panics, because [`RestreamKey`] and [`InputKey`] are validated.
    ///
    /// [SRS]: https://github.com/ossrs/srs
    #[must_use]
    pub fn rtmp_url(self, restream: &RestreamKey, input: &InputKey) -> Url {
        Url::parse(&format!(
            "rtmp://127.0.0.1:1935/{}{}/{}",
            restream,
            match self {
                Self::Rtmp => "",
                Self::Hls => "?vhost=hls",
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
