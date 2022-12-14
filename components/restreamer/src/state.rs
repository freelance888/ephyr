//! Application state.
#![allow(clippy::module_name_repetitions)]

mod client_statistics;
mod input;
mod label;
mod output;
mod restream;
mod settings;

pub use self::{
    client_statistics::{
        Client, ClientId, ClientStatistics, ClientStatisticsResponse,
        ServerInfo, StatusStatistics,
    },
    input::{
        EndpointId, FailoverInputSrc, Input, InputEndpoint, InputEndpointKind,
        InputId, InputKey, InputSrc, InputSrcUrl, RemoteInputSrc,
    },
    label::Label,
    output::{
        Delay, Mixin, MixinId, MixinSrcUrl, Output, OutputDstUrl, OutputId,
        Volume, VolumeLevel,
    },
    restream::{Restream, RestreamId, RestreamKey},
    settings::Settings,
};

use std::{future::Future, mem, panic::AssertUnwindSafe, path::Path};

use anyhow::anyhow;
use ephyr_log::log;
use futures::{
    future::TryFutureExt as _,
    sink,
    stream::{StreamExt as _, TryStreamExt as _},
};
use futures_signals::signal::{Mutable, SignalExt as _};
use juniper::GraphQLEnum;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use tokio::{fs, io::AsyncReadExt as _};

use crate::{
    display_panic, spec,
    state::client_statistics::StreamStatistics,
    stream_probe::{Stream, StreamInfo},
    types::UNumber,
    Spec,
};
use std::collections::HashMap;

/// Reactive application's state.
///
/// Any changes to it automatically propagate to the appropriate subscribers.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct State {
    /// Global [`Settings`] of the server
    pub settings: Mutable<Settings>,

    /// All [`Restream`]s performed by this application.
    pub restreams: Mutable<Vec<Restream>>,

    /// All [`Client`]s for monitoring
    pub clients: Mutable<Vec<Client>>,

    /// Global [`ServerInfo`] of the server
    pub server_info: Mutable<ServerInfo>,
}

impl State {
    /// Instantiates a new [`State`] reading it from a `file` (if any) and
    /// performing all the required inner subscriptions.
    ///
    /// # Errors
    ///
    /// If [`State`] file exists, but fails to be parsed.
    pub async fn try_new<P: AsRef<Path>>(
        file: P,
    ) -> Result<Self, anyhow::Error> {
        let file = file.as_ref();

        let mut contents = vec![];
        let _ = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .open(&file)
            .await
            .map_err(|e| {
                anyhow!("Failed to open '{}' file: {}", file.display(), e)
            })?
            .read_to_end(&mut contents)
            .await
            .map_err(|e| {
                anyhow!("Failed to read '{}' file: {}", file.display(), e)
            })?;

        let state = if contents.is_empty() {
            State::default()
        } else {
            serde_json::from_slice(&contents).map_err(|e| {
                anyhow!(
                    "Failed to deserialize state from '{}' file: {}",
                    file.display(),
                    e,
                )
            })?
        };

        let (file, persisted_state) = (file.to_owned(), state.clone());
        let persist_state1 = move || {
            fs::write(
                file.clone(),
                serde_json::to_vec(&persisted_state)
                    .expect("Failed to serialize server state"),
            )
            .map_err(|e| log::error!("Failed to persist server state: {e}"))
        };
        let persist_state2 = persist_state1.clone();
        let persist_state3 = persist_state1.clone();

        Self::on_change("persist_restreams", &state.restreams, move |_| {
            persist_state1()
        });
        Self::on_change("persist_settings", &state.settings, move |_| {
            persist_state2()
        });
        Self::on_change("persist_clients", &state.clients, move |_| {
            persist_state3()
        });

        Ok(state)
    }

