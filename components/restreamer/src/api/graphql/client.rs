//! Client [GraphQL] API providing application usage.
//!
//! [GraphQL]: https://graphql.com

use std::collections::HashSet;

use actix_web::http::StatusCode;
use anyhow::anyhow;
use ephyr_log::tracing;
use futures::{stream::BoxStream, StreamExt};
use futures_signals::signal::SignalExt as _;
use juniper::{graphql_object, graphql_subscription, GraphQLObject, RootNode};
use once_cell::sync::Lazy;
use rand::Rng as _;
use tap::Tap;

use crate::{
    api::graphql,
    dvr, spec,
    state::{
        Delay, InputEndpointKind, InputId, InputKey, InputSrc, InputSrcUrl,
        Label, MixinId, MixinSrcUrl, OutputDstUrl, OutputId, PasswordKind,
        Restream, RestreamId, RestreamKey, Volume,
    },
    Spec,
};

use super::Context;
use crate::{
    file_manager::{
        get_video_list_from_gdrive_folder, FileCommand, FileId, FileState,
        LocalFileInfo,
    },
    spec::v1::BackupInput,
    state::{Direction, EndpointId, ServerInfo, VolumeLevel},
    types::UNumber,
};
use url::Url;

/// Schema of `Restreamer` app.
pub type Schema =
    RootNode<'static, QueriesRoot, MutationsRoot, SubscriptionsRoot>;

/// Constructs and returns new [`Schema`], ready for use.
#[inline]
#[must_use]
pub fn schema() -> Schema {
    Schema::new(QueriesRoot, MutationsRoot, SubscriptionsRoot)
}

/// Root of all [GraphQL mutations][1] in the [`Schema`].
///
/// [1]: https://spec.graphql.org/June2018/#sec-Root-Operation-Types
#[derive(Clone, Copy, Debug)]
pub struct MutationsRoot;

