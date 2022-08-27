//! Broadcaster for dashboard commands

use crate::{
    client_stat::save_client_error, display_panic, state::ClientId, State,
};
use ephyr_log::log;
use futures::{FutureExt, TryFutureExt};
use graphql_client::{GraphQLQuery, Response};
use reqwest;
use std::panic::AssertUnwindSafe;

/// Set of dashboard commands that can be broadcast to clients
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DashboardCommand {
    /// Command for start playing file
    PlayFile(PlayFileCommand),
}

/// Broadcast command for playing file on any restream of any client
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayFileCommand {
    /// File identity
    pub file_id: String,
}

/// GraphQL mutation for sending play file command
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "client.graphql.schema.json",
    query_path = "src/api/graphql/queries/play_file.graphql",
    response_derives = "Debug"
)]
#[derive(Debug)]
pub(crate) struct BroadcastPlayFile;

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
        let clients = state.clients.lock_mut();

        for command in commands {
            for client in clients.iter() {
                self.handle_one_command(&client.id, &command);
            }
        }
    }

    fn handle_one_command(
        &mut self,
        client_id: &ClientId,
        command: &DashboardCommand,
    ) {
        match command {
            DashboardCommand::PlayFile(c) => {
                self.try_play_file(client_id, &c.file_id);
            }
        }
    }

    fn try_play_file(&mut self, client_id: &ClientId, file_id: &str) {
        let client_id1 = client_id.clone();
        let file_id1 = file_id.to_string();
        let state1 = self.state.clone();

        drop(tokio::spawn(async move {
            let _ = AssertUnwindSafe(
                async {
                    Self::request_play_file(&client_id1, &file_id1, &state1)
                        .await
                }
                .unwrap_or_else(|e| {
                    let error_message = format!(
                        "Error sending play file command for client {}. {}",
                        &client_id1, e
                    );

                    log::error!("{}", error_message);
                    save_client_error(
                        &client_id1,
                        vec![error_message],
                        &state1,
                    );
                }),
            )
            .catch_unwind()
            .await
            .map_err(|p| {
                let error_message = format!(
                    "Panicked while broadcast play file command to client: {}",
                    display_panic(&p)
                );
                log::error!("{}", error_message);
            });
        }));
    }

    async fn request_play_file(
        client_id: &ClientId,
        file_id: &str,
        state: &State,
    ) -> anyhow::Result<()> {
        type Vars = <BroadcastPlayFile as GraphQLQuery>::Variables;
        type ResponseData = <BroadcastPlayFile as GraphQLQuery>::ResponseData;

        let request_body = BroadcastPlayFile::build_query(Vars {
            file_id: file_id.to_string(),
        });
        let request = reqwest::Client::builder().build().unwrap();

        let url = format!("{}api", client_id);
        let res = request
            .post(url.as_str())
            .json(&request_body)
            .send()
            .await?;

        let response: Response<ResponseData> = res.json().await?;
        log::info!(
            "Sending play file to client: {}. Response: {:#?}",
            client_id,
            response
        );

        // TODO: Consider better error handling with ability to constant
        //  displaying error on dashboard
        if response.errors.is_some() {
            let response_errors: Vec<String> = response
                .errors
                .unwrap_or_default()
                .into_iter()
                .map(|e| e.message)
                .collect();

            save_client_error(client_id, response_errors, state);
        }

        Ok(())
    }
}
