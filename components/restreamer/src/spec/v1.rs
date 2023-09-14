//! Version 1 of a shareable (exportable and importable) specification of
//! application's [`State`].
//!
//! [`State`]: state::State
use ephyr_serde::is_false;
use std::collections::HashSet;

use crate::{file_manager::FileId, state, types::UNumber};
use juniper::GraphQLInputObject;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use url::Url;

/// Shareable (exportable and importable) specification of a [`State`].
///
/// [`State`]: state::State
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Spec {
    /// [`Settings`] to be performed.
    pub settings: Option<Settings>,

    /// [`Restream`]s to be performed.
    #[serde(deserialize_with = "Spec::deserialize_restreams")]
    pub restreams: Vec<Restream>,
}

impl Spec {
    fn deserialize_restreams<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<Restream>, D::Error> {
        let restreams = <Vec<Restream>>::deserialize(deserializer)?;

        if !restreams.is_empty() {
            let mut unique = HashSet::with_capacity(restreams.len());
            for r in &restreams {
                if let Some(key) = unique.replace(&r.key) {
                    return Err(D::Error::custom(format!(
                        "Duplicate Restream.key in Spec.restreams: {key}",
                    )));
                }
            }
        }

        Ok(restreams)
    }
}

/// Server's settings.
///
/// It keeps different settings not related to restreams but to whole server
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Settings {
    /// Title for the server
    /// It is used for differentiating servers on UI side if multiple servers
    /// are used.
    pub title: Option<String>,

    /// Whether do we need to confirm deletion of inputs and outputs
    /// If `true` we should confirm deletion, `false` - do not confirm.
    pub delete_confirmation: Option<bool>,

    /// Whether do we need to confirm enabling/disabling of inputs \
    /// or outputs.
    pub enable_confirmation: Option<bool>,

    /// Google API key for file playback and downloading.
    pub google_api_key: Option<String>,

    /// Max number of files allowed in [Restream]'s playlist.
    pub max_downloading_files: Option<UNumber>,
}

/// Shareable (exportable and importable) specification of a
/// [`state::Restream`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Restream {
    /// Unique ID of [`Restream`].
    pub id: Option<state::RestreamId>,

    /// Unique key of this [`Restream`] identifying it, and used to form its
    /// endpoints URLs.
    pub key: state::RestreamKey,

    /// Optional label of this [`Restream`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<state::Label>,

    /// [`Input`] that a live stream is received from.
    pub input: Input,

    /// [`Output`]s that a live stream is re-streamed to.
    #[serde(
        default,
        deserialize_with = "Restream::deserialize_outputs",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub outputs: Vec<Output>,

    /// Playlist for this restream
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playlist: Option<Playlist>,

    /// If need to set input playback encoding.
    ///
    /// Option here to preserve schema backward compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_playback_encoding: Option<bool>,
}

impl Restream {
    /// Deserializes [`Restream::outputs`] ensuring its invariants preserved.
    fn deserialize_outputs<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<Output>, D::Error> {
        let outputs = <Vec<Output>>::deserialize(deserializer)?;

        if !outputs.is_empty() {
            let mut unique = HashSet::with_capacity(outputs.len());
            for o in &outputs {
                if let Some(dst) = unique.replace(&o.dst) {
                    return Err(D::Error::custom(format!(
                        "Duplicate Output.dst in Restream.outputs: {dst}"
                    )));
                }
            }
        }

        Ok(outputs)
    }
}

/// Shareable (exportable and importable) specification of a [`state::Input`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Input {
    /// Unique ID of this `Input`.
    ///
    /// Once assigned, it never changes.
    pub id: Option<state::InputId>,

    /// Key of this [`Input`] to expose its [`InputEndpoint`]s with for
    /// accepting and serving a live stream.
    pub key: state::InputKey,

    /// Endpoints of this [`Input`] serving a live stream for `Output`s and
    /// clients.
    pub endpoints: Vec<InputEndpoint>,

    /// Source to pull a live stream from.
    ///
    /// If empty then a live stream is received (pushed) rather than is pulled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<InputSrc>,

    /// Indicator whether this [`Input`] is enabled, so is allowed to receive a
    /// live stream from its upstream sources.
    #[serde(default, skip_serializing_if = "is_false")]
    pub enabled: bool,
}

impl Input {
    /// Creates a file backup [`Input`] out of the given [`FileId`].
    #[must_use]
    pub fn new_file_backup(file_id: FileId) -> Self {
        Self {
            id: None,
            key: state::InputKey::file_backup(),
            endpoints: vec![InputEndpoint {
                kind: state::InputEndpointKind::File,
                label: None,
                file_id: Some(file_id),
            }],
            src: None,
            enabled: true,
        }
    }
    /// Creates a new primary [`Input`] out of the given [`state::InputSrcUrl`].
    #[must_use]
    pub fn new_primary(src: Option<state::InputSrcUrl>) -> Self {
        Self {
            id: None,
            key: state::InputKey::primary(),
            endpoints: vec![InputEndpoint {
                kind: state::InputEndpointKind::Rtmp,
                label: None,
                file_id: None,
            }],
            src: src.map(InputSrc::RemoteUrl),
            enabled: true,
        }
    }