#[graphql_object(name = "Mutation", context = Context)]
impl MutationsRoot {
    /// Applies the specified JSON `spec` of `Restream`s to this server.
    ///
    /// If `replace` is `true` then replaces all the existing `Restream`s with
    /// the one defined by the `spec`. Otherwise, merges the `spec` with
    /// existing `Restream`s.
    ///
    /// ### Result
    ///
    /// Returns `null` if a `Restream` with the given `id` doesn't exist,
    /// otherwise always returns `true`.
    fn import(
        #[graphql(desc = "JSON spec obtained with `export` query.")]
        spec: String,
        #[graphql(
            description = "Indicator whether the `spec` should replace \
                           existing definitions.",
            default = false
        )]
        replace: bool,
        #[graphql(
            description = "Optional ID of a concrete `Restream` to apply \
                           the `spec` to without touching other `Restream`s."
        )]
        restream_id: Option<RestreamId>,
        context: &Context,
    ) -> Result<Option<bool>, graphql::Error> {
        let spec = serde_json::from_str::<Spec>(&spec)?.into_v1();

        Ok(if let Some(id) = restream_id {
            let spec = (spec.restreams.len() == 1)
                .then(|| spec.restreams.into_iter().next())
                .flatten()
                .ok_or_else(|| {
                    graphql::Error::new("INVALID_SPEC")
                        .status(StatusCode::BAD_REQUEST)
                        .message(
                            "JSON spec should contain exactly one Restream",
                        )
                })?;
            #[allow(clippy::manual_find_map)]
            // due to moving `spec` inside closure
            context
                .state()
                .restreams
                .lock_mut()
                .iter_mut()
                .find(|r| r.id == id)
                .map(|r| {
                    r.apply(spec, replace);
                    true
                })
        } else {
            context.state().apply(spec, replace);
            Some(true)
        })
    }

    /// Sets a new `Restream` or updates an existing one (if `id` is specified).
    ///
    /// ### Idempotency
    ///
    /// Idempotent if `id` is specified. Otherwise is non-idempotent, always
    /// creates a new `Restream` and errors on the `key` duplicates.
    ///
    /// ### Result
    ///
    /// Returns `null` if a `Restream` with the given `id` doesn't exist,
    /// otherwise always returns `true`.
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::too_many_arguments)]
    fn set_restream(
        #[graphql(description = "Unique key to set the `Restream` with.")]
        key: RestreamKey,
        #[graphql(description = "Optional label to set the `Restream` with.")]
        label: Option<Label>,
        #[graphql(description = "URL to pull a live stream from.
                           If not specified then `Restream` will await for a \
                           live stream being pushed to its endpoint.")]
        src: Option<InputSrcUrl>,
        #[graphql(description = "List of backup Inputs")] backup_inputs: Option<
            Vec<BackupInput>,
        >,
        #[graphql(
            description = "Google drive file ID for failover file endpoint."
        )]
        file_id: Option<FileId>,
        #[graphql(description = "Override for global maximum for files in \
                                 playlist")]
        max_files_in_playlist: Option<UNumber>,
        #[graphql(
            description = "Indicator whether the `Restream` should have an \
            additional endpoint for serving a live stream via HLS.",
            default = false
        )]
        with_hls: bool,
        #[graphql(description = "ID of the `Restream` to be updated \
                                 rather than creating a new one.")]
        id: Option<RestreamId>,
        context: &Context,
    ) -> Result<Option<bool>, graphql::Error> {
        let backups = match backup_inputs.clone() {
            None => Vec::new(),
            Some(b) => b,
        };

        let (input_key, input_src) = if backup_inputs.is_some()
            || file_id.is_some()
        {
            (
                InputKey::new("playback").unwrap(),
                Some(spec::v1::InputSrc::FailoverInputs(
                    vec![spec::v1::Input {
                        id: None,
                        key: InputKey::new("primary").unwrap(),
                        endpoints: vec![spec::v1::InputEndpoint {
                            kind: InputEndpointKind::Rtmp,
                            label: None,
                            file_id: None,
                        }],
                        src: src.map(spec::v1::InputSrc::RemoteUrl),
                        enabled: true,
                    }]
                    .into_iter()
                    .chain(backups.into_iter().map(|b| spec::v1::Input {
                        id: None,
                        key: b.key,
                        endpoints: vec![spec::v1::InputEndpoint {
                            kind: InputEndpointKind::Rtmp,
                            label: None,
                            file_id: None,
                        }],
                        src: b.src.map(spec::v1::InputSrc::RemoteUrl),
                        enabled: true,
                    }))
                    .chain(
                        file_id
                            .map_or_else(Vec::new, |id| {
                                vec![spec::v1::Input {
                                    id: None,
                                    key: InputKey::new("file_backup").unwrap(),
                                    endpoints: vec![spec::v1::InputEndpoint {
                                        kind: InputEndpointKind::File,
                                        label: None,
                                        file_id: Some(id),
                                    }],
                                    src: None,
                                    enabled: true,
                                }]
                            })
                            .into_iter(),
                    )
                    .collect(),
                )),
            )
        } else {
            (
                InputKey::new("primary").unwrap(),
                src.map(spec::v1::InputSrc::RemoteUrl),
            )
        };

        let mut endpoints = vec![spec::v1::InputEndpoint {
            kind: InputEndpointKind::Rtmp,
            label: None,
            file_id: None,
        }];
        if with_hls {
            endpoints.push(spec::v1::InputEndpoint {
                kind: InputEndpointKind::Hls,
                label: None,
                file_id: None,
            });
        }

        let spec = spec::v1::Restream {
            id: None,
            key,
            label,
            input: spec::v1::Input {
                id: None,
                key: input_key,
                endpoints,
                src: input_src,
                enabled: true,
            },
            outputs: vec![],
            max_files_in_playlist,
        };

        #[allow(clippy::option_if_let_else)] // due to consuming `spec`
        Ok(if let Some(id) = id {
            context.state().edit_restream(id, spec)
        } else {
            context.state().add_restream(spec).map(Some)
        }
        .tap(|_| {
            let mut commands = context.state().file_commands.lock_mut();
            commands.push(FileCommand::ListOfFilesChanged);
        })
        .map_err(|e| {
            graphql::Error::new("DUPLICATE_RESTREAM_KEY")
                .status(StatusCode::CONFLICT)
                .message(&e)
        })?
        .map(|_| true))
    }

    /// Force download file from Google Drive by it's id
    fn download_file(
        #[graphql(
            description = "ID of the file from `Google Drive` to be downloaded."
        )]
        file_id: FileId,
        context: &Context,
    ) -> Option<bool> {
        let mut restreams = context.state().restreams.lock_mut();
        restreams.iter_mut().for_each(|restream| {
            if let Some(InputSrc::Failover(fo)) = &restream.input.src {
                fo.inputs.clone().iter_mut().for_each(|input| {
                    let _ = input
                        .clone()
                        .endpoints
                        .iter_mut()
                        .find(|endpoint| {
                            endpoint.is_file()
                                && endpoint.file_id == Some(file_id.clone())
                        })
                        .map(|_| input.disable());
                });
            }
        });

        let mut commands = context.state().file_commands.lock_mut();
        commands.push(FileCommand::NeedDownloadFile(file_id));

        Some(true)
    }

    /// Removes a `Restream` by its `id`.
    ///
    /// ### Result
    ///
    /// Returns `null` if `Restream` with the given `id` doesn't exist,
    /// otherwise always returns `true`.
    fn remove_restream(
        #[graphql(description = "ID of the `Restream` to be removed.")]
        id: RestreamId,
        context: &Context,
    ) -> Option<bool> {
        context.state().remove_restream(id)?;
        Some(true)
    }

    /// Enables a `Restream` by its `id`.
    ///
    /// Enabled `Restream` is allowed to accept or pull a live stream.
    ///
    /// ### Result
    ///
    /// Returns `true` if a `Restream` with the given `id` has been enabled,
    /// `false` if it has been enabled already, and `null` if it doesn't exist.
    fn enable_restream(
        #[graphql(description = "ID of the `Restream` to be enabled.")]
        id: RestreamId,
        context: &Context,
    ) -> Option<bool> {
        context.state().enable_restream(id)
    }

    /// Disables a `Restream` by its `id`.
    ///
    /// Disabled `Restream` stops all on-going re-streaming processes and is not
    /// allowed to accept or pull a live stream.
    ///
    /// ### Result
    ///
    /// Returns `true` if a `Restream` with the given `id` has been disabled,
    /// `false` if it has been disabled already, and `null` if it doesn't exist.
    fn disable_restream(
        #[graphql(description = "ID of the `Restream` to be disabled.")]
        id: RestreamId,
        context: &Context,
    ) -> Option<bool> {
        context.state().disable_restream(id)
    }

    fn set_playlist(
        restream_id: RestreamId,
        playlist: Vec<FileId>,
        context: &Context,
    ) -> Option<bool> {
        context.state().restreams.lock_mut().iter_mut().find_map(
            |restream| {
                (restream.id == restream_id).then(|| {
                    restream.playlist.queue = playlist
                        .iter()
                        .filter_map(|order_id| {
                            restream
                                .playlist
                                .queue
                                .iter()
                                .find(|f| f.file_id == *order_id)
                        })
                        .cloned()
                        .collect();
                    let mut commands = context.state().file_commands.lock_mut();
                    commands.push(FileCommand::ListOfFilesChanged);
                })
            },
        )?;
        Some(true)
    }

    fn cancel_playlist_download(
        restream_id: RestreamId,
        context: &Context,
    ) -> Option<bool> {
        let restream = context
            .state()
            .restreams
            .get_cloned()
            .into_iter()
            .find(|r| r.id == restream_id);

        let mut found = false;
        if let Some(r) = restream {
            context.state().files.lock_mut().iter_mut().for_each(|f| {
                r.playlist
                    .queue
                    .iter()
                    .find(|pf| pf.file_id == f.file_id)
                    .is_some()
                    .then(|| {
                        if f.state != FileState::Local {
                            f.state = FileState::DownloadError;
                            f.error = Some("Download was canceled".to_string());
                            found = true
                        }
                    });
            });
        }

        Some(found)
    }

    fn restart_playlist_download(
        restream_id: RestreamId,
        context: &Context,
    ) -> Option<bool> {
        let restream = context
            .state()
            .restreams
            .get_cloned()
            .into_iter()
            .find(|r| r.id == restream_id);

        let mut found = false;
        if let Some(r) = restream {
            context.state().files.lock_mut().iter_mut().for_each(|f| {
                r.playlist
                    .queue
                    .iter()
                    .find(|pf| pf.file_id == f.file_id)
                    .is_some()
                    .then(|| {
                        if f.state != FileState::Local {
                            f.state = FileState::Waiting;
                            f.error = None;
                            found = true
                        }
                    });
            });
        }

        Some(found)
    }

    fn cancel_file_download(
        file_id: FileId,
        context: &Context,
    ) -> Option<bool>
    {
        context.state()
            .files
            .lock_mut()
            .iter_mut()
            .find_map(|f| (f.file_id == file_id)
                .then(|| {
                    if f.state != FileState::Local {
                        f.state = FileState::DownloadError;
                        f.error = Some("Download was canceled".to_string());
                        true
                    } else {
                        false
                    }
                })
            )
    }

    fn stop_playing_file_from_playlist(
        restream_id: RestreamId,
        context: &Context,
    ) -> Option<bool> {
        context
            .state()
            .restreams
            .lock_mut()
            .iter_mut()
            .find_map(|r| {
                (r.id == restream_id).then(|| {
                    r.playlist.currently_playing_file = None;
                })
            })?;

        Some(true)
    }

    /// Starts playing the provided file from playlist
    fn play_file_from_playlist(
        restream_id: RestreamId,
        file_id: FileId,
        context: &Context,
    ) -> Option<bool> {
        context
            .state()
            .restreams
            .lock_mut()
            .iter_mut()
            .find_map(|r| {
                (r.id == restream_id).then(|| {
                    r.playlist.currently_playing_file = r
                        .playlist
                        .queue
                        .iter()
                        .find(|f| f.file_id == file_id)
                        .cloned();
                })
            })?;

        Some(true)
    }

    /// Starts playing file if it's found in playlist of any `[Restream]`
    ///
    /// Returns `true` if file was found in any of existing `[Restream]`s
    /// and `false` if no such file was found
    fn broadcast_play_file(
        #[graphql(description = "file identity")] file_id: FileId,
        context: &Context,
    ) -> Option<bool> {
        let mut has_found = false;
        context
            .state()
            .restreams
            .lock_mut()
            .iter_mut()
            .for_each(|r| {
                let found =
                    r.playlist.queue.iter().find(|f| f.file_id == file_id);

                if found.is_some() {
                    r.playlist.currently_playing_file = found.cloned();
                    has_found = true;
                }
            });

        Some(has_found)
    }

    /// Stops playing file if it's found in playlist of any `[Restream]`
    ///
    /// Returns `true` if file was found in any of existing `[Restream]`s
    /// and `false` if no such file was found
    fn broadcast_stop_playing_file(
        #[graphql(description = "file identity")] file_id: FileId,
        context: &Context,
    ) -> Option<bool> {
        let mut has_found = false;
        context
            .state()
            .restreams
            .lock_mut()
            .iter_mut()
            .for_each(|r| {
                let found =
                    r.playlist.queue.iter().find(|f| f.file_id == file_id);

                if found.is_some() {
                    r.playlist.currently_playing_file = None;
                    has_found = true;
                }
            });

        Some(has_found)
    }

    /// Sends request to Google API and appends found files to the provided
    /// restream's playlist.
    async fn get_playlist_from_gdrive(
        restream_id: RestreamId,
        folder_id: String,
        context: &Context,
    ) -> Result<Option<bool>, graphql::Error> {
        let api_key = context
            .state()
            .settings
            .lock_mut()
            .google_api_key
            .clone()
            .ok_or_else(|| {
                graphql::Error::new("NO_API_KEY")
                    .status(StatusCode::UNAUTHORIZED)
                    .message("No API key")
            })?;
        let result =
            get_video_list_from_gdrive_folder(&api_key, &folder_id).await;
        if let Ok(mut playlist_files) = result {
            if playlist_files.is_empty() {
                let err = "No files in playlist. Probably there is no public access to the playlist";
                tracing::error!(err);
                return Err(graphql::Error::new("GDRIVE_API_ERROR")
                    .status(StatusCode::BAD_REQUEST)
                    .message(&err));
            }

            playlist_files.sort_by_key(|x| x.name.clone());
            context
                .state()
                .restreams
                .lock_mut()
                .iter_mut()
                .find(|r| r.id == restream_id)
                .ok_or_else(|| {
                    graphql::Error::new("UNKNOWN_RESTREAM")
                        .message("Could not find restream with provided ID")
                })?
                .playlist
                .apply(playlist_files);

            let mut commands = context.state().file_commands.lock_mut();
            commands.push(FileCommand::ListOfFilesChanged);

            Ok(Some(true))
        } else {
            let err = result.err().unwrap();
            tracing::error!(err);
            Err(graphql::Error::new("GDRIVE_API_ERROR")
                .status(StatusCode::BAD_REQUEST)
                .message(&err))
        }
    }

    /// Enables an `Input` by its `id`.
    ///
    /// Enabled `Input` is allowed to accept or pull a live stream.
    ///
    /// ### Result
    ///
    /// Returns `true` if an `Input` with the given `id` has been enabled,
    /// `false` if it has been enabled already, and `null` if it doesn't exist.
    fn enable_input(
        #[graphql(description = "ID of the `Input` to be enabled.")]
        id: InputId,
        #[graphql(
            description = "ID of the `Restream` to enable the `Input` in."
        )]
        restream_id: RestreamId,
        context: &Context,
    ) -> Option<bool> {
        context.state().enable_input(id, restream_id)
    }

    /// Disables an `Input` by its `id`.
    ///
    /// Disabled `Input` stops all on-going re-streaming processes and is not
    /// allowed to accept or pull a live stream.
    ///
    /// ### Result
    ///
    /// Returns `true` if an `Input` with the given `id` has been disabled,
    /// `false` if it has been disabled already, and `null` if it doesn't exist.
    fn disable_input(
        #[graphql(description = "ID of the `Input` to be disabled.")]
        id: InputId,
        #[graphql(
            description = "ID of the `Restream` to disable the `Input` in."
        )]
        restream_id: RestreamId,
        context: &Context,
    ) -> Option<bool> {
        context.state().disable_input(id, restream_id)
    }

    /// Moves this [`Input`] in given direction.
    ///
    /// This may affect the order and priority of endpoints.
    /// E.g. if the second endpoint is moved up, it will become the new primary.
    ///
    /// ### Result
    ///
    /// Returns `true` if the move was successful, or `false` if not.
    fn move_input_in_direction(
        #[graphql(description = "ID of the `Input` to be streamed.")]
        id: InputId,
        #[graphql(
            description = "ID of the `Restream` to stream the `Input` in."
        )]
        restream_id: RestreamId,
        context: &Context,
        direction: Direction,
    ) -> Result<Option<bool>, graphql::Error> {
        context
            .state()
            .move_input_in_direction(id, restream_id, direction)
    }

    /// Sets an `Input`'s endpoint label by `Input` and `Endpoint` `id`.
    ///
    /// ### Result
    ///
    /// Returns `true` if the label has been set with the given `label`,
    /// `false` if it was not
    /// `null` if the `Input` or `Endpoint` doesn't exist.
    fn set_endpoint_label(
        #[graphql(description = "ID of the `Input` to be changed.")]
        id: InputId,
        #[graphql(description = "ID of the `Restream` to change.")]
        restream_id: RestreamId,
        endpoint_id: EndpointId,
        label: Option<Label>,
        context: &Context,
    ) -> Option<bool> {
        context
            .state()
            .set_endpoint_label(id, restream_id, endpoint_id, label)
    }

    /// Sets a new `Output` or updates an existing one (if `id` is specified).
    ///
    /// ### Idempotency
    ///
    /// Idempotent if `id` is specified. Otherwise is non-idempotent, always
    /// creates a new `Output` and errors on the `dst` duplicates within the
    /// specified `Restream`.
    ///
    /// ### Result
    ///
    /// Returns `null` if a `Restream` with the given `restreamId` doesn't
    /// exist, or an `Output` with the given `id` doesn't exist, otherwise
    /// always returns `true`.
    fn set_output(
        #[graphql(
            description = "ID of the `Restream` to add a new `Output` to."
        )]
        restream_id: RestreamId,
        #[graphql(
            description = "Destination URL to re-stream a live stream onto.\
                           \n\n\
                           At the moment only [RTMP] and [Icecast] are \
                           supported.\
                           \n\n\
                           [Icecast]: https://icecast.org\n\
                           [RTMP]: https://en.wikipedia.org/wiki/\
                                   Real-Time_Messaging_Protocol"
        )]
        dst: OutputDstUrl,
        #[graphql(description = "Optional label to add a new `Output` with.")]
        label: Option<Label>,
        preview_url: Option<Url>,
        #[graphql(
            description = "Optional `MixinSrcUrl`s to mix into this `Output`.",
            default = Vec::new(),
        )]
        mixins: Vec<MixinSrcUrl>,
        #[graphql(description = "ID of the `Output` to be updated \
                                 rather than creating a new one.")]
        id: Option<OutputId>,
        context: &Context,
    ) -> Result<Option<bool>, graphql::Error> {
        if mixins.len() > 5 {
            return Err(graphql::Error::new("TOO_MUCH_MIXIN_URLS")
                .status(StatusCode::BAD_REQUEST)
                .message("Maximum 5 mixing URLs are allowed"));
        }
        if !mixins.is_empty() {
            let mut unique = HashSet::with_capacity(mixins.len());
            for m in &mixins {
                if let Some(dup) = unique.replace(m) {
                    return Err(graphql::Error::new("DUPLICATE_MIXIN_URL")
                        .status(StatusCode::BAD_REQUEST)
                        .message(&format!(
                            "Duplicate Output.mixin.src: {dup}",
                        )));
                }
            }
            if mixins.iter().filter(|u| u.scheme() == "ts").take(4).count() > 3
            {
                return Err(graphql::Error::new(
                    "TOO_MUCH_TEAMSPEAK_MIXIN_URLS",
                )
                .status(StatusCode::BAD_REQUEST)
                .message("Maximum 3 TeamSpeak URLs are allowed"));
            }
        }

        let existing_output = id.as_ref().and_then(|output_id| {
            context.state().get_output(restream_id, *output_id)
        });
        let mut original_volume = Volume::ORIGIN.export();
        if let Some(output) = existing_output.as_ref() {
            if !mixins.is_empty() {
                original_volume = output.volume.export();
            }
        }

        let spec = spec::v1::Output {
            id: None,
            dst,
            label,
            preview_url,
            volume: original_volume,
            mixins: mixins
                .into_iter()
                .map(|src| {
                    let delay;
                    let volume;
                    let sidechain;
                    if let Some(orig_mixin) =
                        existing_output.as_ref().and_then(|val| {
                            val.mixins.iter().find(|val| val.src == src)
                        })
                    {
                        volume = orig_mixin.volume.export();
                        delay = orig_mixin.delay;
                        sidechain = orig_mixin.sidechain;
                    } else {
                        volume = Volume::ORIGIN.export();
                        delay = (src.scheme() == "ts")
                            .then(|| Delay::from_millis(3500))
                            .flatten()
                            .unwrap_or_default();
                        sidechain = false;
                    }
                    spec::v1::Mixin {
                        src,
                        volume,
                        delay,
                        sidechain,
                    }
                })
                .collect(),
            enabled: false,
        };

        #[allow(clippy::option_if_let_else)] // due to consuming `spec`
        Ok(if let Some(id) = id {
            context.state().edit_output(restream_id, id, spec)
        } else {
            context.state().add_output(restream_id, spec)
        }
        .map_err(|e| {
            graphql::Error::new("DUPLICATE_OUTPUT_URL")
                .status(StatusCode::CONFLICT)
                .message(&e)
        })?
        .map(|_| true))
    }

    /// Removes an `Output` by its `id` from the specified `Restream`.
    ///
    /// ### Result
    ///
    /// Returns `null` if the specified `Restream`/`Output` doesn't exist,
    /// otherwise always returns `true`.
    fn remove_output(
        #[graphql(description = "ID of the `Output` to be removed.")]
        id: OutputId,
        #[graphql(
            description = "ID of the `Restream` to remove the `Output` from."
        )]
        restream_id: RestreamId,
        context: &Context,
    ) -> Option<bool> {
        context.state().remove_output(id, restream_id).map(|_| true)
    }

    /// Enables an `Output` by its `id` in the specified `Restream`.
    ///
    /// Enabled `Output` starts re-streaming a live stream to its destination.
    ///
    /// ### Result
    ///
    /// Returns `true` if an `Output` with the given `id` has been enabled,
    /// `false` if it has been enabled already, and `null` if the specified
    /// `Restream`/`Output` doesn't exist.
    fn enable_output(
        #[graphql(description = "ID of the `Output` to be enabled.")]
        id: OutputId,
        #[graphql(
            description = "ID of the `Restream` to enable the `Output` in."
        )]
        restream_id: RestreamId,
        context: &Context,
    ) -> Option<bool> {
        context.state().enable_output(id, restream_id)
    }

    /// Disables an `Output` by its `id` in the specified `Restream`.
    ///
    /// Disabled `Output` stops re-streaming a live stream to its destination.
    ///
    /// ### Result
    ///
    /// Returns `true` if an `Output` with the given `id` has been disabled,
    /// `false` if it has been disabled already, and `null` if the specified
    /// `Restream`/`Output` doesn't exist.
    fn disable_output(
        #[graphql(description = "ID of the `Output` to be disabled.")]
        id: OutputId,
        #[graphql(
            description = "ID of the `Restream` to disable the `Output` in."
        )]
        restream_id: RestreamId,
        context: &Context,
    ) -> Option<bool> {
        context.state().disable_output(id, restream_id)
    }

    /// Enables all `Output`s in the specified `Restream`.
    ///
    /// Enabled `Output`s start re-streaming a live stream to their
    /// destinations.
    ///
    /// ### Result
    ///
    /// Returns `true` if at least one `Output` has been enabled, `false` if all
    /// `Output`s have been enabled already, and `null` if the specified
    /// `Restream` doesn't exist.
    fn enable_all_outputs(
        #[graphql(
            description = "ID of the `Restream` to enable all `Output`s in."
        )]
        restream_id: RestreamId,
        context: &Context,
    ) -> Option<bool> {
        context.state().enable_all_outputs(restream_id)
    }

    /// Disables all `Output`s in the specified `Restream`.
    ///
    /// Disabled `Output`s stop re-streaming a live stream to their
    /// destinations.
    ///
    /// ### Result
    ///
    /// Returns `true` if at least one `Output` has been disabled, `false` if
    /// all `Output`s have been disabled already, and `null` if the specified
    /// `Restream` doesn't exist.
    fn disable_all_outputs(
        #[graphql(
            description = "ID of the `Restream` to disable all `Output`s in."
        )]
        restream_id: RestreamId,
        context: &Context,
    ) -> Option<bool> {
        context.state().disable_all_outputs(restream_id)
    }

    /// Disables all `Output`s in all `Restream`s.
    ///
    /// Disabled `Output`s stop re-streaming a live stream to their
    /// destinations.
    ///
    /// ### Result
    ///
    /// Returns `true` if at least one `Output` has been disabled, `false` if
    /// all `Output`s have been disabled already or there are no outputs
    fn disable_all_outputs_of_restreams(context: &Context) -> bool {
        context.state().disable_all_outputs_of_restreams()
    }

    /// Enables all `Output`s in all `Restream`s.
    ///
    /// Enabled `Output`s start re-streaming a live stream to their
    /// destinations.
    ///
    /// ### Result
    ///
    /// Returns `true` if at least one `Output` has been enabled, `false` if all
    /// `Output`s have been enabled already or there are no outputs
    fn enables_all_outputs_of_restreams(context: &Context) -> bool {
        context.state().enable_all_outputs_of_restreams()
    }

    /// Tunes a `Volume` rate of the specified `Output` or one of its `Mixin`s.
    ///
    /// ### Result
    ///
    /// Returns `true` if a `Volume` rate has been changed, `false` if it has
    /// the same value already, or `null` if the specified `Output` or `Mixin`
    /// doesn't exist.
    fn tune_volume(
        #[graphql(
            description = "ID of the `Restream` to tune the `Output` in."
        )]
        restream_id: RestreamId,
        #[graphql(description = "ID of the tuned `Output`.")]
        output_id: OutputId,
        #[graphql(description = "Optional ID of the tuned `Mixin`.\
                                \n\n\
                                If set, then tunes the `Mixin` rather than \
                                the `Output`.")]
        mixin_id: Option<MixinId>,
        #[graphql(description = "Volume rate in percents to be set.")]
        level: VolumeLevel,
        muted: bool,
        context: &Context,
    ) -> Option<bool> {
        context.state().tune_volume(
            restream_id,
            output_id,
            mixin_id,
            Volume { level, muted },
        )
    }

    /// Tunes a `Delay` of the specified `Mixin` before mix it into its
    /// `Output`.
    ///
    /// ### Result
    ///
    /// Returns `true` if a `Delay` has been changed, `false` if it has the same
    /// value already, or `null` if the specified `Output` or `Mixin` doesn't
    /// exist.
    fn tune_delay(
        #[graphql(
            description = "ID of the `Restream` to tune the the `Mixin` in."
        )]
        restream_id: RestreamId,
        #[graphql(description = "ID of the `Output` of the tuned `Mixin`.")]
        output_id: OutputId,
        #[graphql(description = "ID of the tuned `Mixin`.")] mixin_id: MixinId,
        #[graphql(description = "Number of milliseconds to delay \
                                 the `Mixin` before mix it into its `Output`.")]
        delay: Delay,
        context: &Context,
    ) -> Option<bool> {
        context
            .state()
            .tune_delay(restream_id, output_id, mixin_id, delay)
    }

    /// Tunes a `Sidechain` of the specified `Mixin` before mix it into its
    /// `Output`.
    ///
    /// ### Result
    ///
    /// Returns `true` if a `Sidechain` has been changed, `false` if it has
    /// the same value already, or `null` if the specified `Output`
    /// or `Mixin` doesn't exist.
    fn tune_sidechain(
        #[graphql(
            description = "ID of the `Restream` to tune the the `Mixin` in."
        )]
        restream_id: RestreamId,
        #[graphql(description = "ID of the `Output` of the tuned `Mixin`.")]
        output_id: OutputId,
        #[graphql(description = "ID of the tuned `Mixin`.")] mixin_id: MixinId,
        sidechain: bool,
        context: &Context,
    ) -> Option<bool> {
        context.state().tune_sidechain(
            restream_id,
            output_id,
            mixin_id,
            sidechain,
        )
    }

    /// Removes the specified recorded file.
    ///
    /// ### Result
    ///
    /// Returns `true` if the specified recorded file was removed, otherwise
    /// `false` if nothing changes.
    async fn remove_dvr_file(
        #[graphql(
            description = "Relative path of the recorded file to be removed.\
                           \n\n \
                           Use the exact value returned by `Query.dvrFiles`."
        )]
        path: String,
    ) -> Result<bool, graphql::Error> {
        if path.starts_with('/') || path.contains("../") {
            return Err(graphql::Error::new("INVALID_DVR_FILE_PATH")
                .status(StatusCode::BAD_REQUEST)
                .message(&format!("Invalid DVR file path: {path}")));
        }

        Ok(dvr::Storage::global().remove_file(path).await)
    }

    /// Sets or unsets the password to protect this GraphQL API with.
    ///
    /// Once password is set, any subsequent requests to this GraphQL API should
    /// perform [HTTP Basic auth][1], where any username is allowed, but the
    /// password should match the one being set.
    ///
    /// ### Result
    ///
    /// Returns `true` if password has been changed or unset, otherwise `false`
    /// if nothing changes.
    ///
    /// [1]: https://en.wikipedia.org/wiki/Basic_access_authentication
    fn set_password(
        #[graphql(description = "New password to be set. \
                                 In `null` then unsets the current password.")]
        new: Option<String>,
        #[graphql(description = "Old password for authorization, \
                                 if it was set previously.")]
        old: Option<String>,
        kind: Option<PasswordKind>,
        context: &Context,
    ) -> Result<bool, graphql::Error> {
        static HASH_CFG: Lazy<argon2::Config<'static>> =
            Lazy::new(argon2::Config::default);

        let settings = context.state().settings.get_cloned();
        let hash = match kind {
            None | Some(PasswordKind::Main) => settings.password_hash,
            Some(PasswordKind::Output) => settings.password_output_hash,
        };

        if let Some(hash) = &hash {
            match old {
                None => {
                    return Err(graphql::Error::new("NO_OLD_PASSWORD")
                        .status(StatusCode::FORBIDDEN)
                        .message("Old password required for this action"))
                }
                Some(pass) => {
                    if !argon2::verify_encoded(hash, pass.as_bytes()).unwrap() {
                        return Err(graphql::Error::new("WRONG_OLD_PASSWORD")
                            .status(StatusCode::FORBIDDEN)
                            .message("Wrong old password specified"));
                    }
                }
            }
        }

        if hash.is_none() && new.is_none() {
            return Ok(false);
        }

        let new_hash = new.map(|v| {
            argon2::hash_encoded(
                v.as_bytes(),
                &rand::thread_rng().gen::<[u8; 32]>(),
                &HASH_CFG,
            )
            .unwrap()
        });

        let mut settings = context.state().settings.lock_mut();
        match kind {
            None | Some(PasswordKind::Main) => {
                settings.password_hash = new_hash;
            }
            Some(PasswordKind::Output) => {
                settings.password_output_hash = new_hash;
            }
        };

        Ok(true)
    }

    /// Sets settings of the server
    ///
    /// ### Result
    ///
    /// Returns `false` if title does not pass validation for max allowed
    /// characters length. Otherwise returns `true`
    fn set_settings(
        #[graphql(description = "Title for the server")] title: Option<String>,
        #[graphql(description = "Whether do we need to confirm deletion \
                                 of inputs and outputs")]
        delete_confirmation: Option<bool>,
        #[graphql(
            description = "Whether do we need to confirm enabling/disabling \
                           of inputs or outputs"
        )]
        enable_confirmation: Option<bool>,
        #[graphql(description = "Google API key for google drive access")]
        google_api_key: Option<String>,
        #[graphql(description = "Maximum number of files in playlist")]
        max_files_in_playlist: Option<UNumber>,
        context: &Context,
    ) -> Result<bool, graphql::Error> {
        // Validate title
        let value = title.unwrap_or_default();
        if value.len() > 70 {
            return Err(graphql::Error::new("WRONG_TITLE_LENGTH")
                .status(StatusCode::BAD_REQUEST)
                .message("Title exceeds max allowed length of 70 characters"));
        }

        let mut settings = context.state().settings.lock_mut();
        settings.title = Some(value);
        settings.delete_confirmation = delete_confirmation;
        settings.enable_confirmation = enable_confirmation;
        settings.google_api_key = google_api_key;
        settings.max_files_in_playlist = max_files_in_playlist;
        Ok(true)
    }
}

