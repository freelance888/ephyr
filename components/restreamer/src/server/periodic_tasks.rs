//! Module for running periodic tasks
use std::future::Future;
use systemstat::{Platform, System};
use tokio::time;

use crate::{
    cli::Failure,
    display_panic,
    file_manager::{FileCommand, FileState},
    state::{InputEndpointKind, InputSrc, ServerInfo, Status},
    types::UNumber,
    State,
};
use anyhow::anyhow;
use ephyr_log::{tracing, tracing::instrument};
use futures::FutureExt;
use num_cpus;
use std::panic::AssertUnwindSafe;

fn run_periodic<F, Fut>(state: State, interval: time::Duration, mut func: F)
where
    F: FnMut(State) -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), anyhow::Error>> + Send,
{
    let spawner = async move {
        let mut interval = time::interval(interval);
        loop {
            _ = interval.tick().await;

            let result = AssertUnwindSafe(async { func(state.clone()).await })
                .catch_unwind()
                .await;

            match result {
                Ok(r) => match r {
                    Ok(()) => {}
                    Err(err) => {
                        tracing::error!(
                            "An error: `{err}` occurred while running \
                            periodic task. Stopping task."
                        );
                        break;
                    }
                },
                Err(p) => {
                    tracing::error!(
                        "Panicked while running periodic task: {}",
                        display_panic(&p),
                    );
                    break;
                }
            }
        }
    };

    drop(tokio::spawn(spawner));
}

/// Runs periodic tasks
///
/// # Panics
/// Panic is captured to log. Could be panicked during getting server
/// statistics.
///
/// # Errors
/// No return errors expected. Preserved return signature in order to
/// run in `future::try_join3`
#[instrument(skip_all, name = "periodic_task::run")]
pub async fn run(state: State) -> Result<(), Failure> {
    run_periodic(
        state.clone(),
        time::Duration::from_secs(10),
        |state| async move {
            // we use tx_last and rx_last to compute the delta
            // (send/receive bytes last second)
            let mut tx_last: f64 = 0.0;
            let mut rx_last: f64 = 0.0;
            update_server_statistics(state, &mut tx_last, &mut rx_last).await
        },
    );

    run_periodic(
        state.clone(),
        time::Duration::from_secs(2),
        |state| async move { sync_stream_info(state) },
    );

    run_periodic(
        state.clone(),
        time::Duration::from_secs(2),
        |state| async move { start_pending_downloads(state) },
    );

    Ok(())
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::unnecessary_wraps)]
#[allow(clippy::needless_pass_by_value)]
async fn update_server_statistics(
    state: State,
    tx_last: &mut f64,
    rx_last: &mut f64,
) -> Result<(), anyhow::Error> {
    let sys = System::new();
    if let Err(e) = sys.cpu_load_aggregate().and(sys.memory()) {
        return Err(anyhow!(
            "Skip statistics. Failed to gather with error: {e}"
        ));
    }
    let mut info = ServerInfo::default();

    // Update cpu usage
    match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            // Need to wait some time to let the library compute
            // CPU usage.
            // Do not change delay time, since it is also used
            // further to compute network statistics
            // (bytes sent/received last second)
            time::sleep(time::Duration::from_secs(1)).await;
            let cpu_idle = cpu.done().map_or(0.0, |c| c.idle);

            // in percents
            info.update_cpu(Some(f64::from(1.0 - cpu_idle) * 100.0));

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
            info.update_ram(Some(mem_total as f64), Some(mem_free as f64));
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
                let (tx_bytes, rx_bytes) =
                    sys.network_stats(&netif.name).map_or((0, 0), |stat| {
                        (stat.tx_bytes.as_u64(), stat.rx_bytes.as_u64())
                    });
                // in megabytes
                tx += tx_bytes as f64 / 1024.0 / 1024.0;
                // in megabytes
                rx += rx_bytes as f64 / 1024.0 / 1024.0;
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
            tracing::error!("Statistics. Networks: error: {}", x);
        }
    }

    *state.server_info.lock_mut() = info;
    Ok(())
}

/// Synchronize stream statistics
#[allow(clippy::unnecessary_wraps)]
#[allow(clippy::needless_pass_by_value)]
fn sync_stream_info(state: State) -> Result<(), anyhow::Error> {
    let files = state.files.lock_mut();
    let mut restreams = state.restreams.lock_mut();
    restreams.iter_mut().for_each(|r| {
        if let Some(InputSrc::Failover(s)) = &mut r.input.src {
            for e in s.inputs.iter_mut().flat_map(|i| i.endpoints.iter_mut()) {
                if e.kind == InputEndpointKind::File && e.file_id.is_some() {
                    // For file - populate statistics from [`LocalFileInfo`]
                    if let Some(file_id) = e.file_id.clone() {
                        _ = files.iter().find_map(|f| {
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
    Ok(())
}

/// Controls the number of simultaneous downloads in queue
#[allow(clippy::unnecessary_wraps)]
#[allow(clippy::needless_pass_by_value)]
fn start_pending_downloads(state: State) -> Result<(), anyhow::Error> {
    let mut files = state.files.lock_mut();
    let files_in_queue_count = files
        .iter()
        .filter(|f| {
            [FileState::Pending, FileState::Downloading].contains(&f.state)
        })
        .count();

    let allowed_to_add = state
        .settings
        .get_cloned()
        .max_downloading_files
        .unwrap_or(UNumber(3))
        .0 as usize
        - files_in_queue_count;

    if allowed_to_add > 0 {
        let file_ids = files
            .iter_mut()
            .filter(|f| f.state == FileState::Waiting)
            .take(allowed_to_add)
            .map(|f| {
                f.stream_stat = None;
                f.state = FileState::Pending;
                f.error = None;
                f.file_id.clone()
            })
            .collect();

        state
            .file_commands
            .lock_mut()
            .push(FileCommand::StartDownloadFile(file_ids));
    }
    Ok(())
}
