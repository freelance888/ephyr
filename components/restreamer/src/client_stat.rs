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
use std::net::{IpAddr, Ipv4Addr};
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
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            pool: HashMap::new(),
        }
    }

    /// Creates new [`ClientJob`] for added [`Client`] and removes for deleted [`Client`]
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
                        log::info!("Get statistics from client: {}", client_id);

                        let client_ip: IpAddr = client_id.into();
                        let ip0: IpAddr = Ipv4Addr::new(0, 0, 0, 0).into();
                        let ip100: IpAddr = Ipv4Addr::new(0, 0, 0, 100).into();

                        if client_ip == ip0 { panic!("Can't get data from {}", ip0) };

                        let result: Result<(), graphql::Error> = if client_ip == ip100 {
                            let msg = format!("Error while getting data from {}", ip100);
                            Err(graphql::Error::new("ERROR_UNKNOWN").message(&msg))
                        } else {
                            Ok(())
                        };

                        future::ready(result).await
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
            log::info!("Client {} removed. Stop getting statistics", client_id);
        })));

        Self {
            ip: client_id.into(),
            abort: DroppableAbortHandle::new(abort_handle),
        }
    }
}
