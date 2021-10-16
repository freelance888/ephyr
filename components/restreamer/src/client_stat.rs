//! Clients statistics

use std::{
    collections::HashMap,
    panic::AssertUnwindSafe,
    time::Duration,
};

use ephyr_log::log;
use futures::{future, FutureExt as _, TryFutureExt};
use tokio::time;
use crate::state::{Client, ClientId};
use std::net::IpAddr;
use crate::display_panic;
use crate::types::DroppableAbortHandle;
use crate::api::graphql;

/// Poll of [`ClientJob`]s for getting statistics info on each [`Client`]
///
#[derive(Debug)]
pub struct ClientJobsPool {
    /// Pool of [`ClientJob`]s
    pool: HashMap<IpAddr, ClientJob>,
}

impl ClientJobsPool {
    /// Creates new pull of [`ClientJob`]
    // #[inline]
    // #[must_use]
    pub fn new() -> Self {
        Self {
            pool: HashMap::new(),
        }
    }

    /// Creates new [`ClientJob`] for added [`Client`] and removes for deleted [`Client`]
    // #[must_use]
    pub fn apply(&mut self, clients: &[Client]) {
        let mut new_pool = HashMap::with_capacity(self.pool.len() + 1);

        for c in clients {
            let client_ip = c.id.into();
            let job = self
                .pool
                .remove(&client_ip)
                .unwrap_or_else(|| {
                    ClientJob::run(c.id)
                });

            drop(new_pool.insert(client_ip, job));
        }

        self.pool = new_pool;
    }
}

/// Job for retrieving statistics from client from specific `ip`
#[derive(Debug)]
pub struct ClientJob {
    /// IP address - identity of client
    ip: IpAddr,

    /// Callback for stop job
    abort: DroppableAbortHandle,
}

impl ClientJob {
    /// Spawns new future for getting client statistics from [`Client`]
    /// [`ClientId`] is essentially IP address of remote host
    #[must_use]
    pub fn run(client_id: ClientId) -> Self {
        let (spawner, abort_handle) = future::abortable(async move {
            loop {
                let _ = AssertUnwindSafe(
                    async move {
                        println!("Get statistics from client: {}", client_id);
                        let result: Result<(), graphql::Error> = Ok(());
                        future::ready(result).await
                    }
                    .unwrap_or_else(|_| {
                        println!("ERROR. Error retrieving data for client {}. Stop gathering statistics from client", client_id);
                    }),
                )
                .catch_unwind()
                .await
                .map_err(|p| {
                    log::crit!(
                        "Panicked while gather statistics from client: {}",
                        display_panic(&p),
                    );
                });

                time::delay_for(Duration::from_secs(2)).await;
            }
        });

        // Spawn periodic job for gathering info from client
        drop(tokio::spawn(spawner.map(move |_| {
            println!("OFFLINE. Stop gathering statistics from client {}", client_id);
        })));

        Self {
            ip: client_id.into(),
            abort: DroppableAbortHandle::new(abort_handle),
        }
    }
}
