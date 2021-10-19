//! Clients statistics

// This is required because of graphql_client generate
// module for query without documentation and causes warning messages
#![allow(missing_docs)]

use std::{
    collections::HashMap,
    panic::AssertUnwindSafe,
    time::Duration,
};

use ephyr_log::log;
use futures::{future, FutureExt as _, TryFutureExt};
use tokio::time;
use crate::state::{Client, ClientId, ClientStatisticsResponse, ClientStatistics};
use crate::display_panic;
use crate::types::DroppableAbortHandle;

use graphql_client::{GraphQLQuery, Response};
use reqwest;
use chrono::{DateTime, Utc};

/// Poll of [`ClientJob`]s for getting statistics info from each [`Client`]
///
#[derive(Debug)]
pub struct ClientJobsPool {
    /// Pool of [`ClientJob`]s
    pool: HashMap<String, ClientJob>,
}

impl ClientJobsPool {
    /// Creates new pull of [`ClientJob`]
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            pool: HashMap::new(),
        }
    }

    /// Creates new [`ClientJob`] for added [`Client`] and removes for deleted [`Client`]
    pub fn apply(&mut self, clients: &[Client]) {
        let mut new_pool= HashMap::with_capacity(self.pool.len() + 1);

        for c in clients {
            let client_id = c.id.clone().into();
            let job = self
                .pool
                .remove(&client_id)
                .unwrap_or_else(|| {
                    ClientJob::run(c.id.clone())
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
    response_derives = "Debug",
)]
#[derive(Debug)]
pub struct StatisticsQuery;

/// Job for retrieving statistics from client from specific `ip`
#[derive(Debug)]
pub struct ClientJob {
    /// IP address - identity of client
    id: ClientId,

    /// Callback for stop job
    abort: DroppableAbortHandle,
}

impl ClientJob {
    /// Spawns new future for getting client statistics from [`Client`]
    #[must_use]
    pub fn run(id: ClientId) -> Self {
        let client_id1 = id.clone();
        let client_id2 = id.clone();

        let (spawner, abort_handle) = future::abortable(async move {
            loop {
                let client_id = &id;
                let _ = AssertUnwindSafe(
                    async move {
                        let result = Self::fetch_client_stat(client_id).await;
                        println!("DATA from {}: {:#?}", client_id, result);
                        match result {
                            Ok(_) => Ok(()),
                            Err(e) => Err(e)
                        }
                    }
                    .unwrap_or_else(|e| {
                        log::info!("Error retrieving data for client {}. {}", client_id, e);
                    }),
                )
                .catch_unwind()
                .await
                .map_err(|p| {
                    log::error!(
                        "Panicked while getting statistics from client: {}",
                        display_panic(&p)
                    );
                });

                time::delay_for(Duration::from_secs(2)).await;
            }
        });

        // Spawn periodic job for gathering info from client
        drop(tokio::spawn(spawner.map(move |_| {
            log::info!("Client {} removed. Stop getting statistics", client_id1);
        })));

        Self {
            id: client_id2,
            abort: DroppableAbortHandle::new(abort_handle),
        }
    }

    async fn fetch_client_stat(client_id: &ClientId) -> anyhow::Result<ClientStatisticsResponse> {
        log::info!("Get statistics from client: {}", client_id);

        type Vars = <StatisticsQuery as GraphQLQuery>::Variables;
        type ResponseData = <StatisticsQuery as GraphQLQuery>::ResponseData;

        let request_body = StatisticsQuery::build_query(Vars {});

        let client = reqwest::Client::new();
        let url = format!("http://{}/api", client_id);
        let res = client.post(url.as_str()).json(&request_body).send().await?;

        let response: Response<ResponseData> = res.json().await?;
        let errors= response.errors.unwrap_or(vec![]).into_iter().map(|e| e.message).collect();

        match response.data {
            Some(data) => Ok(ClientStatisticsResponse {
                data: Some(ClientStatistics {
                    public_host: data.statistics.public_host,
                    timestamp: data.statistics.timestamp,
                }),
                errors: Some(errors)
            }),
            None => Ok(ClientStatisticsResponse {
                data: None,
                errors: Some(errors)
            })
        }
    }
}