/// Root of all [GraphQL queries][1] in the [`Schema`].
///
/// [1]: https://spec.graphql.org/June2018/#sec-Root-Operation-Types
#[derive(Clone, Copy, Debug)]
pub struct QueriesRoot;

#[graphql_object(name = "Query", context = Context)]
impl QueriesRoot {
    /// Returns the current `Info` parameters of this server.
    fn info(context: &Context) -> Info {
        let settings = context.state().settings.get_cloned();
        Info {
            public_host: context.config().public_host.clone().unwrap(),
            password_hash: settings.password_hash,
            password_output_hash: settings.password_output_hash,
            title: settings.title,
            delete_confirmation: settings.delete_confirmation,
            enable_confirmation: settings.enable_confirmation,
            google_api_key: settings.google_api_key,
            max_files_in_playlist: settings.max_files_in_playlist,
        }
    }

    /// Returns the current `ServerInfo`
    fn server_info(context: &Context) -> ServerInfo {
        let info = context.state().server_info.get_cloned();
        ServerInfo {
            cpu_usage: info.cpu_usage,
            cpu_cores: info.cpu_cores,
            ram_total: info.ram_total,
            ram_free: info.ram_free,
            tx_delta: info.tx_delta,
            rx_delta: info.rx_delta,
            error_msg: info.error_msg,
        }
    }

