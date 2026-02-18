use bollard::container::StatsOptions;
use bollard::Docker;
use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

use crate::types::PodStatsUpdate;

pub async fn stream_stats(
    docker: Docker,
    container_id: String,
    pod_id: String,
    app: AppHandle,
    cancel: CancellationToken,
) {
    let options = StatsOptions {
        stream: true,
        one_shot: false,
    };

    let mut stream = docker.stats(&container_id, Some(options));

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::debug!("Stats stream cancelled for pod {}", pod_id);
                break;
            }
            item = stream.next() => {
                match item {
                    Some(Ok(stats)) => {
                        let cpu_percent = calculate_cpu_percent(&stats);
                        let memory_used = stats.memory_stats.usage.unwrap_or(0);
                        let memory_limit = stats.memory_stats.limit.unwrap_or(0);

                        let update = PodStatsUpdate {
                            pod_id: pod_id.clone(),
                            cpu_percent,
                            memory_used,
                            memory_limit,
                        };

                        if let Err(e) = app.emit("pod-stats-update", &update) {
                            tracing::error!("Failed to emit stats update: {}", e);
                        }
                    }
                    Some(Err(e)) => {
                        tracing::error!("Stats stream error for pod {}: {}", pod_id, e);
                        break;
                    }
                    None => {
                        tracing::debug!("Stats stream ended for pod {}", pod_id);
                        break;
                    }
                }
            }
        }
    }
}

fn calculate_cpu_percent(stats: &bollard::container::Stats) -> f64 {
    let cpu_stats = &stats.cpu_stats;
    let precpu_stats = &stats.precpu_stats;

    let cpu_delta = cpu_stats.cpu_usage.total_usage as f64
        - precpu_stats.cpu_usage.total_usage as f64;

    let system_delta = cpu_stats.system_cpu_usage.unwrap_or(0) as f64
        - precpu_stats.system_cpu_usage.unwrap_or(0) as f64;

    if system_delta > 0.0 && cpu_delta >= 0.0 {
        let num_cpus = cpu_stats
            .online_cpus
            .unwrap_or_else(|| cpu_stats.cpu_usage.percpu_usage.as_ref().map_or(1, |v| v.len() as u64));
        (cpu_delta / system_delta) * num_cpus as f64 * 100.0
    } else {
        0.0
    }
}
