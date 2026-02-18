use std::collections::{HashMap, HashSet};

use bollard::exec::{CreateExecOptions, StartExecOptions};
use bollard::Docker;
use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

use crate::network::heuristics;
use crate::network::polling;
use crate::types::{Confidence, DetectedPort, DetectionMethod, PortDetectedEvent};

/// Periodically scan for listening ports inside a container and emit
/// `port-detected` events for newly discovered ports.
pub async fn poll_ports(
    docker: Docker,
    container_id: String,
    pod_id: String,
    app: AppHandle,
    cancel: CancellationToken,
    interval_ms: u64,
    port_overrides: HashMap<String, String>,
) {
    let interval = tokio::time::Duration::from_millis(interval_ms);
    let mut known_ports: HashSet<u16> = HashSet::new();

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::debug!("Port scanning cancelled for pod {}", pod_id);
                break;
            }
            _ = tokio::time::sleep(interval) => {
                if cancel.is_cancelled() {
                    break;
                }

                let ports = scan_container_ports(&docker, &container_id).await;

                for lp in ports {
                    if known_ports.contains(&lp.port) {
                        continue;
                    }
                    known_ports.insert(lp.port);

                    let process_name = lp.process_name.clone().unwrap_or_default();

                    // Determine protocol via heuristics
                    let protocol = heuristics::protocol_for_port(lp.port, &port_overrides)
                        .or_else(|| heuristics::protocol_for_process(&process_name))
                        .unwrap_or_else(|| "unknown".to_string());

                    let (method, confidence) = if heuristics::protocol_for_port(lp.port, &port_overrides).is_some() {
                        (DetectionMethod::WellKnown, Confidence::High)
                    } else if heuristics::protocol_for_process(&process_name).is_some() {
                        (DetectionMethod::ProcessName, Confidence::High)
                    } else {
                        (DetectionMethod::Unknown, Confidence::Low)
                    };

                    let detected = DetectedPort {
                        container_port: lp.port,
                        protocol,
                        process_name,
                        detection_method: method,
                        confidence,
                    };

                    let event = PortDetectedEvent {
                        pod_id: pod_id.clone(),
                        port: detected,
                    };

                    if let Err(e) = app.emit("port-detected", &event) {
                        tracing::error!("Failed to emit port-detected: {}", e);
                    }
                }
            }
        }
    }
}

/// Run `ss -tlnp` inside the container and parse the output.
/// Falls back to reading `/proc/net/tcp` if ss is not available.
async fn scan_container_ports(
    docker: &Docker,
    container_id: &str,
) -> Vec<polling::ListeningPort> {
    // Try ss first
    if let Some(output) = docker_exec(docker, container_id, &["ss", "-tlnp"]).await {
        let ports = polling::parse_ss_output(&output);
        if !ports.is_empty() {
            return ports;
        }
    }

    // Fall back to /proc/net/tcp
    if let Some(output) = docker_exec(docker, container_id, &["cat", "/proc/net/tcp"]).await {
        return polling::parse_proc_net_tcp(&output);
    }

    Vec::new()
}

async fn docker_exec(docker: &Docker, container_id: &str, cmd: &[&str]) -> Option<String> {
    let exec = docker
        .create_exec(
            container_id,
            CreateExecOptions::<&str> {
                cmd: Some(cmd.to_vec()),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            },
        )
        .await
        .ok()?;

    let output = docker
        .start_exec(
            &exec.id,
            Some(StartExecOptions {
                detach: false,
                ..Default::default()
            }),
        )
        .await
        .ok()?;

    let mut result = String::new();
    if let bollard::exec::StartExecResults::Attached { mut output, .. } = output {
        while let Some(Ok(chunk)) = output.next().await {
            result.push_str(&chunk.to_string());
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}
