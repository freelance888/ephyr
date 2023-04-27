//! Module which collects server statistics and updates them every second
use std::time::Duration;
use systemstat::{Platform, System};
use tokio::time;

use crate::{cli::Failure, display_panic, state::ServerInfo, State};
use ephyr_log::{tracing, tracing::instrument};
use futures::FutureExt;
use num_cpus;
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
#[allow(clippy::cast_possible_wrap)]
#[instrument(skip_all, name = "statistics::run")]
pub async fn run(state: State) -> Result<(), Failure> {
    // we use tx_last and rx_last to compute the delta
    // (send/receive bytes last second)
    let mut tx_last: f64 = 0.0;
    let mut rx_last: f64 = 0.0;

    let sys = System::new();
    if let Err(e) = sys.cpu_load_aggregate().and(sys.memory()) {
        tracing::error!("Skip statistics. Failed to gather with error: {}", e);
        return Ok(());
    }

    let spawner = async move {
        loop {
            let state = &state;

            _ = AssertUnwindSafe(async {
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
                        let cpu_idle = cpu.done().map_or(0.0, |c| c.idle);

                        // in percents
                        info.update_cpu(Some(
                            f64::from(1.0 - cpu_idle) * 100.0,
                        ));

                        let cpus_usize = num_cpus::get();
                        let cpus: i32 = cpus_usize as i32;

                        info.update_cores(Some(cpus));
                    }
                    Err(x) => {
                        info.set_error(Some(x.to_string()));
                        tracing::error!("Statistics. CPU load: error: {}", x);
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
                        tracing::error!("Statistics. Memory: error: {}", x);
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
                            let (tx_bytes, rx_bytes) = sys
                                .network_stats(&netif.name)
                                .map_or((0, 0), |stat| {
                                    (
                                        stat.tx_bytes.as_u64(),
                                        stat.rx_bytes.as_u64(),
                                    )
                                });
                            // in megabytes
                            tx += tx_bytes as f64 / 1024.0 / 1024.0;
                            // in megabytes
                            rx += rx_bytes as f64 / 1024.0 / 1024.0;
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
                        tracing::error!("Statistics. Networks: error: {}", x);
                    }
                }

                *state.server_info.lock_mut() = info;

                // Try to syncronize stream info
                state.sync_stream_info();
            })
            .catch_unwind()
            .await
            .map_err(|p| {
                tracing::error!(
                    "Panicked while getting server statistics {}",
                    display_panic(&p),
                );
            });
        }
    };

    drop(tokio::spawn(spawner));

    Ok(())
}