    /// Returns all the `Restream`s happening on this server.
    fn all_restreams(context: &Context) -> Vec<Restream> {
        context.state().restreams.get_cloned()
    }

    /// Returns list of recorded files of the specified `Output`.
    ///
    /// If returned list is empty, the there is no recorded files for the
    /// specified `Output`.
    ///
    /// Each recorded file is represented as a relative path on [SRS] HTTP
    /// server in `dvr/` directory, so the download link should look like this:
    /// ```ignore
    /// http://my.host:8080/dvr/returned/file/path.flv
    /// http://my.host:8080/dvr/returned/file/path.wav
    /// http://my.host:8080/dvr/returned/file/path.mp3
    /// ```
    ///
    /// [SRS]: https://github.com/ossrs/srs
    async fn dvr_files(
        #[graphql(
            description = "ID of the `Output` to return recorded files of."
        )]
        id: OutputId,
    ) -> Vec<String> {
        dvr::Storage::global().list_files(id).await
    }

    /// Returns `Restream`s happening on this server and identifiable by the
    /// given `ids` in an exportable JSON format.
    ///
    /// If no `ids` specified, then returns all the `Restream`s happening on
    /// this server at the moment.
    fn export(
        #[graphql(
            description = "IDs of `Restream`s to be exported. \n\n \
                           If empty, then all the `Restream`s \
                           will be exported.",
            default = Vec::new(),
        )]
        ids: Vec<RestreamId>,
        context: &Context,
    ) -> Result<Option<String>, graphql::Error> {
        let settings = context.state().settings.get_cloned().export();
        let restreams = context
            .state()
            .restreams
            .get_cloned()
            .into_iter()
            .filter_map(|r| {
                (ids.is_empty() || ids.contains(&r.id)).then(|| r.export())
            })
            .collect::<Vec<_>>();
        (!restreams.is_empty())
            .then(|| {
                let spec: Spec = spec::v1::Spec {
                    settings: Some(settings),
                    restreams,
                }
                .into();
                serde_json::to_string(&spec).map_err(|e| {
                    anyhow!("Failed to JSON-serialize spec: {e}").into()
                })
            })
            .transpose()
    }
}

