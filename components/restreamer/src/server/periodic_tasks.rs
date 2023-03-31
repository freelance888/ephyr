//! Module for running periodic tasks
use std::time::Duration;
use systemstat::{Platform, System};
use tokio::time;

use crate::{
    cli::Failure,
    display_panic,
    file_manager::{FileCommand, FileState},
    state::{InputEndpointKind, InputSrc, ServerInfo, Status},
    State,
};
use ephyr_log::log;
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
pub async fn run(state: State) -> Result<(), Failure> {
    // we use tx_last and rx_last to compute the delta
    // (send/receive bytes last second)
    let mut tx_last: f64 = 0.0;
    let mut rx_last: f64 = 0.0;

    let spawner = async move {
        loop {
            let state = &state;

            let _ = AssertUnwindSafe(async {
                // Update server's statistics
                update_server_statistics(state, &mut tx_last, &mut rx_last)
                    .await;

                // Try to synchronize stream info
                sync_stream_info(state);

                start_pending_downloads(state);
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

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_precision_loss)]
async fn update_server_statistics(
    state: &State,
    tx_last: &mut f64,
    rx_last: &mut f64,
) {
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
            info.update_cpu(Some(f64::from(1.0 - cpu.idle) * 100.0));

            let cpus_usize = num_cpus::get();
            let cpus: i32 = cpus_usize as i32;

            info.update_cores(Some(cpus));
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
            info.update_ram(Some(mem_total as f64), Some(mem_free as f64));
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
                let netstats = sys.network_stats(&netif.name).unwrap();
                // in megabytes
                tx += netstats.tx_bytes.as_u64() as f64 / 1024.0 / 1024.0;
                // in megabytes
                rx += netstats.rx_bytes.as_u64() as f64 / 1024.0 / 1024.0;
            }

            // Compute delta
            let tx_delta = tx - *tx_last;
            let rx_delta = rx - *rx_last;

            // Update server info
            info.update_traffic_usage(Some(tx_delta), Some(rx_delta));

            *tx_last = tx;
            *rx_last = rx;
        }
        Err(x) => {
            info.set_error(Some(x.to_string()));
            log::error!("Statistics. Networks: error: {}", x);
        }
    }

    *state.server_info.lock_mut() = info;
}

/// Synchronize stream statistics
fn sync_stream_info(state: &State) {
    let files = state.files.lock_mut();
    let mut restreams = state.restreams.lock_mut();
    restreams.iter_mut().for_each(|r| {
        if let Some(InputSrc::Failover(s)) = &mut r.input.src {
            for mut e in
                s.inputs.iter_mut().flat_map(|i| i.endpoints.iter_mut())
            {
                if e.kind == InputEndpointKind::File && e.file_id.is_some() {
                    // For file - populate statistics from [`LocalFileInfo`]
                    if let Some(file_id) = e.file_id.clone() {
                        let _ = files.iter().find_map(|f| {
                            (f.file_id == file_id).then(|| {
                                e.stream_stat = f.stream_stat.clone();
                            })
                        });
                    }
                } else if e.stream_stat.is_some() && e.status == Status::Offline
                {
                    // For stream - clear statistics if stream is offline
                    e.stream_stat = None;
                }
            }
        }
    });
}

/// Controls the number of simultaneous downloads in queue
fn start_pending_downloads(state: &State) {
    let files = state.files.lock_mut();
    let files_in_queue_count = files
        .iter()
        .filter(|f| {
            [FileState::Pending, FileState::Downloading].contains(&f.state)
        })
        .count();

    let allowed_to_add = 3 - files_in_queue_count;
    if allowed_to_add > 0 {
        let mut commands = state.file_commands.lock_mut();
        files
            .iter()
            .filter(|f| f.state == FileState::Waiting)
            .take(allowed_to_add)
            .for_each(|f| {
                commands
                    .push(FileCommand::StartDownloadFile(f.file_id.clone()));
            });
    }
}
