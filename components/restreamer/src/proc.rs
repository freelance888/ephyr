use anyhow::anyhow;
use ephyr_log::tracing;
use tokio::process::Command;

/// Send SIGTERM signal to provided process name.
pub(crate) async fn kill_process_by_name(
    process_name: &str,
) -> Result<(), anyhow::Error> {
    // Find the PIDs of the running processes with process_name using `pgrep`
    let output = Command::new("pgrep")
        .arg(process_name)
        .output()
        .await
        .map_err(|e| anyhow!("Failed to execute pgrep: {e}"))?;

    if !output.status.success() {
        // No running process with process_name
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pids: Vec<i32> = stdout
        .lines()
        .filter_map(|line| line.parse::<i32>().ok())
        .collect();

    // Send SIGTERM to each process
    for pid in pids {
        _ = kill_process(pid).map_err(|err| {
            tracing::error!("Failed to kill SRS process: {err}");
        });
    }

    Ok(())
}

/// Send SIGTERM signal to provided process pid.
pub(crate) fn kill_process(pid: i32) -> Result<(), anyhow::Error> {
    use nix::{
        sys::signal::{kill, Signal::SIGTERM},
        unistd::Pid,
    };
    if let Err(err) = kill(Pid::from_raw(pid), SIGTERM) {
        Err(anyhow!("Failed to send SIGTERM to process {pid}: {err}"))
    } else {
        Ok(())
    }
}
