//! Clients statistics

// This is required because of graphql_client generate
// module for graphql query without documentation and that causes warning messages
#![allow(missing_docs)]

use std::{collections::HashMap, panic::AssertUnwindSafe, time::Duration};

use crate::state::{
    Client, ClientId, ClientStatistics, ClientStatisticsResponse,
    StatusStatistics,
};
use crate::types::DroppableAbortHandle;
use crate::{display_panic, State};
use ephyr_log::log;
use futures::{future, FutureExt as _, TryFutureExt};
use tokio::time;

use crate::client_stat::statistics_query::StatisticsQueryStatisticsInputs;
use crate::client_stat::statistics_query::StatisticsQueryStatisticsOutputs;
use chrono::{DateTime, Utc};
use graphql_client::{GraphQLQuery, Response};
use reqwest;

/// Poll of [`ClientJob`]s for getting statistics info from each [`Client`]
///
#[derive(Debug)]
pub struct ClientJobsPool {
    /// Pool of [`ClientJob`]s
    pool: HashMap<String, ClientJob>,

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

    /// Creates new [`ClientJob`] for added [`Client`] and removes for deleted [`Client`]
    pub fn apply(&mut self, clients: &[Client]) {
        let mut new_pool = HashMap::with_capacity(self.pool.len() + 1);

        for c in clients {
            let client_id = c.id.clone().into();
            let job = self.pool.remove(&client_id).unwrap_or_else(|| {
                ClientJob::run(c.id.clone(), self.state.clone())
            });

            drop(new_pool.insert(client_id, job));
        }

        self.pool = new_pool;
    }
}

type DateTimeUtc = DateTime<Utc>;

/// GrapthQL query for getting client statistics
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "client.graphql.schema.json",
    query_path = "src/api/graphql/queries/client_stat.graphql",
    response_derives = "Debug"
)]
#[derive(Debug)]
pub struct StatisticsQuery;

impl From<StatisticsQueryStatisticsInputs> for StatusStatistics {
    fn from(item: StatisticsQueryStatisticsInputs) -> Self {
        StatusStatistics {
            initializing: item.initializing as i32,
            online: item.online as i32,
            offline: item.offline as i32,
            unstable: item.unstable as i32,
        }
    }
}

impl From<StatisticsQueryStatisticsOutputs> for StatusStatistics {
    fn from(item: StatisticsQueryStatisticsOutputs) -> Self {
        StatusStatistics {
            initializing: item.initializing as i32,
            online: item.online as i32,
            offline: item.offline as i32,
            unstable: item.unstable as i32,
        }
    }
}

/// Job for retrieving statistics from client from specific `ip`
#[derive(Debug)]
pub struct ClientJob {
    /// identity of client
    id: ClientId,

    /// Callback for stop job
    abort: DroppableAbortHandle,
}

impl ClientJob {
    /// Spawns new future for getting client statistics from [`Client`]
    #[must_use]
    pub fn run(id: ClientId, state: State) -> Self {
        let client_id1 = id.clone();
        let client_id2 = id.clone();

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

                            log::info!("{}", error_message);
                            Self::save_client_error(
                                client_id,
                                error_message,
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

                time::delay_for(Duration::from_secs(2)).await;
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
            id: client_id2,
            abort: DroppableAbortHandle::new(abort_handle),
        }
    }

    async fn fetch_client_stat(
        client_id: &ClientId,
        state: &State,
    ) -> anyhow::Result<()> {
        log::info!("Getting statistics from client: {}", client_id);

        type Vars = <StatisticsQuery as GraphQLQuery>::Variables;
        type ResponseData = <StatisticsQuery as GraphQLQuery>::ResponseData;

        let request_body = StatisticsQuery::build_query(Vars {});

        let request = reqwest::Client::new();
        let url = format!("http://{}/api", client_id);
        let res = request
            .post(url.as_str())
            .json(&request_body)
            .send()
            .await?;

        let response: Response<ResponseData> = res.json().await?;
        Self::save_client_stat(client_id, response, state)
    }

    fn save_client_error(
        client_id: &ClientId,
        error_message: String,
        state: &State,
    ) {
        let mut clients = state.clients.lock_mut();
        let client =
            match clients.iter_mut().find(|r| r.id == client_id.to_owned()) {
                Some(c) => c,
                None => panic!("Client with id = {} was not found", client_id),
            };

        client.statistics = Some(ClientStatisticsResponse {
            data: None,
            errors: Some(vec![error_message]),
        });
    }

    fn save_client_stat(
        client_id: &ClientId,
        response: Response<<StatisticsQuery as GraphQLQuery>::ResponseData>,
        state: &State,
    ) -> anyhow::Result<()> {
        let response_errors: Vec<String> = response
            .errors
            .unwrap_or(vec![])
            .into_iter()
            .map(|e| e.message)
            .collect();

        let mut clients = state.clients.lock_mut();
        let client =
            match clients.iter_mut().find(|r| r.id == client_id.to_owned()) {
                Some(c) => c,
                None => panic!("Client with id = {} was not found", client_id),
            };

        client.statistics = match response.data {
            Some(data) => Some(ClientStatisticsResponse {
                data: Some(ClientStatistics {
                    public_host: data.statistics.public_host,
                    timestamp: data.statistics.timestamp,
                    inputs: data.statistics.inputs.into(),
                    outputs: data.statistics.outputs.into(),
                }),
                errors: Some(response_errors),
            }),
            None => Some(ClientStatisticsResponse {
                data: None,
                errors: Some(response_errors),
            }),
        };

        Ok(())
    }
}