/// Root of all [GraphQL subscriptions][1] in the [`Schema`].
///
/// [1]: https://spec.graphql.org/June2018/#sec-Root-Operation-Types
#[derive(Clone, Copy, Debug)]
pub struct SubscriptionsRoot;

#[graphql_subscription(name = "Subscription", context = Context)]
impl SubscriptionsRoot {
    /// Subscribes to updates of `Info` parameters of this server.
    async fn info(context: &Context) -> BoxStream<'static, Info> {
        let public_host = context.config().public_host.clone().unwrap();
        context
            .state()
            .settings
            .signal_cloned()
            .dedupe_cloned()
            .map(move |h| Info {
                public_host: public_host.clone(),
                password_hash: h.password_hash,
                password_output_hash: h.password_output_hash,
                title: h.title,
                delete_confirmation: h.delete_confirmation,
                enable_confirmation: h.enable_confirmation,
                google_api_key: h.google_api_key,
                max_files_in_playlist: h.max_files_in_playlist,
            })
            .to_stream()
            .boxed()
    }

    /// Subscribes to updates of `ServerInfo` parameters of this server.
    async fn server_info(context: &Context) -> BoxStream<'static, ServerInfo> {
        context
            .state()
            .server_info
            .signal_cloned()
            .dedupe_cloned()
            .to_stream()
            .boxed()
    }

    /// Subscribes to updates of all `Restream`s happening on this server.
    async fn all_restreams(
        context: &Context,
    ) -> BoxStream<'static, Vec<Restream>> {
        context
            .state()
            .restreams
            .signal_cloned()
            .dedupe_cloned()
            .to_stream()
            .boxed()
    }

    /// Subscribes to updates of all `File`'s happening on this server
    async fn files(
        context: &Context,
    ) -> BoxStream<'static, Vec<LocalFileInfo>> {
        context
            .state()
            .files
            .signal_cloned()
            .dedupe_cloned()
            .to_stream()
            .boxed()
    }

    /// Subscribes to updates of specific file
    async fn file(
        id: FileId,
        context: &Context,
    ) -> BoxStream<'static, Option<LocalFileInfo>> {
        context
            .state()
            .files
            .signal_cloned()
            .filter_map(move |files| {
                files.into_iter().find(|f| f.file_id == id)
            })
            .dedupe_cloned()
            .to_stream()
            .boxed()
    }

    /// Subscribes to updates of currently playing file in playlist
    async fn currently_playing_file(
        id: RestreamId,
        context: &Context,
    ) -> BoxStream<'static, Option<LocalFileInfo>> {
        let files = context.state().files.get_cloned();

        context
            .state()
            .restreams
            .signal_cloned()
            .filter_map(move |restreams| {
                restreams.into_iter().find(|r| r.id == id).and_then(|r| {
                    if let Some(playing_file) =
                        r.playlist.currently_playing_file
                    {
                        files
                            .clone()
                            .into_iter()
                            .find(|f| f.file_id == playing_file.file_id)
                    } else {
                        None
                    }
                })
            })
            .dedupe_cloned()
            .to_stream()
            .boxed()
    }

    /// Subscribes to updates of specific restream
    async fn restream(
        id: RestreamId,
        context: &Context,
    ) -> BoxStream<'static, Option<Restream>> {
        context
            .state()
            .restreams
            .signal_cloned()
            .filter_map(move |restreams| {
                restreams.into_iter().find(|r| r.id == id)
            })
            .dedupe_cloned()
            .to_stream()
            .boxed()
    }
}

