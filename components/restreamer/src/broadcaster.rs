//! Broadcaster for dashboard commands

use crate::{
    display_panic,
    state::{ClientId, ClientStatisticsResponse},
    State,
};
use ephyr_log::log;
use futures::{FutureExt, TryFutureExt};
use graphql_client::{GraphQLQuery, Response};
use reqwest;
use std::{future::Future, panic::AssertUnwindSafe};

/// Set of dashboard commands that can be broadcast to clients
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DashboardCommand {
    /// Command for enabling all restreams' outputs
    EnableAllOutputs(),
    /// Command for disabling all restreams' outputs
    DisableAllOutputs(),
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
    pub fn handle_commands(&mut self) {
        // Pops all existing command from queue
        let commands: Vec<DashboardCommand> =
            self.state.dashboard_commands.lock_mut().drain(..).collect();

        let state = self.state.clone();

        // We send command only clients protected by password,
        // i.e having base auth url
        state
            .clients
            .lock_mut()
            .iter()
            .filter(|client| client.is_protected)
            .for_each(|client| {
                for command in &commands {
                    self.handle_one_command(
                        &client.id.clone(),
                        &command.clone(),
                    );
                }
            });
    }

    fn handle_one_command(
        &mut self,
        client_id: &ClientId,
        command: &DashboardCommand,
    ) {
        match command {
            DashboardCommand::EnableAllOutputs() => {
                let client_id = client_id.clone();
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
                let client_id = client_id.clone();
                let state = self.state.clone();
                Self::try_to_run_command(
                    client_id.clone(),
                    state.clone(),
                    async move {
                        Self::request_disable_outputs(client_id, state).await
                    },
                );
            }
        }
    }

    fn try_to_run_command<
        FutureCommand: Future<Output = anyhow::Result<()>> + Send + 'static,
    >(
        client_id: ClientId,
        state: State,
        command: FutureCommand,
    ) {
        drop(tokio::spawn(async move {
            let _ = AssertUnwindSafe(command.unwrap_or_else(|e| {
                let error_message = format!(
                    "Error sending command for client {}. {}",
                    client_id, e
                );
                log::error!("{}", error_message);
                Self::save_command_error(
                    &client_id,
                    vec![error_message],
                    &state,
                );
            }))
            .catch_unwind()
            .await
            .map_err(|p| {
                log::error!(
                    "{}",
                    format!(
                        "Panicked while broadcast command to client: {}",
                        display_panic(&p)
                    )
                );
            });
        }));
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

        let url = format!("{}api", client_id);
        let res = request
            .post(url.as_str())
            .json(&request_body)
            .send()
            .await?;

        let response: Response<ResponseData> = res.json().await?;
        log::info!(
            "Enabling outputs on client: {}. Response: {:#?}",
            client_id,
            response
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

        let url = format!("{}api", client_id);
        let res = request
            .post(url.as_str())
            .json(&request_body)
            .send()
            .await?;

        let response: Response<ResponseData> = res.json().await?;
        log::info!(
            "Disabling outputs on client: {}. Response: {:#?}",
            client_id,
            response
        );

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

            Self::save_command_error(client_id, response_errors, state);
        }
    }

    /// Saves error in [`State`] for specific [`Client`]
    fn save_command_error(
        client_id: &ClientId,
        error_messages: Vec<String>,
        state: &State,
    ) {
        let mut clients = state.clients.lock_mut();
        if let Some(c) = clients.iter_mut().find(|r| &r.id == client_id) {
            c.statistics = Some(ClientStatisticsResponse {
                data: None,
                errors: Some(error_messages),
            });
        };
    }
}
