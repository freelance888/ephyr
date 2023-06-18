//! Broadcaster for dashboard commands

use crate::{
    console_logger::{ConsoleLogger, ConsoleMessageKind, ConsoleMessageSource},
    display_panic,
    file_manager::FileId,
    state::ClientId,
    State,
};
use derive_more::Display;
use ephyr_log::{
    tracing,
    tracing::{instrument, Instrument},
};
use futures::{FutureExt, TryFutureExt};
use graphql_client::{GraphQLQuery, Response};
use reqwest;
use std::{future::Future, panic::AssertUnwindSafe};

/// Set of dashboard commands that can be broadcast to clients
#[derive(Clone, Debug, PartialEq, Display, Eq)]
pub enum DashboardCommand {
    /// Command for enabling all restreams' outputs
    EnableAllOutputs(),
    /// Command for disabling all restreams' outputs
    DisableAllOutputs(),
    /// Command for initiation playing specific file on any of registered
    /// client
    StartPlayingFile(String),
    /// Command for stop playing specific file on any of registered
    /// client
    StopPlayingFile(String),
}

/// GraphQL mutation for enabling outputs
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "client.graphql.schema.json",
    query_path = "src/api/graphql/queries/enable_outputs.graphql",
    response_derives = "Debug"
)]
#[derive(Debug)]
pub(crate) struct EnableAllOutputsOfRestreams;

/// GraphQL mutation for disabling outputs
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "client.graphql.schema.json",
    query_path = "src/api/graphql/queries/disable_outputs.graphql",
    response_derives = "Debug"
)]
#[derive(Debug)]
pub(crate) struct DisableAllOutputsOfRestreams;

/// GraphQL mutation for start playing file
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "client.graphql.schema.json",
    query_path = "src/api/graphql/queries/start_playing_file.graphql",
    response_derives = "Debug"
)]
#[derive(Debug)]
pub(crate) struct StartPlayingFile;

/// GraphQL mutation for stop playing file
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "client.graphql.schema.json",
    query_path = "src/api/graphql/queries/stop_playing_file.graphql",
    response_derives = "Debug"
)]
#[derive(Debug)]
pub(crate) struct StopPlayingFile;

/// Broadcast [`DashboardCommand`] to clients
#[derive(Debug, Default)]
pub struct Broadcaster {
    state: State,
}

impl Broadcaster {
    /// Creates new [`Broadcaster`]
    #[inline]
    #[must_use]
    pub fn new(state: State) -> Self {
        Self { state }
    }

    /// Processes all commands from queue
    #[instrument(skip_all, name = "broadcaster::handle_commands")]
    pub fn handle_commands(&mut self) {
        // Pops all existing command from queue
        let commands: Vec<DashboardCommand> =
            self.state.dashboard_commands.lock_mut().drain(..).collect();

        let state = self.state.clone();

        // We send commands only to clients protected by password,
        // i.e having base auth url
        state
            .clients
            .lock_mut()
            .iter()
            //.filter(|client| client.is_protected)
            .for_each(|client| {
                for command in &commands {
                    self.handle_one_command(client.id.clone(), command.clone());
                }
            });
    }

    fn handle_one_command(
        &mut self,
        client_id: ClientId,
        command: DashboardCommand,
    ) {
        match command {
            DashboardCommand::EnableAllOutputs() => {
                let state = self.state.clone();
                Self::try_to_run_command(
                    client_id.clone(),
                    state.clone(),
                    async move {
                        Self::request_enable_outputs(client_id, state).await
                    },
                );
            }
            DashboardCommand::DisableAllOutputs() => {
                let state = self.state.clone();
                Self::try_to_run_command(
                    client_id.clone(),
                    state.clone(),
                    async move {
                        Self::request_disable_outputs(client_id, state).await
                    },
                );
            }
            DashboardCommand::StartPlayingFile(name_prefix) => {
                let state = self.state.clone();
                Self::try_to_run_command(
                    client_id.clone(),
                    state.clone(),
                    async move {
                        Self::request_start_playing_file(
                            client_id,
                            name_prefix,
                            state,
                        )
                        .await
                    },
                );
            }
            DashboardCommand::StopPlayingFile(name_prefix) => {
                let state = self.state.clone();
                Self::try_to_run_command(
                    client_id.clone(),
                    state.clone(),
                    async move {
                        Self::request_stop_playing_file(
                            client_id,
                            name_prefix,
                            state,
                        )
                        .await
                    },
                );
            }
        }
    }