/// Information about parameters that this server operates with.
#[derive(Clone, Debug, GraphQLObject)]
pub struct Info {
    /// Host that this server is reachable via in public.
    ///
    /// Use it for constructing URLs to this server.
    pub public_host: String,

    /// Title of the server
    pub title: Option<String>,

    /// Whether do we need to confirm deletion of inputs and outputs
    pub delete_confirmation: Option<bool>,

    /// Whether do we need to confirm enabling/disabling of inputs or outputs
    pub enable_confirmation: Option<bool>,

    /// [Argon2] hash of the password that this server's GraphQL API is
    /// protected with, if any.
    ///
    /// Non-`null` value means that any request to GraphQL API should perform
    /// [HTTP Basic auth][1]. Any username is allowed, but the password should
    /// match this hash.
    ///
    /// [Argon2]: https://en.wikipedia.org/wiki/Argon2
    /// [1]: https://en.wikipedia.org/wiki/Basic_access_authentication
    pub password_hash: Option<String>,

    /// Password hash for single output application
    pub password_output_hash: Option<String>,

    /// Google API key for file downloading
    pub google_api_key: Option<String>,

    /// Max number of files allowed in [Restream]'s playlist
    /// This value can be overwritten by the similar setting
    /// on a particular [Restream]
    pub max_files_in_playlist: Option<UNumber>,
}
