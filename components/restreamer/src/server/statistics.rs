//! Module which collects server statistics and updates them every second
use std::time::Duration;
use systemstat::{Platform, System};
use tokio::time;

use crate::{cli::Failure, display_panic, state::ServerInfo, State};
use actix_web::middleware::ErrorHandlerResponse::Response;
use ephyr_log::log;
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use std::panic::AssertUnwindSafe;

/// Runs statistics monitoring
///
/// # Panics
/// Panic is captured to log. Could be panicked during getting server
/// statistics.
///
/// # Errors
/// No return errors expected. Preserved return signature in order to
/// run in `future::try_join3`
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
pub async fn run(state: State) -> Result<(), Failure> {
    // we use tx_last and rx_last to compute the delta
    // (send/receive bytes last second)
    let mut tx_last: f64 = 0.0;
    let mut rx_last: f64 = 0.0;

    let spawner = async move {
        loop {
            let state = &state;

            let _ = AssertUnwindSafe(async {
                let sys = System::new();

                let mut info = ServerInfo::default();

                // Update cpu usage
                match sys.cpu_load_aggregate() {
                    Ok(cpu) => {
                        // Need to wait some time to let the library compute
                        // CPU usage.
                        // Do not change delay time, since it is also used
                        // further to compute network statistics
                        // (bytes sent/received last second)
                        time::sleep(Duration::from_secs(1)).await;
                        let cpu = cpu.done().unwrap();

                        // in percents
                        info.update_cpu(Some(
                            f64::from(1.0 - cpu.idle) * 100.0,
                        ));
                    }
                    Err(x) => {
                        info.set_error(Some(x.to_string()));
                        log::error!("Statistics. CPU load: error: {}", x);
                    }
                }

                // Update ram usage
                match sys.memory() {
                    Ok(mem) => {
                        // in megabytes
                        let mem_total = mem.total.as_u64() / 1024 / 1024;
                        // in megabytes
                        let mem_free = mem.free.as_u64() / 1024 / 1024;
                        info.update_ram(
                            Some(mem_total as f64),
                            Some(mem_free as f64),
                        );
                    }
                    Err(x) => {
                        info.set_error(Some(x.to_string()));
                        log::error!("Statistics. Memory: error: {}", x);
                    }
                }

                // Update network usage
                match sys.networks() {
                    Ok(netifs) => {
                        // Sum up along network interfaces
                        let mut tx: f64 = 0.0;
                        let mut rx: f64 = 0.0;

                        // Note that the sum of sent/received bytes are
                        // computed among all the available network
                        // interfaces
                        for netif in netifs.values() {
                            let netstats =
                                sys.network_stats(&netif.name).unwrap();
                            // in megabytes
                            tx += netstats.tx_bytes.as_u64() as f64
                                / 1024.0
                                / 1024.0;
                            // in megabytes
                            rx += netstats.rx_bytes.as_u64() as f64
                                / 1024.0
                                / 1024.0;
                        }

                        // Compute delta
                        let tx_delta = tx - tx_last;
                        let rx_delta = rx - rx_last;

                        // Update server info
                        info.update_traffic_usage(
                            Some(tx_delta),
                            Some(rx_delta),
                        );

                        tx_last = tx;
                        rx_last = rx;
                    }
                    Err(x) => {
                        info.set_error(Some(x.to_string()));
                        log::error!("Statistics. Networks: error: {}", x);
                    }
                }

                *state.server_info.lock_mut() = info;

                update_stream_info(state).await
            })
            .catch_unwind()
            .await
            .map_err(|p| {
                log::crit!(
                    "Panicked while getting server statistics {}",
                    display_panic(&p),
                );
            });
        }
    };

    drop(tokio::spawn(spawner));

    Ok(())
}

async fn update_stream_info(state: &State) {
    let response: Result<StreamsResponse, reqwest::Error> =
        reqwest::get("http://127.0.0.1:8002/api/v1/streams")
            .await?
            .json::<StreamsResponse>()
            .await?;

    match response {
        StreamsResponse(streams) => {}
        Err(error) => {}
    }

    println!("body = {:?}", body);
}
