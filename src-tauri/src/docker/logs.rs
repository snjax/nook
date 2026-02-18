use bollard::container::LogsOptions;
use bollard::Docker;
use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

use crate::state::AppState;
use crate::types::{LogBatchEvent, LogEntry, LogLevel, LogSource};

/// Stream container logs in real-time, batching into events.
/// Follows the same cancellation pattern as `stream_stats`.
pub async fn stream_logs(
    docker: Docker,
    container_id: String,
    pod_id: String,
    app: AppHandle,
    state: AppState,
    cancel: CancellationToken,
    tail_lines: u64,
) {
    let options = LogsOptions::<String> {
        follow: true,
        stdout: true,
        stderr: true,
        tail: tail_lines.to_string(),
        ..Default::default()
    };

    let mut stream = docker.logs(&container_id, Some(options));
    let mut batch: Vec<LogEntry> = Vec::new();
    let mut last_flush = tokio::time::Instant::now();
    let flush_interval = tokio::time::Duration::from_millis(200);
    let max_batch = 50;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::debug!("Log stream cancelled for pod {}", pod_id);
                // Flush remaining
                if !batch.is_empty() {
                    flush_batch(&app, &state, &pod_id, &mut batch).await;
                }
                break;
            }
            _ = tokio::time::sleep(flush_interval) => {
                if !batch.is_empty() {
                    flush_batch(&app, &state, &pod_id, &mut batch).await;
                    last_flush = tokio::time::Instant::now();
                }
            }
            item = stream.next() => {
                match item {
                    Some(Ok(output)) => {
                        let (level, message) = match output {
                            bollard::container::LogOutput::StdOut { message } => {
                                (LogLevel::Stdout, String::from_utf8_lossy(&message).to_string())
                            }
                            bollard::container::LogOutput::StdErr { message } => {
                                (LogLevel::Stderr, String::from_utf8_lossy(&message).to_string())
                            }
                            _ => continue,
                        };

                        // Split multi-line output into individual entries
                        for line in message.lines() {
                            batch.push(LogEntry {
                                timestamp: chrono::Utc::now().timestamp(),
                                message: line.to_string(),
                                source: LogSource::Container,
                                level: level.clone(),
                            });
                        }

                        // Flush if batch is full or interval elapsed
                        if batch.len() >= max_batch || last_flush.elapsed() >= flush_interval {
                            flush_batch(&app, &state, &pod_id, &mut batch).await;
                            last_flush = tokio::time::Instant::now();
                        }
                    }
                    Some(Err(e)) => {
                        tracing::error!("Log stream error for pod {}: {}", pod_id, e);
                        break;
                    }
                    None => {
                        tracing::debug!("Log stream ended for pod {}", pod_id);
                        break;
                    }
                }
            }
        }
    }
}

async fn flush_batch(
    app: &AppHandle,
    state: &AppState,
    pod_id: &str,
    batch: &mut Vec<LogEntry>,
) {
    let entries: Vec<LogEntry> = std::mem::take(batch);

    // Store in LogBuffer
    {
        let mut state = state.lock().await;
        let buf = state
            .log_buffers
            .entry(pod_id.to_string())
            .or_default();
        buf.push_batch(entries.clone());
    }

    // Emit to frontend
    let event = LogBatchEvent {
        pod_id: pod_id.to_string(),
        entries,
    };
    if let Err(e) = app.emit("pod-log-update", &event) {
        tracing::error!("Failed to emit log batch: {}", e);
    }
}
