//! Module which collects server statistics and updates them every second
use std::time::Duration;
use systemstat::{Platform, System};
use tokio::time;

use crate::state::{EndpointId, InputKey, InputSrc, RestreamKey, Status};
use crate::stream_probe::stream_probe;
use crate::{cli::Failure, display_panic, state::ServerInfo, State};
use anyhow::anyhow;
use ephyr_log::log;
use ephyr_log::slog::log;
use futures::{FutureExt, TryFutureExt};
use std::panic::AssertUnwindSafe;
use url::Url;

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

                // update_streams_info(state.clone()).await;
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

fn rtmp_url(restream_key: &RestreamKey, input_key: &InputKey) -> Url {
    Url::parse(&format!(
        "rtmp://127.0.0.1:1935/{}/{}",
        restream_key, input_key,
    ))
    .unwrap()
}

async fn update_streams_info(state: State) {
    let mut restreams = state.restreams.lock_mut();
    restreams.iter_mut().for_each(|r| {
        if let Some(InputSrc::Failover(s)) = &mut r.input.src {
            for mut i in s.inputs.iter_mut() {
                for mut e in i.endpoints.iter_mut() {
                    log::debug!("ENDPOINT: {:?}", e);
                    if e.status == Status::Online && e.stream_stat.is_none() {
                        let url = rtmp_url(&r.key, &i.key);
                        if !url.to_string().contains("playback") {
                            e.stream_stat = None;
                            set_stream_info(e.id, url, state.clone());
                        }
                    }
                }
            }
        }
    });
}

fn set_stream_info(id: EndpointId, url: Url, state: State) {
    drop(tokio::spawn(
        AssertUnwindSafe(async move {
            time::sleep(Duration::from_secs(10)).await;

            stream_probe(url).await.map_or_else(
                |e| log::error!("FFPROBE ERROR: {}", e),
                |info| {
                    state.set_stream_info(id, info).unwrap_or_else(|e| {
                        log::error!("SET STREAM INFO ERROR: {}", e)
                    })
                },
            );
        })
        .catch_unwind()
        .map_err(move |p| {
            log::crit!("Can not fetch stream info: {}", display_panic(&p),);
        }),
    ));
}