    /// Applies the given [`Spec`] to this [`State`].
    ///
    /// If `replace` is `true` then all the [`Restream`]s, [`Restream::outputs`]
    /// and [`Output::mixins`] will be replaced with new ones, otherwise new
    /// ones will be merged with already existing ones.
    pub fn apply(&self, new: spec::v1::Spec, replace: bool) {
        let mut restreams = self.restreams.lock_mut();
        if replace {
            let mut olds = mem::replace(
                &mut *restreams,
                Vec::with_capacity(new.restreams.len()),
            );
            for new in new.restreams {
                if let Some(mut old) = olds
                    .iter()
                    .enumerate()
                    .find_map(|(n, o)| (o.key == new.key).then_some(n))
                    .map(|n| olds.swap_remove(n))
                {
                    old.apply(new, replace);
                    restreams.push(old);
                } else {
                    restreams.push(Restream::new(new));
                }
            }
        } else {
            for new in new.restreams {
                if let Some(old) =
                    restreams.iter_mut().find(|o| o.key == new.key)
                {
                    old.apply(new, replace);
                } else {
                    restreams.push(Restream::new(new));
                }
            }
        }

        let mut settings = self.settings.lock_mut();
        if new.settings.is_some() || replace {
            settings.apply(
                new.settings.unwrap_or_else(|| Settings::default().export()),
            );
        }
    }

    /// Exports this [`State`] as a [`spec::v1::Spec`].
    #[inline]
    #[must_use]
    pub fn export(&self) -> Spec {
        spec::v1::Spec {
            settings: Some(self.settings.get_cloned().export()),
            restreams: self
                .restreams
                .get_cloned()
                .iter()
                .map(Restream::export)
                .collect(),
        }
        .into()
    }

