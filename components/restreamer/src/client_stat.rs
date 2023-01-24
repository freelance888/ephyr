//! Clients statistics

// This is required because of `graphql_client` crate generate module for
// graphql query without documentation and that causes warning messages
#![allow(missing_docs)]

use std::{collections::HashMap, panic::AssertUnwindSafe, time::Duration};

use crate::{
    display_panic,
    state::{
        Client, ClientId, ClientStatistics, ClientStatisticsResponse, Status,
        StatusStatistics,
    },
    types::DroppableAbortHandle,
    State,
};
use ephyr_log::log;
use futures::{future, FutureExt as _, TryFutureExt};
use tokio::time;

use crate::client_stat::statistics_query::{
    StatisticsQueryStatisticsInputs, StatisticsQueryStatisticsOutputs,
    StatisticsQueryStatisticsServerInfo,
};

use crate::state::ServerInfo;
use graphql_client::{GraphQLQuery, Response};
use reqwest;

/// Poll of [`ClientJob`]s for getting statistics info from each [`Client`]
#[derive(Debug)]
pub struct ClientJobsPool {
    /// Pool of [`ClientJob`]s
    pool: HashMap<ClientId, ClientJob>,

    /// Application [`State`]
    state: State,
}

impl ClientJobsPool {
    /// Creates new pull of [`ClientJob`]
    #[inline]
    #[must_use]
    pub fn new(state: State) -> Self {
        Self {
            pool: HashMap::new(),
            state,
        }
    }

    /// Creates new [`ClientJob`] for added [`Client`] and removes for
    /// deleted [`Client`]
    pub fn start_statistics_loop(&mut self, clients: &[Client]) {
        let mut new_pool = HashMap::with_capacity(self.pool.len() + 1);

        for c in clients {
            let client_id = c.id.clone();
            let job = self.pool.remove(&client_id).unwrap_or_else(|| {
                ClientJob::gather_statistics(c.id.clone(), self.state.clone())
            });

            drop(new_pool.insert(client_id, job));
        }

        self.pool = new_pool;
    }
}

/// GraphQL query for getting client statistics
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "statistics.graphql.schema.json",
    query_path = "src/api/graphql/queries/client_stat.graphql",
    response_derives = "Debug"
)]
#[derive(Debug)]
pub struct StatisticsQuery;

