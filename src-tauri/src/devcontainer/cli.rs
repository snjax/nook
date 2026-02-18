use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::error::{NookError, NookResult};
use crate::state::AppState;
use crate::types::{LogBatchEvent, LogEntry, LogLevel, LogSource};

/// Run `devcontainer up` for the given workspace folder
pub async fn devcontainer_up(workspace_path: &str) -> NookResult<String> {
    let timeout = Duration::from_secs(120);

    let result = tokio::time::timeout(timeout, async {
        let output = Command::new("devcontainer")
            .args(["up", "--workspace-folder", workspace_path])
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    NookError::DevcontainerCliNotFound
                } else {
                    NookError::Io(e)
                }
            })?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            // devcontainer up outputs JSON with containerId
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let last_lines: Vec<&str> = stderr.lines().rev().take(3).collect();
            let error_msg = last_lines.into_iter().rev().collect::<Vec<&str>>().join("\n");
            Err(NookError::DevcontainerUpFailed(format!(
                "Exit code: {:?}. {}",
                output.status.code(),
                error_msg
            )))
        }
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => Err(NookError::Timeout(
            "Container did not start within 120s".to_string(),
        )),
    }
}

/// Run `devcontainer up --remove-existing-container` for rebuild
pub async fn devcontainer_up_rebuild(workspace_path: &str) -> NookResult<String> {
    let timeout = Duration::from_secs(180);

    let result = tokio::time::timeout(timeout, async {
        let output = Command::new("devcontainer")
            .args([
                "up",
                "--workspace-folder",
                workspace_path,
                "--remove-existing-container",
            ])
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    NookError::DevcontainerCliNotFound
                } else {
                    NookError::Io(e)
                }
            })?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let last_lines: Vec<&str> = stderr.lines().rev().take(3).collect();
            let error_msg = last_lines.into_iter().rev().collect::<Vec<&str>>().join("\n");
            Err(NookError::DevcontainerUpFailed(format!(
                "Exit code: {:?}. {}",
                output.status.code(),
                error_msg
            )))
        }
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => Err(NookError::Timeout(
            "Container rebuild did not complete within 180s".to_string(),
        )),
    }
}

/// Run `devcontainer up` with streaming build output.
/// Each line of stdout/stderr is emitted as a LogEntry with source=Build.
/// Returns the final JSON output (last line with containerId) on success.
/// On cancellation, kills the child process.
pub async fn devcontainer_up_streaming(
    workspace_path: &str,
    pod_id: &str,
    app: &AppHandle,
    state: &AppState,
    cancel: CancellationToken,
) -> NookResult<String> {
    let timeout = Duration::from_secs(180);

    let result = tokio::time::timeout(timeout, async {
        let mut child = Command::new("devcontainer")
            .args(["up", "--workspace-folder", workspace_path])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    NookError::DevcontainerCliNotFound
                } else {
                    NookError::Io(e)
                }
            })?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let pod_id_owned = pod_id.to_string();
        let app_clone = app.clone();
        let state_clone = state.clone();

        // Stream stdout in background
        let stdout_pod = pod_id_owned.clone();
        let stdout_app = app_clone.clone();
        let stdout_state = state_clone.clone();
        let stdout_handle = stdout.map(|stdout| {
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                let mut last_line = String::new();
                while let Ok(Some(line)) = lines.next_line().await {
                    last_line = line.clone();
                    emit_build_line(&stdout_app, &stdout_state, &stdout_pod, &line, LogLevel::Stdout).await;
                }
                last_line
            })
        });

        // Stream stderr in background
        let stderr_pod = pod_id_owned.clone();
        let stderr_app = app_clone.clone();
        let stderr_state = state_clone.clone();
        let stderr_handle = stderr.map(|stderr| {
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                let mut all_stderr = String::new();
                while let Ok(Some(line)) = lines.next_line().await {
                    if !all_stderr.is_empty() {
                        all_stderr.push('\n');
                    }
                    all_stderr.push_str(&line);
                    emit_build_line(&stderr_app, &stderr_state, &stderr_pod, &line, LogLevel::Stderr).await;
                }
                all_stderr
            })
        });

        // Wait for process or cancellation
        tokio::select! {
            _ = cancel.cancelled() => {
                let _ = child.kill().await;
                Err(NookError::Cancelled("Build cancelled".to_string()))
            }
            status = child.wait() => {
                let status = status.map_err(NookError::Io)?;
                let last_stdout_line = if let Some(h) = stdout_handle {
                    h.await.unwrap_or_default()
                } else {
                    String::new()
                };
                let stderr_output = if let Some(h) = stderr_handle {
                    h.await.unwrap_or_default()
                } else {
                    String::new()
                };

                if status.success() {
                    // The last stdout line should contain the JSON with containerId
                    Ok(last_stdout_line)
                } else {
                    let last_lines: Vec<&str> = stderr_output.lines().rev().take(3).collect();
                    let error_msg = last_lines.into_iter().rev().collect::<Vec<&str>>().join("\n");
                    Err(NookError::DevcontainerUpFailed(format!(
                        "Exit code: {:?}. {}",
                        status.code(),
                        error_msg
                    )))
                }
            }
        }
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => Err(NookError::Timeout(
            "Container build did not complete within 180s".to_string(),
        )),
    }
}

async fn emit_build_line(
    app: &AppHandle,
    state: &AppState,
    pod_id: &str,
    line: &str,
    level: LogLevel,
) {
    let entry = LogEntry {
        timestamp: chrono::Utc::now().timestamp(),
        message: line.to_string(),
        source: LogSource::Build,
        level,
    };

    // Store in LogBuffer
    {
        let mut state = state.lock().await;
        let buf = state
            .log_buffers
            .entry(pod_id.to_string())
            .or_default();
        buf.push(entry.clone());
    }

    // Emit to frontend
    let event = LogBatchEvent {
        pod_id: pod_id.to_string(),
        entries: vec![entry],
    };
    let _ = app.emit("pod-log-update", &event);
}

/// Run a command inside the devcontainer
pub async fn devcontainer_exec(
    workspace_path: &str,
    command: &[&str],
) -> NookResult<String> {
    let mut args = vec!["exec", "--workspace-folder", workspace_path, "--"];
    args.extend_from_slice(command);

    let output = Command::new("devcontainer")
        .args(&args)
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                NookError::DevcontainerCliNotFound
            } else {
                NookError::Io(e)
            }
        })?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Check if devcontainer CLI is available
pub async fn is_devcontainer_available() -> bool {
    Command::new("devcontainer")
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}