    /// Subscribes the specified `hook` to changes of the [`Mutable`] `val`ue.
    ///
    /// `name` is just a convenience for describing the `hook` in logs.
    pub fn on_change<F, Fut, T>(name: &'static str, val: &Mutable<T>, hook: F)
    where
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: Future + Send + 'static,
        T: Clone + PartialEq + Send + Sync + 'static,
    {
        drop(tokio::spawn(
            AssertUnwindSafe(
                val.signal_cloned().dedupe_cloned().to_stream().then(hook),
            )
            .catch_unwind()
            .map_err(move |p| {
                log::crit!(
                    "Panicked executing `{}` hook of state: {}",
                    name,
                    display_panic(&p),
                );
            })
            .map(|_| Ok(()))
            .forward(sink::drain()),
        ));
    }

    /// Adds a new [`Client`] to this [`State`]
    ///
    /// # Errors
    ///
    /// If this [`State`] has a [`Client`] with the same host
    pub fn add_client(&self, client_id: &ClientId) -> anyhow::Result<()> {
        let mut clients = self.clients.lock_mut();

        if clients.iter().any(|r| r.id == *client_id) {
            return Err(anyhow!("Client host '{}' is used already", client_id));
        }

        clients.push(Client::new(client_id));

        Ok(())
    }

    /// Removes a [`Client`] with the given `id` from this [`State`].
    ///
    /// Returns [`None`] if there is no [`Client`] with such `id` in this
    /// [`State`].
    #[allow(clippy::must_use_candidate)]
    pub fn remove_client(&self, client_id: &ClientId) -> Option<()> {
        let mut clients = self.clients.lock_mut();
        let prev_len = clients.len();
        clients.retain(|r| r.id != *client_id);
        (clients.len() != prev_len).then_some(())
    }

    /// Adds a new [`Restream`] by the given `spec` to this [`State`].
    ///
    /// # Errors
    ///
    /// If this [`State`] has a [`Restream`] with such `key` already.
    pub fn add_restream(&self, spec: spec::v1::Restream) -> anyhow::Result<()> {
        let mut restreams = self.restreams.lock_mut();

        if restreams.iter().any(|r| r.key == spec.key) {
            return Err(anyhow!("Restream.key '{}' is used already", spec.key));
        }

        restreams.push(Restream::new(spec));
        Ok(())
    }

    /// Edits a [`Restream`] with the given `spec` identified by the given `id`
    /// in this [`State`].
    ///
    /// Returns [`None`] if there is no [`Restream`] with such `id` in this
    /// [`State`].
    ///
    /// # Errors
    ///
    /// If this [`State`] has a [`Restream`] with such `key` already.
    pub fn edit_restream(
        &self,
        id: RestreamId,
        spec: spec::v1::Restream,
    ) -> anyhow::Result<Option<()>> {
        let mut restreams = self.restreams.lock_mut();

        if restreams.iter().any(|r| r.key == spec.key && r.id != id) {
            return Err(anyhow!("Restream.key '{}' is used already", spec.key));
        }

        #[allow(clippy::manual_find_map)] // due to consuming `spec`
        Ok(restreams
            .iter_mut()
            .find(|r| r.id == id)
            .map(|r| r.apply(spec, false)))
    }

    /// Removes a [`Restream`] with the given `id` from this [`State`].
    ///
    /// Returns [`None`] if there is no [`Restream`] with such `id` in this
    /// [`State`].
    #[allow(clippy::must_use_candidate)]
    pub fn remove_restream(&self, id: RestreamId) -> Option<()> {
        let mut restreams = self.restreams.lock_mut();
        let prev_len = restreams.len();
        restreams.retain(|r| r.id != id);
        (restreams.len() != prev_len).then_some(())
    }

    /// Enables a [`Restream`] with the given `id` in this [`State`].
    ///
    /// Returns `true` if it has been enabled, or `false` if it already has been
    /// enabled, or [`None`] if it doesn't exist.
    #[must_use]
    pub fn enable_restream(&self, id: RestreamId) -> Option<bool> {
        self.restreams
            .lock_mut()
            .iter_mut()
            .find_map(|r| (r.id == id).then(|| r.input.enable()))
    }

    /// Disables a [`Restream`] with the given `id` in this [`State`].
    ///
    /// Returns `true` if it has been disabled, or `false` if it already has
    /// been disabled, or [`None`] if it doesn't exist.
    #[must_use]
    pub fn disable_restream(&self, id: RestreamId) -> Option<bool> {
        self.restreams
            .lock_mut()
            .iter_mut()
            .find_map(|r| (r.id == id).then(|| r.input.disable()))
    }

    /// Enables an [`Input`] with the given `id` in the specified [`Restream`]
    /// of this [`State`].
    ///
    /// Returns `true` if it has been enabled, or `false` if it already has been
    /// enabled, or [`None`] if it doesn't exist.
    #[must_use]
    pub fn enable_input(
        &self,
        id: InputId,
        restream_id: RestreamId,
    ) -> Option<bool> {
        self.restreams
            .lock_mut()
            .iter_mut()
            .find(|r| r.id == restream_id)?
            .input
            .find_mut(id)
            .map(Input::enable)
    }

    /// Disables an [`Input`] with the given `id` in the specified [`Restream`]
    /// of this [`State`].
    ///
    /// Returns `true` if it has been disabled, or `false` if it already has
    /// been disabled, or [`None`] if it doesn't exist.
    #[must_use]
    pub fn disable_input(
        &self,
        id: InputId,
        restream_id: RestreamId,
    ) -> Option<bool> {
        self.restreams
            .lock_mut()
            .iter_mut()
            .find(|r| r.id == restream_id)?
            .input
            .find_mut(id)
            .map(Input::disable)
    }

    /// Sets label on [`Input`] with the given `id` in
    /// the specified [`Restream`] of this [`State`].
    ///
    /// Returns `true` if it has been set, or `false` if it already has
    /// been set, or [`None`] if it doesn't exist.
    #[must_use]
    pub fn set_endpoint_label(
        &self,
        id: InputId,
        restream_id: RestreamId,
        endpoint_id: EndpointId,
        label: Option<Label>,
    ) -> Option<bool> {
        self.restreams
            .lock_mut()
            .iter_mut()
            .find(|r| r.id == restream_id)?
            .input
            .find_mut(id)?
            .endpoints
            .iter_mut()
            .find(|endpoint| endpoint.id == endpoint_id)
            .map(|mut ie| {
                if ie.label == label {
                    false
                } else {
                    ie.label = label;
                    true
                }
            })
    }

    /// Adds a new [`Output`] to the specified [`Restream`] of this [`State`].
    ///
    /// Returns [`None`] if there is no [`Restream`] with such `id` in this
    /// [`State`].
    ///
    /// # Errors
    ///
    /// If the [`Restream`] has an [`Output`] with such `dst` already.
    pub fn add_output(
        &self,
        restream_id: RestreamId,
        spec: spec::v1::Output,
    ) -> anyhow::Result<Option<()>> {
        let mut restreams = self.restreams.lock_mut();

        let outputs = if let Some(r) =
            restreams.iter_mut().find(|r| r.id == restream_id)
        {
            &mut r.outputs
        } else {
            return Ok(None);
        };

        if let Some(o) = outputs.iter().find(|o| o.dst == spec.dst) {
            return Err(anyhow!("Output.dst '{}' is used already", o.dst));
        }

        outputs.push(Output::new(spec));
        Ok(Some(()))
    }

    /// Edits an [`Output`] with the given `spec` identified by the given `id`
    /// in the specified [`Restream`] of this [`State`].
    ///
    /// Returns [`None`] if there is no [`Restream`] with such `restream_id` in
    /// this [`State`], or there is no [`Output`] with such `id`.
    ///
    /// # Errors
    ///
    /// If the [`Restream`] has an [`Output`] with such `dst` already.
    pub fn edit_output(
        &self,
        restream_id: RestreamId,
        id: OutputId,
        spec: spec::v1::Output,
    ) -> anyhow::Result<Option<()>> {
        let mut restreams = self.restreams.lock_mut();

        let outputs = if let Some(r) =
            restreams.iter_mut().find(|r| r.id == restream_id)
        {
            &mut r.outputs
        } else {
            return Ok(None);
        };

        if outputs.iter().any(|o| o.dst == spec.dst && o.id != id) {
            return Err(anyhow!("Output.dst '{}' is used already", spec.dst));
        }

        #[allow(clippy::manual_find_map)] // due to consuming `spec`
        Ok(outputs
            .iter_mut()
            .find(|o| o.id == id)
            .map(|o| o.apply(spec, true)))
    }

    /// Removes an [`Output`] with the given `id` from the specified
    /// [`Restream`] of this [`State`].
    ///
    /// Returns [`None`] if there is no [`Restream`] with such `restream_id` or
    /// no [`Output`] with such `id` in this [`State`].
    #[must_use]
    pub fn remove_output(
        &self,
        id: OutputId,
        restream_id: RestreamId,
    ) -> Option<()> {
        let mut restreams = self.restreams.lock_mut();
        let outputs =
            &mut restreams.iter_mut().find(|r| r.id == restream_id)?.outputs;

        let prev_len = outputs.len();
        outputs.retain(|o| o.id != id);
        (outputs.len() != prev_len).then_some(())
    }

    /// Enables an [`Output`] with the given `id` in the specified [`Restream`]
    /// of this [`State`].
    ///
    /// Returns `true` if it has been enabled, or `false` if it already has been
    /// enabled, or [`None`] if it doesn't exist.
    #[must_use]
    pub fn enable_output(
        &self,
        id: OutputId,
        restream_id: RestreamId,
    ) -> Option<bool> {
        let mut restreams = self.restreams.lock_mut();
        let output = restreams
            .iter_mut()
            .find(|r| r.id == restream_id)?
            .outputs
            .iter_mut()
            .find(|o| o.id == id)?;

        if output.enabled {
            return Some(false);
        }

        output.enabled = true;
        Some(true)
    }

    /// Disables an [`Output`] with the given `id` in the specified [`Restream`]
    /// of this [`State`].
    ///
    /// Returns `true` if it has been disabled, or `false` if it already has
    /// been disabled, or [`None`] if it doesn't exist.
    #[must_use]
    pub fn disable_output(
        &self,
        id: OutputId,
        restream_id: RestreamId,
    ) -> Option<bool> {
        let mut restreams = self.restreams.lock_mut();
        let output = restreams
            .iter_mut()
            .find(|r| r.id == restream_id)?
            .outputs
            .iter_mut()
            .find(|o| o.id == id)?;

        if !output.enabled {
            return Some(false);
        }

        output.enabled = false;
        Some(true)
    }

    /// Get [Output] from [Restream] by `restream_id` and `output_id`
    #[must_use]
    pub fn get_output(
        &self,
        restream_id: RestreamId,
        output_id: OutputId,
    ) -> Option<Output> {
        self.restreams
            .get_cloned()
            .into_iter()
            .find(|r| r.id == restream_id)?
            .outputs
            .into_iter()
            .find(|o| o.id == output_id)
    }

    /// Enables all [`Output`]s in the specified [`Restream`] of this [`State`].
    ///
    /// Returns `true` if at least one [`Output`] has been enabled, or `false`
    /// if all of them already have been enabled, or [`None`] if no [`Restream`]
    /// with such `restream_id` exists.
    #[must_use]
    pub fn enable_all_outputs(&self, restream_id: RestreamId) -> Option<bool> {
        self.set_state_of_all_outputs(restream_id, true)
    }

    /// Disables all [`Output`]s in the specified [`Restream`] of this
    /// [`State`].
    ///
    /// Returns `true` if at least one [`Output`] has been disabled, or `false`
    /// if all of them already have been disabled, or [`None`] if no
    /// [`Restream`] with such `restream_id` exists.
    #[must_use]
    pub fn disable_all_outputs(&self, restream_id: RestreamId) -> Option<bool> {
        self.set_state_of_all_outputs(restream_id, false)
    }

    /// Enables all [`Output`]s in all [`Restream`]s of this [`State`].
    ///
    /// Returns `true` if at least one [`Output`] has been enabled, or `false`
    /// if all of them already have been enabled or there are no outputs
    #[must_use]
    pub fn enable_all_outputs_of_restreams(&self) -> bool {
        self.set_state_of_all_outputs_of_restreams(true)
    }

    /// Disables all [`Output`]s in ALL [`Restream`]s of this [`State`].
    ///
    /// Returns `true` if at least one [`Output`] has been disabled, or `false`
    /// if all of them already have been disabled or there are no outputs
    #[must_use]
    pub fn disable_all_outputs_of_restreams(&self) -> bool {
        self.set_state_of_all_outputs_of_restreams(false)
    }

    /// Tunes a [`Volume`] rate of the specified [`Output`] or its [`Mixin`] in
    /// this [`State`].
    ///
    /// Returns `true` if a [`Volume`] rate has been changed, or `false` if it
    /// has the same value already.
    ///
    /// Returns [`None`] if no such [`Restream`]/[`Output`]/[`Mixin`] exists.
    #[must_use]
    pub fn tune_volume(
        &self,
        restream_id: RestreamId,
        output_id: OutputId,
        mixin_id: Option<MixinId>,
        volume: Volume,
    ) -> Option<bool> {
        let mut restreams = self.restreams.lock_mut();
        let output = restreams
            .iter_mut()
            .find(|r| r.id == restream_id)?
            .outputs
            .iter_mut()
            .find(|o| o.id == output_id)?;

        let curr_volume = if let Some(id) = mixin_id {
            &mut output.mixins.iter_mut().find(|m| m.id == id)?.volume
        } else {
            &mut output.volume
        };

        if *curr_volume == volume {
            return Some(false);
        }

        *curr_volume = volume;
        Some(true)
    }

    /// Tunes a [`Delay`] of the specified [`Mixin`] in this [`State`].
    ///
    /// Returns `true` if a [`Delay`] has been changed, or `false` if it has the
    /// same value already.
    ///
    /// Returns [`None`] if no such [`Restream`]/[`Output`]/[`Mixin`] exists.
    #[must_use]
    pub fn tune_delay(
        &self,
        input_id: RestreamId,
        output_id: OutputId,
        mixin_id: MixinId,
        delay: Delay,
    ) -> Option<bool> {
        let mut restreams = self.restreams.lock_mut();
        let mixin = restreams
            .iter_mut()
            .find(|r| r.id == input_id)?
            .outputs
            .iter_mut()
            .find(|o| o.id == output_id)?
            .mixins
            .iter_mut()
            .find(|m| m.id == mixin_id)?;

        if mixin.delay == delay {
            return Some(false);
        }

        mixin.delay = delay;
        Some(true)
    }

    /// Tunes a the specified [`Mixin.sidechain`] in this [`State`].
    ///
    /// Returns `true` if a [`Mixin.sidechain`] has been changed, or `false`
    /// if it has the same value already.
    ///
    /// Returns [`None`] if no such [`Restream`]/[`Output`]/[`Mixin`] exists.
    #[must_use]
    pub fn tune_sidechain(
        &self,
        input_id: RestreamId,
        output_id: OutputId,
        mixin_id: MixinId,
        sidechain: bool,
    ) -> Option<bool> {
        let mut restreams = self.restreams.lock_mut();
        let mixin = restreams
            .iter_mut()
            .find(|r| r.id == input_id)?
            .outputs
            .iter_mut()
            .find(|o| o.id == output_id)?
            .mixins
            .iter_mut()
            .find(|m| m.id == mixin_id)?;

        if mixin.sidechain == sidechain {
            return Some(false);
        }

        mixin.sidechain = sidechain;
        Some(true)
    }

    /// Clean up stream statistics info
    pub fn cleanup_stream_info(&self) {
        let mut restreams = self.restreams.lock_mut();
        restreams.iter_mut().for_each(|r| {
            if let Some(InputSrc::Failover(s)) = &mut r.input.src {
                for mut e in
                    s.inputs.iter_mut().flat_map(|i| i.endpoints.iter_mut())
                {
                    if e.status == Status::Offline {
                        e.stream_stat = None;
                    }
                }
            }
        });
    }

    /// Updates info about stream for [`InputEndpoint`]
    ///
    /// # Errors
    ///
    /// If endpoint with specified `id` is not found.
    /// Or if audio stream not found in [`StreamInfo`]
    /// Or if video stream not found in [`StreamInfo`]
    pub fn set_stream_info(
        &self,
        id: EndpointId,
        info: StreamInfo,
    ) -> anyhow::Result<()> {
        fn find_stream(
            stream_type: &str,
            stream_info: &StreamInfo,
        ) -> Option<Stream> {
            stream_info.streams.clone().into_iter().find(|x| {
                x.codec_type.clone().unwrap_or_default() == stream_type
            })
        }

        let mut restreams = self.restreams.lock_mut();
        let endpoint = restreams
            .iter_mut()
            .find_map(|r| r.input.find_endpoint(id))
            .ok_or_else(|| anyhow!("Can't find endpoint with id: {:?}", id))?;

        log::debug!("TEST: found endpoint {:?}", endpoint);

        let audio_stream = find_stream("audio", &info)
            .ok_or_else(|| anyhow!("Can't find 'audio' stream"))?;
        let video_stream = find_stream("video", &info)
            .ok_or_else(|| anyhow!("Can't find 'video' stream"))?;

        let stream_stat = StreamStatistics {
            audio_codec_name: audio_stream.codec_name,
            audio_channel_layout: audio_stream.channel_layout,
            audio_sample_rate: audio_stream.sample_rate,
            audio_channels: audio_stream
                .channels
                .map(|x| UNumber::new(x.into())),
            video_codec_name: video_stream.codec_name,
            video_r_frame_rate: video_stream.r_frame_rate,
            video_width: video_stream.width.map(UNumber::new),
            video_height: video_stream.height.map(UNumber::new),
            bit_rate: info.format.bit_rate,
        };

        endpoint.stream_stat = Some(stream_stat);
        Ok(())
    }

    /// Gather statistics about [`Input`]s statuses
    #[must_use]
    pub fn get_inputs_statistics(&self) -> Vec<StatusStatistics> {
        self.restreams
            .get_cloned()
            .into_iter()
            .fold(HashMap::new(), |mut stat, restream| {
                let item =
                    restream.input.endpoints.iter().find(|e| e.is_rtmp());
                match item {
                    Some(main_input) => {
                        Self::update_stat(&mut stat, main_input.status);
                    }
                    None => log::error!(
                        "Main endpoint not found for {} input",
                        restream.input.id
                    ),
                };

                stat
            })
            .into_iter()
            .map(|x| StatusStatistics {
                status: x.0,
                count: x.1,
            })
            .collect()
    }

    /// Gather statistics about [`Output`]s statuses
    #[must_use]
    pub fn get_outputs_statistics(&self) -> Vec<StatusStatistics> {
        self.restreams
            .get_cloned()
            .into_iter()
            .flat_map(|r| r.outputs.into_iter())
            .fold(HashMap::new(), |mut stat, output| {
                Self::update_stat(&mut stat, output.status);
                stat
            })
            .into_iter()
            .map(|x| StatusStatistics {
                status: x.0,
                count: x.1,
            })
            .collect()
    }

    /// Statistics for statuses of this [`Client`]
    #[must_use]
    pub fn get_statistics(&self) -> ClientStatistics {
        let settings = self.settings.get_cloned();
        let title = match settings.title {
            Some(t) => t,
            None => String::new(),
        };

        let inputs_stat = self.get_inputs_statistics();
        let outputs_stat = self.get_outputs_statistics();
        ClientStatistics::new(
            title,
            inputs_stat,
            outputs_stat,
            self.server_info.lock_mut().clone(),
        )
    }

    fn update_stat(stat: &mut HashMap<Status, i32>, status: Status) {
        if let Some(x) = stat.get_mut(&status) {
            *x += 1;
        } else {
            let _ = stat.insert(status, 1);
        }
    }

    /// Disables/Enables all [`Output`]s in the specified [`Restream`] of this
    /// [`State`].
    #[must_use]
    fn set_state_of_all_outputs(
        &self,
        restream_id: RestreamId,
        enabled: bool,
    ) -> Option<bool> {
        let mut restreams = self.restreams.lock_mut();
        Some(
            restreams
                .iter_mut()
                .find(|r| r.id == restream_id)?
                .outputs
                .iter_mut()
                .filter(|o| o.enabled != enabled)
                .fold(false, |_, o| {
                    o.enabled = enabled;
                    true
                }),
        )
    }

    /// Disables/Enables all [`Output`]s in ALL [`Restream`]s of this [`State`].
    #[must_use]
    fn set_state_of_all_outputs_of_restreams(&self, enabled: bool) -> bool {
        let mut restreams = self.restreams.lock_mut();
        restreams
            .iter_mut()
            .flat_map(|r| r.outputs.iter_mut())
            .filter(|o| o.enabled != enabled)
            .fold(false, |_, o| {
                o.enabled = enabled;
                true
            })
    }
}

/// Specifies kind of password
#[derive(Clone, Copy, Debug, Eq, GraphQLEnum, PartialEq)]
pub enum PasswordKind {
    /// Password for main application
    Main,

    /// Password for single output application
    Output,
}

/// Status indicating availability of an `Input`, `Output`, or a `Mixin`.
#[derive(
    Clone, Copy, Debug, Eq, GraphQLEnum, PartialEq, SmartDefault, Hash,
)]
pub enum Status {
    /// Inactive, no operations are performed and no media traffic is flowed.
    #[default]
    Offline,

    /// Initializing, media traffic doesn't yet flow as expected.
    Initializing,

    /// Active, all operations are performing successfully and media traffic
    /// flows as expected.
    Online,

    /// Failed recently
    Unstable,
}