#[allow(clippy::cast_possible_truncation)]
impl From<StatisticsQueryStatisticsServerInfo> for ServerInfo {
    fn from(item: StatisticsQueryStatisticsServerInfo) -> Self {
        let cpu_cores_unwrapped = item.cpu_cores.unwrap_or(0);

        ServerInfo {
            cpu_usage: item.cpu_usage,
            cpu_cores: Some(cpu_cores_unwrapped as i32),
            ram_total: item.ram_total,
            ram_free: item.ram_free,
            rx_delta: item.rx_delta,
            tx_delta: item.tx_delta,
            error_msg: item.error_msg,
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
impl From<StatisticsQueryStatisticsInputs> for StatusStatistics {
    fn from(item: StatisticsQueryStatisticsInputs) -> Self {
        StatusStatistics {
            status: item.status.into(),
            count: item.count as i32,
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
impl From<StatisticsQueryStatisticsOutputs> for StatusStatistics {
    fn from(item: StatisticsQueryStatisticsOutputs) -> Self {
        StatusStatistics {
            status: item.status.into(),
            count: item.count as i32,
        }
    }
}

impl From<statistics_query::Status> for Status {
    fn from(status: statistics_query::Status) -> Self {
        match status {
            statistics_query::Status::ONLINE => Status::Online,
            statistics_query::Status::OFFLINE => Status::Offline,
            statistics_query::Status::INITIALIZING => Status::Initializing,
            statistics_query::Status::UNSTABLE => Status::Unstable,
            statistics_query::Status::Other(other) => {
                panic!("Unknown status {}", other)
            }
        }
    }
}

/// Job for retrieving statistics from client from specific host
/// i.e [`ClientId`]
#[derive(Debug)]
pub struct ClientJob {
    /// Callback for stop job
    _abort: DroppableAbortHandle,
}

impl ClientJob {
    /// Spawns new future for getting client statistics from [`Client`]
    #[must_use]
    pub fn gather_statistics(id: ClientId, state: State) -> Self {
        let client_id1 = id.clone();

        let (spawner, abort_handle) = future::abortable(async move {
            loop {
                let client_id = &id;
                let state1 = &state.clone();
                let _ =
                    AssertUnwindSafe(
                        async move {
                            Self::fetch_client_stat(client_id, state1).await
                        }
                        .unwrap_or_else(|e| {
                            let error_message = format!(
                                "Error retrieving data for client {}. {}",
                                client_id, e
                            );

                            log::error!("{}", error_message);
                            save_client_error(
                                client_id,
                                vec![error_message],
                                state1,
                            );
                        }),
                    )
                    .catch_unwind()
                    .await
                    .map_err(|p| {
                        let error_message = format!(
                            "Panicked while getting statistics from client: {}",
                            display_panic(&p)
                        );
                        log::error!("{}", error_message);
                    });

                time::sleep(Duration::from_secs(2)).await;
            }
        });

        // Spawn periodic job for gathering info from client
        drop(tokio::spawn(spawner.map(move |_| {
            log::info!(
                "Client {} removed. Stop getting statistics",
                client_id1
            );
        })));

        Self {
            _abort: DroppableAbortHandle::new(abort_handle),
        }
    }

    async fn fetch_client_stat(
        client_id: &ClientId,
        state: &State,
    ) -> anyhow::Result<()> {
        type Vars = <StatisticsQuery as GraphQLQuery>::Variables;
        type ResponseData = <StatisticsQuery as GraphQLQuery>::ResponseData;

        log::info!("Getting statistics from client: {}", client_id);

        let request_body = StatisticsQuery::build_query(Vars {});
        let request = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let url = format!("{client_id}api-statistics");
        let res = request
            .post(url.as_str())
            .json(&request_body)
            .send()
            .await?;

        let response: Response<ResponseData> = res.json().await?;
        save_client_statistics(client_id, response, state);
        Ok(())
    }
}

/// Saves error in [`State`] for specific [`Client`]
///
/// # Panics
/// if [`Client`] is not found
pub fn save_client_error(
    client_id: &ClientId,
    error_messages: Vec<String>,
    state: &State,
) {
    let mut clients = state.clients.lock_mut();
    let client = match clients.iter_mut().find(|r| r.id == *client_id) {
        Some(c) => c,
        None => panic!("Client with id = {} was not found", client_id),
    };

    client.statistics = Some(ClientStatisticsResponse {
        data: None,
        errors: Some(error_messages),
    });
}

/// Saves [`Client`] statistics result in [`State`]
///
/// # Panics
/// if [`Client`] is not found
pub fn save_client_statistics(
    client_id: &ClientId,
    response: Response<<StatisticsQuery as GraphQLQuery>::ResponseData>,
    state: &State,
) {
    let response_errors: Vec<String> = response
        .errors
        .unwrap_or_default()
        .into_iter()
        .map(|e| e.message)
        .collect();

    let mut clients = state.clients.lock_mut();
    let client = match clients.iter_mut().find(|r| r.id == *client_id) {
        Some(c) => c,
        None => panic!("Client with id = {} was not found", client_id),
    };

    client.statistics = match response.data {
        Some(data) => Some(ClientStatisticsResponse {
            data: Some(ClientStatistics::new(
                data.statistics.client_title,
                data.statistics.inputs.into_iter().map(Into::into).collect(),
                data.statistics
                    .outputs
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                data.statistics.server_info.into(),
            )),
            errors: Some(response_errors),
        }),
        None => Some(ClientStatisticsResponse {
            data: None,
            errors: Some(response_errors),
        }),
    };
}