    fn try_to_run_command<FutureCommand>(
        client_id: ClientId,
        state: State,
        command: FutureCommand,
    ) where
        FutureCommand: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        drop(tokio::spawn(
            async move {
                _ = AssertUnwindSafe(command.unwrap_or_else(|e| {
                    let error_message =
                        format!("Error sending command for client. {e}");
                    tracing::error!(error_message);
                    Self::save_command_error(
                        &client_id,
                        &[error_message],
                        &state,
                    );
                }))
                .catch_unwind()
                .await
                .map_err(|p| {
                    tracing::error!(
                        "Panicked while broadcast command to client: {}",
                        display_panic(&p)
                    );
                });
            }
            .in_current_span(),
        ));
    }

    async fn request_enable_outputs(
        client_id: ClientId,
        state: State,
    ) -> anyhow::Result<()> {
        type Vars = <EnableAllOutputsOfRestreams as GraphQLQuery>::Variables;
        type ResponseData =
            <EnableAllOutputsOfRestreams as GraphQLQuery>::ResponseData;

        let request_body = EnableAllOutputsOfRestreams::build_query(Vars {});

        let request = reqwest::Client::builder().build().unwrap();

        let url = format!("{client_id}api");
        let res = request
            .post(url.as_str())
            .json(&request_body)
            .send()
            .await?;

        let response: Response<ResponseData> = res.json().await?;
        tracing::info!(?response, "Enabling outputs on client",);

        Self::handle_errors(&client_id, &state, response.errors);
        Ok(())
    }

    async fn request_stop_playing_file(
        client_id: ClientId,
        name_prefix: String,
        state: State,
    ) -> anyhow::Result<()> {
        type Vars = <StopPlayingFile as GraphQLQuery>::Variables;
        type ResponseData = <StopPlayingFile as GraphQLQuery>::ResponseData;

        let request_body = StopPlayingFile::build_query(Vars {
            name_prefix: name_prefix.clone(),
        });

        let request = reqwest::Client::builder().build().unwrap();

        let url = format!("{client_id}api");
        let res = request
            .post(url.as_str())
            .json(&request_body)
            .send()
            .await?;

        let response: Response<ResponseData> = res.json().await?;
        tracing::info!(
            ?response,
            "Stop playing file starting with '{name_prefix}'",
        );

        Self::handle_errors(&client_id, &state, response.errors);
        Ok(())
    }

    async fn request_start_playing_file(
        client_id: ClientId,
        name_prefix: String,
        state: State,
    ) -> anyhow::Result<()> {
        type Vars = <StartPlayingFile as GraphQLQuery>::Variables;
        type ResponseData = <StartPlayingFile as GraphQLQuery>::ResponseData;

        let request_body = StartPlayingFile::build_query(Vars {
            name_prefix: name_prefix.clone(),
        });

        let request = reqwest::Client::builder().build().unwrap();

        let url = format!("{client_id}api");
        let res = request
            .post(url.as_str())
            .json(&request_body)
            .send()
            .await?;

        let response: Response<ResponseData> = res.json().await?;
        tracing::info!(
            ?response,
            "Start playing file starting with '{name_prefix}'",
        );

        Self::handle_errors(&client_id, &state, response.errors);
        Ok(())
    }

    async fn request_disable_outputs(
        client_id: ClientId,
        state: State,
    ) -> anyhow::Result<()> {
        type Vars = <DisableAllOutputsOfRestreams as GraphQLQuery>::Variables;
        type ResponseData =
            <DisableAllOutputsOfRestreams as GraphQLQuery>::ResponseData;

        let request_body = DisableAllOutputsOfRestreams::build_query(Vars {});

        let request = reqwest::Client::builder().build().unwrap();

        let url = format!("{client_id}api");
        let res = request
            .post(url.as_str())
            .json(&request_body)
            .send()
            .await?;

        let response: Response<ResponseData> = res.json().await?;
        tracing::info!(?response, "Disabling outputs on client",);

        Self::handle_errors(&client_id, &state, response.errors);
        Ok(())
    }

    fn handle_errors(
        client_id: &ClientId,
        state: &State,
        errors: Option<Vec<graphql_client::Error>>,
    ) {
        //  Consider better error handling with ability to constant
        //  displaying error on dashboard
        if errors.is_some() {
            let response_errors: Vec<String> = errors
                .unwrap_or_default()
                .into_iter()
                .map(|e| e.message)
                .collect();

            Self::save_command_error(client_id, &response_errors, state);
        }
    }

    /// Saves error in [`State`] for specific [`Client`]
    fn save_command_error(
        client_id: &ClientId,
        error_messages: &[String],
        state: &State,
    ) {
        let err_message =
            format!("{}: {}", client_id, error_messages.join(", "));
        tracing::error!(err_message);
        ConsoleLogger::new(state.clone()).log_message(
            err_message,
            ConsoleMessageKind::Err,
            ConsoleMessageSource::Dashboard,
        );
    }
}
