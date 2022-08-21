//! Broadcaster for dashboard commands

use crate::state::{ClientId, ClientStatisticsResponse, DashboardCommand};
use crate::{client_stat, display_panic, State};
use ephyr_log::log;
use futures::{FutureExt, TryFutureExt};
use std::panic::AssertUnwindSafe;

#[derive(Debug, Default)]
pub struct Broadcaster {
    state: State,
}

impl Broadcaster {
    /// Creates new pull of [`Broadcaster`]
    #[inline]
    #[must_use]
    pub fn new(state: State) -> Self {
        Self { state }
    }

    /// Processes all commands in queue
    pub async fn handle_commands(&mut self) {
        // pops all existing command from queue
        let commands: Vec<DashboardCommand> =
            self.state.dashboard_commands.lock_mut().drain(..).collect();

        let state = self.state.clone();
        let mut clients = state.clients.lock_mut();

        for command in commands {
            for client in clients.iter() {
                self.handle_one_command(&client.id, &command).await
            }
        }
    }

    async fn handle_one_command(
        &mut self,
        client_id: &ClientId,
        command: &DashboardCommand,
    ) {
        match command {
            DashboardCommand::PlayFile(c) => {
                self.try_play_file(client_id, &c.file_id).await
            }
        }
    }

    async fn try_play_file(&mut self, client_id: &ClientId, file_id: &String) {
        let _ = AssertUnwindSafe(
            async move { Self::request_play_file(client_id, &file_id).await }
                .unwrap_or_else(|e| {
                    let error_message = format!(
                        "Error sending play file command for client {}. {}",
                        client_id, e
                    );

                    log::error!("{}", error_message);
                    client_stat::save_client_error(
                        client_id,
                        error_message,
                        &self.state,
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
    }

    async fn request_play_file(
        client_id: &ClientId,
        file_id: &String,
    ) -> anyhow::Result<()> {
        log::info!("PLAY FILE {} for client {}", file_id, client_id);
        Ok(())
    }
}
