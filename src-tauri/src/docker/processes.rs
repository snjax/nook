use bollard::container::TopOptions;
use bollard::Docker;
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

use crate::types::{Process, ProcessListUpdate};

pub async fn poll_processes(
    docker: Docker,
    container_id: String,
    pod_id: String,
    app: AppHandle,
    cancel: CancellationToken,
    interval_ms: u64,
) {
    let interval = tokio::time::Duration::from_millis(interval_ms);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::debug!("Process polling cancelled for pod {}", pod_id);
                break;
            }
            _ = tokio::time::sleep(interval) => {
                if cancel.is_cancelled() {
                    tracing::debug!("Process polling cancelled (post-sleep) for pod {}", pod_id);
                    break;
                }
                match get_processes(&docker, &container_id).await {
                    Ok(processes) => {
                        let update = ProcessListUpdate {
                            pod_id: pod_id.clone(),
                            processes,
                        };
                        if let Err(e) = app.emit("process-list-update", &update) {
                            tracing::error!("Failed to emit process list: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to get processes for pod {}: {}", pod_id, e);
                    }
                }
            }
        }
    }
}

async fn get_processes(
    docker: &Docker,
    container_id: &str,
) -> Result<Vec<Process>, bollard::errors::Error> {
    let options = TopOptions { ps_args: "aux" };
    let top = docker.top_processes(container_id, Some(options)).await?;

    let titles = top.titles.unwrap_or_default();
    let processes_raw = top.processes.unwrap_or_default();

    let pid_idx = titles.iter().position(|t| t == "PID").unwrap_or(1);
    let cmd_idx = titles.iter().position(|t| t == "COMMAND").unwrap_or(titles.len().saturating_sub(1));
    let cpu_idx = titles.iter().position(|t| t == "%CPU").unwrap_or(2);
    let mem_idx = titles.iter().position(|t| t == "RSS" || t == "VSZ").unwrap_or(5);

    let mut processes = Vec::new();
    for row in processes_raw {
        let pid = row.get(pid_idx).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let name = row.get(cmd_idx).cloned().unwrap_or_default();
        let name = name.split_whitespace().next().unwrap_or(&name).to_string();
        let cpu_percent = row
            .get(cpu_idx)
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let memory_bytes = row
            .get(mem_idx)
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0)
            * 1024; // RSS is in KB

        processes.push(Process {
            pid,
            name,
            cpu_percent,
            memory_bytes,
        });
    }

    Ok(processes)
}