    /// Creates a new backup [`Input`] out of the given [`BackupInput`].
    #[must_use]
    pub fn new_backup(backup: BackupInput) -> Self {
        Self {
            id: None,
            key: backup.key,
            endpoints: vec![InputEndpoint {
                kind: state::InputEndpointKind::Rtmp,
                label: None,
                file_id: None,
            }],
            src: backup.src.map(InputSrc::RemoteUrl),
            enabled: true,
        }
    }

    /// Creates a new [`Input`] out of the given
    /// [`state::InputKey`] and [`InputSrc`].
    #[must_use]
    pub fn new(
        input_key: state::InputKey,
        input_src: Option<InputSrc>,
        with_hls: bool,
    ) -> Input {
        let mut endpoints = vec![InputEndpoint {
            kind: state::InputEndpointKind::Rtmp,
            label: None,
            file_id: None,
        }];
        if with_hls {
            endpoints.push(InputEndpoint {
                kind: state::InputEndpointKind::Hls,
                label: None,
                file_id: None,
            });
        }

        Input {
            id: None,
            key: input_key,
            endpoints,
            src: input_src,
            enabled: true,
        }
    }
}
impl<'de> Deserialize<'de> for Input {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawInput {
            key: state::InputKey,
            endpoints: Vec<InputEndpoint>,
            #[serde(default)]
            src: Option<InputSrc>,
            #[serde(default)]
            enabled: bool,
        }

        let raw = RawInput::deserialize(deserializer)?;

        let mut unique_endpoints = HashSet::with_capacity(raw.endpoints.len());
        for e in &raw.endpoints {
            if let Some(kind) = unique_endpoints.replace(e.kind) {
                return Err(D::Error::custom(format!(
                    "Duplicate InputEndpoint.kind in Input.endpoints: {kind}"
                )));
            }
        }
        if !unique_endpoints.contains(&state::InputEndpointKind::Rtmp)
            && !unique_endpoints.contains(&state::InputEndpointKind::File)
        {
            return Err(D::Error::custom(format!(
                "Input.endpoints should contain at least one {} or {} endpoint",
                state::InputEndpointKind::Rtmp,
                state::InputEndpointKind::File,
            )));
        }

        if let Some(src) = &raw.src {
            fn ensure_srcs_unique<'i>(
                src: &'i InputSrc,
                unique_urls: &mut HashSet<&'i state::InputSrcUrl>,
                unique_keys: &mut HashSet<&'i state::InputKey>,
            ) -> Result<(), String> {
                match src {
                    InputSrc::RemoteUrl(url) => {
                        if let Some(url) = unique_urls.replace(url) {
                            return Err(format!(
                                "Duplicate RemoteInputSrc.url in Input.src: \
                                {url}"
                            ));
                        }
                    }
                    InputSrc::FailoverInputs(inputs) => {
                        for i in inputs {
                            if let Some(key) = unique_keys.replace(&i.key) {
                                return Err(format!(
                                    "Duplicate Input.key in Input.srcs: {key}"
                                ));
                            }
                            if let Some(s) = i.src.as_ref() {
                                ensure_srcs_unique(
                                    s,
                                    unique_urls,
                                    unique_keys,
                                )?;
                            }
                        }
                    }
                }
                Ok(())
            }

            let mut unique_urls = HashSet::new();
            let mut unique_keys = HashSet::with_capacity(1);
            _ = unique_keys.insert(&raw.key);
            ensure_srcs_unique(src, &mut unique_urls, &mut unique_keys)
                .map_err(D::Error::custom)?;
        }

        Ok(Self {
            id: None,
            key: raw.key,
            endpoints: raw.endpoints,
            src: raw.src,
            enabled: raw.enabled,
        })
    }
}

/// Shareable (exportable and importable) specification of a
/// [`state::InputEndpoint`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InputEndpoint {
    /// Kind of this [`InputEndpoint`].
    pub kind: state::InputEndpointKind,

    /// Label for this input
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<state::Label>,

    /// If the endpoint has kind FILE then this contains the file ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<FileId>,
}

/// Shareable (exportable and importable) specification of a
/// [`state::InputSrc`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSrc {
    /// Remote endpoint represented by its URL.
    RemoteUrl(state::InputSrcUrl),

    /// Multiple [`Input`]s forming a failover source.
    FailoverInputs(Vec<Input>),
}

/// Shareable (exportable and importable) specification of a [`state::Output`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Output {
    /// Unique ID of this `Output`.
    ///
    /// Once assigned, it never changes.
    pub id: Option<state::OutputId>,

    /// Downstream URL to re-stream a live stream onto.
    pub dst: state::OutputDstUrl,

    /// Optional label of this [`Output`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<state::Label>,

    /// Url of stream preview.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<Url>,

    /// Volume rate of this [`Output`]'s audio tracks when mixed with
    /// [`Output::mixins`].
    #[serde(default, skip_serializing_if = "Volume::is_origin")]
    pub volume: Volume,

    /// [`Mixin`]s to mix this [`Output`] with before re-streaming it to its
    /// downstream destination.
    ///
    /// If empty, then no mixing is performed.
    #[serde(
        default,
        deserialize_with = "Output::deserialize_mixins",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub mixins: Vec<Mixin>,

    /// Indicator whether this [`Output`]  is enabled, so is allowed to perform
    /// a live stream re-streaming to its downstream destination.
    #[serde(default, skip_serializing_if = "is_false")]
    pub enabled: bool,
}

impl Output {
    /// Deserializes [`Output::mixins`] ensuring its invariants preserved.
    fn deserialize_mixins<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<Mixin>, D::Error> {
        let mixins = <Vec<Mixin>>::deserialize(deserializer)?;

        if !mixins.is_empty() {
            let mut unique = HashSet::with_capacity(mixins.len());
            let mut ts_count: u8 = 0;
            let mut has_sidechain = false;
            for m in &mixins {
                if let Some(src) = unique.replace(&m.src) {
                    return Err(D::Error::custom(format!(
                        "Duplicate Mixin.src in Output.mixins: {src}"
                    )));
                }
                if m.src.scheme() == "ts" {
                    ts_count += 1;
                    if ts_count > 3 {
                        return Err(D::Error::custom(format!(
                            "Maximum 3 TeamSpeak Mixin.src allowed \
                            in Output.mixins: {}",
                            m.src,
                        )));
                    }
                }
                if m.sidechain {
                    if has_sidechain {
                        return Err(D::Error::custom(format!(
                            "Only one Mixin.sidechain is allowed \
                            in Output.mixins: {}",
                            m.src
                        )));
                    }
                    has_sidechain = true;
                }
            }
        }

        Ok(mixins)
    }
}

/// Shareable (exportable and importable) specification of a [`state::Mixin`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Mixin {
    /// URL of the source to be mixed with an [`Output`].
    pub src: state::MixinSrcUrl,

    /// Volume rate of this [`Mixin`]'s audio tracks to mix them with.
    #[serde(default, skip_serializing_if = "Volume::is_origin")]
    pub volume: Volume,

    /// Delay that this [`Mixin`] should wait before being mixed with an
    /// [`Output`].
    #[serde(default, skip_serializing_if = "state::Delay::is_zero")]
    pub delay: state::Delay,

    /// Set that this [`Mixin`] should be side-chained with an
    /// [`Output`].
    #[serde(default, skip_serializing_if = "is_false")]
    pub sidechain: bool,
}

/// Shareable specification of [`state::Volume`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Volume {
    /// Volume level or rate in percentages
    #[serde(default)]
    pub level: state::VolumeLevel,

    /// Flag if the volume is muted
    #[serde(default)]
    pub muted: bool,
}

impl Volume {
    /// Indicates whether this [`Volume`] rate value corresponds
    /// origin value of `state::Volume`
    #[allow(clippy::trivially_copy_pass_by_ref)] // required for `serde`
    #[inline]
    #[must_use]
    pub fn is_origin(&self) -> bool {
        state::Volume::new(self).is_origin()
    }
}

/// Implementation of Default volume as [`state::Volume::default`]
impl Default for Volume {
    fn default() -> Self {
        state::Volume::default().export()
    }
}

/// Backup input
#[derive(
    Clone, Debug, Deserialize, Eq, PartialEq, Serialize, GraphQLInputObject,
)]
pub struct BackupInput {
    /// Key for this [`BackupInput`]
    pub key: state::InputKey,

    /// URL to pull a live stream from for a backup endpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<state::InputSrcUrl>,
}

/// Shareable (exportable and importable) specification of a [`state::Playlist`]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Playlist {
    /// List of files in playlist
    pub queue: Vec<PlaylistFileInfo>,
}

impl Playlist {
    /// Creates a new [`Playlist`] out of the given
    /// [`state::Playlist`].
    #[must_use]
    pub fn new(playlist: state::Playlist) -> Playlist {
        Self {
            queue: playlist
                .queue
                .into_iter()
                .map(|x| PlaylistFileInfo {
                    name: x.name,
                    file_id: x.file_id,
                })
                .collect(),
        }
    }
}

/// Shareable (exportable and importable) specification of a
/// [`PlaylistFileInfo`]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaylistFileInfo {
    /// Google ID of this file
    pub file_id: FileId,

    /// Name of this file
    pub name: String,
}
