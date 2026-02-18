use tauri::{AppHandle, Emitter, State};

use crate::config::pod_state::PodStateConfig;
use crate::devcontainer::cli;
use crate::docker::containers;
use crate::state::AppState;
use crate::terminal;
use crate::types::{
    DependencyCheck, ExposedPort, Pod, PodStatus, PodStatusChanged, ProcessListUpdate,
};

#[tauri::command]
pub async fn list_pods(state: State<'_, AppState>) -> Result<Vec<Pod>, String> {
    let state = state.lock().await;
    let pods: Vec<Pod> = state.pods.values().cloned().collect();
    Ok(pods)
}

#[tauri::command]
pub async fn add_pod(
    state: State<'_, AppState>,
    path: String,
) -> Result<Pod, String> {
    let name = std::path::Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string());

    let pod_id = uuid::Uuid::new_v4().to_string();

    let pod = Pod {
        id: pod_id.clone(),
        name: name.clone(),
        project_path: path.clone(),
        image: String::new(),
        status: PodStatus::Stopped,
        uptime_secs: None,
        cpu_percent: 0.0,
        memory_used: 0,
        memory_limit: 0,
        default_shell: String::new(),
        exposed_ports: Vec::new(),
        detected_ports: Vec::new(),
        processes: Vec::new(),
        error_message: None,
        container_id: None,
        started_at: None,
        alias: None,
        remote_user: None,
        remote_workspace_folder: None,
        container_name: None,
    };

    // Save pod state
    let config = PodStateConfig {
        name: name.clone(),
        project_path: path,
        shell: String::new(),
        auto_expose_ports: Vec::new(),
        port_mappings: std::collections::HashMap::new(),
        alias: String::new(),
        not_expose_filters: Vec::new(),
        manual_expose_ports: Vec::new(),
        terminal_override: String::new(),
        working_dir: String::new(),
        bind_address: String::new(),
        remote_user: String::new(),
    };
    config.save().map_err(|e| e.to_string())?;

    let mut state = state.lock().await;
    state.pods.insert(pod_id, pod.clone());

    Ok(pod)
}

#[tauri::command]
pub async fn start_pod(
    state: State<'_, AppState>,
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    let (project_path, pod_lock) = {
        let mut state = state.lock().await;
        {
            let pod = state
                .pods
                .get_mut(&id)
                .ok_or_else(|| format!("Pod not found: {}", id))?;

            if pod.status == PodStatus::Running || pod.status == PodStatus::Starting {
                return Err("Pod is already running or starting".to_string());
            }

            pod.status = PodStatus::Starting;
            pod.error_message = None;
        }

        let _ = app.emit(
            "pod-status-changed",
            &PodStatusChanged {
                pod_id: id.clone(),
                status: PodStatus::Starting,
                error_message: None,
            },
        );

        let project_path = state.pods.get(&id).unwrap().project_path.clone();
        let lock = state.get_or_create_pod_lock(&id);
        (project_path, lock)
    };

    let _guard = pod_lock.lock().await;

    // Create a build cancellation token
    let build_cancel = tokio_util::sync::CancellationToken::new();
    {
        let mut s = state.lock().await;
        s.build_cancel_tokens.insert(id.clone(), build_cancel.clone());
    }

    // Use streaming build to emit log lines during devcontainer up
    let inner_state: crate::state::AppState = (*state).clone();
    match cli::devcontainer_up_streaming(
        &project_path,
        &id,
        &app,
        &inner_state,
        build_cancel.clone(),
    )
    .await
    {
        Ok(output) => {
            // Remove build cancel token
            {
                let mut s = state.lock().await;
                s.build_cancel_tokens.remove(&id);
            }

            // Parse full devcontainer up output
            let dc_result = parse_devcontainer_output(&output);
            let container_id = dc_result.container_id;
            let remote_user = dc_result.remote_user;
            let remote_workspace_folder = dc_result.remote_workspace_folder;

            // Extract image name, container name, and detect shell from Docker inspect
            let (image_name, detected_shell, container_name) = if let Some(ref cid) = container_id {
                let state_guard = state.lock().await;
                if let Some(docker) = &state_guard.docker {
                    let inspect = containers::inspect_container(docker, cid).await.ok();
                    let img = inspect
                        .as_ref()
                        .and_then(|info| info.config.as_ref())
                        .and_then(|c| c.image.clone());
                    let cname = inspect
                        .as_ref()
                        .and_then(|info| info.name.clone())
                        .map(|n| n.trim_start_matches('/').to_string());
                    let shell = terminal::shell::detect_shell(
                        docker,
                        cid,
                        remote_user.as_deref(),
                    )
                    .await;
                    (img, shell, cname)
                } else {
                    (None, "/bin/sh".to_string(), None)
                }
            } else {
                (None, "/bin/sh".to_string(), None)
            };

            let mut state = state.lock().await;
            if let Some(pod) = state.pods.get_mut(&id) {
                pod.status = PodStatus::Running;
                pod.container_id = container_id;
                pod.started_at = Some(chrono::Utc::now().timestamp());
                pod.default_shell = detected_shell.clone();
                pod.remote_user = remote_user.clone();
                pod.remote_workspace_folder = remote_workspace_folder.clone();
                pod.container_name = container_name;
                if let Some(ref img) = image_name {
                    pod.image = img.clone();
                }

                // Persist detected metadata to pod config for future restarts
                let pod_name = pod.name.clone();
                if let Ok(Some(mut cfg)) = PodStateConfig::load(&pod_name) {
                    let mut changed = false;
                    if cfg.remote_user.is_empty() {
                        if let Some(ref u) = remote_user {
                            cfg.remote_user = u.clone();
                            changed = true;
                        }
                    }
                    if cfg.working_dir.is_empty() {
                        if let Some(ref w) = remote_workspace_folder {
                            cfg.working_dir = w.clone();
                            changed = true;
                        }
                    }
                    if cfg.shell.is_empty() && terminal::shell::is_real_shell(&detected_shell) {
                        cfg.shell = detected_shell.clone();
                        changed = true;
                    }
                    if changed {
                        let _ = cfg.save();
                    }
                }
            }

            let _ = app.emit(
                "pod-status-changed",
                &PodStatusChanged {
                    pod_id: id.clone(),
                    status: PodStatus::Running,
                    error_message: None,
                },
            );

            // Start background monitoring tasks
            let docker_clone = state.docker.clone();
            let cid = state.pods.get(&id).and_then(|p| p.container_id.clone());

            if let (Some(docker), Some(container_id)) = (docker_clone, cid) {
                let cancel = tokio_util::sync::CancellationToken::new();
                state.cancellation_tokens.insert(id.clone(), cancel.clone());
                let interval = state.settings.process_scan_interval;

                // Stats streaming
                let docker_c = docker.clone();
                let cid_c = container_id.clone();
                let id_c = id.clone();
                let app_c = app.clone();
                let cancel_c = cancel.clone();
                tokio::spawn(async move {
                    crate::docker::stats::stream_stats(docker_c, cid_c, id_c, app_c, cancel_c)
                        .await;
                });

                // Container log streaming
                let docker_c = docker.clone();
                let cid_c = container_id.clone();
                let id_c = id.clone();
                let app_c = app.clone();
                let cancel_c = cancel.clone();
                let state_c = inner_state.clone();
                tokio::spawn(async move {
                    crate::docker::logs::stream_logs(
                        docker_c, cid_c, id_c, app_c, state_c, cancel_c, 100,
                    )
                    .await;
                });

                // Process polling
                let docker_c = docker.clone();
                let cid_c = container_id.clone();
                let id_c = id.clone();
                let app_c = app.clone();
                let cancel_c = cancel.clone();
                tokio::spawn(async move {
                    crate::docker::processes::poll_processes(
                        docker_c, cid_c, id_c, app_c, cancel_c, interval,
                    )
                    .await;
                });

                // Port scanning
                let docker_c = docker.clone();
                let cid_c = container_id.clone();
                let id_c = id.clone();
                let app_c = app.clone();
                let cancel_c = cancel.clone();
                let port_interval = state.settings.ports_scan_interval;
                let port_overrides: std::collections::HashMap<String, String> = state
                    .settings
                    .port_protocols
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                tokio::spawn(async move {
                    crate::network::scan::poll_ports(
                        docker_c, cid_c, id_c, app_c, cancel_c, port_interval, port_overrides,
                    )
                    .await;
                });
            }
            Ok(())
        }
        Err(e) => {
            // Remove build cancel token
            {
                let mut s = state.lock().await;
                s.build_cancel_tokens.remove(&id);
            }

            let mut state = state.lock().await;
            if let Some(pod) = state.pods.get_mut(&id) {
                pod.status = PodStatus::Error;
                pod.error_message = Some(e.to_string());
                let _ = app.emit(
                    "pod-status-changed",
                    &PodStatusChanged {
                        pod_id: id.clone(),
                        status: PodStatus::Error,
                        error_message: Some(e.to_string()),
                    },
                );
            }
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn stop_pod(
    state: State<'_, AppState>,
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    let (container_id, pod_lock) = {
        let mut state = state.lock().await;
        {
            let pod = state
                .pods
                .get_mut(&id)
                .ok_or_else(|| format!("Pod not found: {}", id))?;
            pod.status = PodStatus::Stopping;
        }

        let _ = app.emit(
            "pod-status-changed",
            &PodStatusChanged {
                pod_id: id.clone(),
                status: PodStatus::Stopping,
                error_message: None,
            },
        );

        // Cancel monitoring tasks
        if let Some(cancel) = state.cancellation_tokens.remove(&id) {
            cancel.cancel();
        }

        // Cancel all port proxies for this pod
        let proxy_keys: Vec<(String, u16)> = state
            .port_proxy_tokens
            .keys()
            .filter(|(pid, _)| pid == &id)
            .cloned()
            .collect();
        for key in proxy_keys {
            if let Some(cancel) = state.port_proxy_tokens.remove(&key) {
                cancel.cancel();
            }
        }

        let container_id = state.pods.get(&id).unwrap().container_id.clone();
        let lock = state.get_or_create_pod_lock(&id);
        (container_id, lock)
    };

    let _guard = pod_lock.lock().await;

    // Stop container via Docker API (sends SIGTERM, waits grace period, then SIGKILL)
    if let Some(cid) = &container_id {
        let state_guard = state.lock().await;
        if let Some(docker) = &state_guard.docker {
            let _ = containers::stop_container(docker, cid).await;
        }
    }

    let mut state = state.lock().await;
    if let Some(pod) = state.pods.get_mut(&id) {
        pod.status = PodStatus::Stopped;
        pod.exposed_ports.clear();
        pod.detected_ports.clear();
        pod.processes.clear();
        pod.cpu_percent = 0.0;
        pod.memory_used = 0;
        pod.started_at = None;
        let _ = app.emit(
            "pod-status-changed",
            &PodStatusChanged {
                pod_id: id.clone(),
                status: PodStatus::Stopped,
                error_message: None,
            },
        );
        // Emit authoritative empty process list to prevent stale data
        let _ = app.emit(
            "process-list-update",
            &ProcessListUpdate {
                pod_id: id.clone(),
                processes: Vec::new(),
            },
        );
    }

    Ok(())
}

#[tauri::command]
pub async fn restart_pod(
    state: State<'_, AppState>,
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    // Stop first, then start
    stop_pod(state.clone(), app.clone(), id.clone()).await?;
    start_pod(state, app, id).await?;
    Ok(())
}

#[tauri::command]
pub async fn force_stop_pod(
    state: State<'_, AppState>,
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    let container_id = {
        let mut state = state.lock().await;
        let pod = state
            .pods
            .get_mut(&id)
            .ok_or_else(|| format!("Pod not found: {}", id))?;
        let cid = pod.container_id.clone();

        // Cancel monitoring tasks
        if let Some(cancel) = state.cancellation_tokens.remove(&id) {
            cancel.cancel();
        }

        // Cancel all port proxies for this pod
        let proxy_keys: Vec<(String, u16)> = state
            .port_proxy_tokens
            .keys()
            .filter(|(pid, _)| pid == &id)
            .cloned()
            .collect();
        for key in proxy_keys {
            if let Some(cancel) = state.port_proxy_tokens.remove(&key) {
                cancel.cancel();
            }
        }

        cid
    };

    // Force kill container via Docker SIGKILL
    if let Some(cid) = &container_id {
        let state_guard = state.lock().await;
        if let Some(docker) = &state_guard.docker {
            let _ = containers::kill_container(docker, cid).await;
        }
    }

    let mut state = state.lock().await;
    if let Some(pod) = state.pods.get_mut(&id) {
        pod.status = PodStatus::Stopped;
        pod.exposed_ports.clear();
        pod.detected_ports.clear();
        pod.processes.clear();
        pod.cpu_percent = 0.0;
        pod.memory_used = 0;
        pod.started_at = None;
        let _ = app.emit(
            "pod-status-changed",
            &PodStatusChanged {
                pod_id: id.clone(),
                status: PodStatus::Stopped,
                error_message: None,
            },
        );
        let _ = app.emit(
            "process-list-update",
            &ProcessListUpdate {
                pod_id: id.clone(),
                processes: Vec::new(),
            },
        );
    }

    Ok(())
}

#[tauri::command]
pub async fn rebuild_pod(
    state: State<'_, AppState>,
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    // Stop if running
    let current_status = {
        let state = state.lock().await;
        state
            .pods
            .get(&id)
            .map(|p| p.status.clone())
            .ok_or_else(|| format!("Pod not found: {}", id))?
    };

    if current_status == PodStatus::Running || current_status == PodStatus::Starting {
        stop_pod(state.clone(), app.clone(), id.clone()).await?;
    }

    // Now rebuild with --remove-existing-container
    let project_path = {
        let mut state = state.lock().await;
        {
            let pod = state
                .pods
                .get_mut(&id)
                .ok_or_else(|| format!("Pod not found: {}", id))?;
            pod.status = PodStatus::Starting;
            pod.error_message = None;
        }

        let _ = app.emit(
            "pod-status-changed",
            &PodStatusChanged {
                pod_id: id.clone(),
                status: PodStatus::Starting,
                error_message: None,
            },
        );

        state.pods.get(&id).unwrap().project_path.clone()
    };

    // Rebuild still uses the non-streaming rebuild command (--remove-existing-container)
    // but we stream its output through build logs
    let build_cancel = tokio_util::sync::CancellationToken::new();
    {
        let mut s = state.lock().await;
        s.build_cancel_tokens.insert(id.clone(), build_cancel.clone());
    }

    let inner_state: crate::state::AppState = (*state).clone();

    match cli::devcontainer_up_rebuild(&project_path).await {
        Ok(output) => {
            {
                let mut s = state.lock().await;
                s.build_cancel_tokens.remove(&id);
            }

            let dc_result = parse_devcontainer_output(&output);
            let container_id = dc_result.container_id;
            let remote_user = dc_result.remote_user;
            let remote_workspace_folder = dc_result.remote_workspace_folder;

            // Detect shell and container name inside the rebuilt container
            let (detected_shell, container_name) = if let Some(ref cid) = container_id {
                let state_guard = state.lock().await;
                if let Some(docker) = &state_guard.docker {
                    let cname = containers::inspect_container(docker, cid)
                        .await
                        .ok()
                        .and_then(|info| info.name)
                        .map(|n| n.trim_start_matches('/').to_string());
                    let shell = terminal::shell::detect_shell(
                        docker,
                        cid,
                        remote_user.as_deref(),
                    )
                    .await;
                    (shell, cname)
                } else {
                    ("/bin/sh".to_string(), None)
                }
            } else {
                ("/bin/sh".to_string(), None)
            };

            let mut state = state.lock().await;
            if let Some(pod) = state.pods.get_mut(&id) {
                pod.status = PodStatus::Running;
                pod.container_id = container_id;
                pod.started_at = Some(chrono::Utc::now().timestamp());
                pod.default_shell = detected_shell;
                pod.remote_user = remote_user;
                pod.remote_workspace_folder = remote_workspace_folder;
                pod.container_name = container_name;
            }

            let _ = app.emit(
                "pod-status-changed",
                &PodStatusChanged {
                    pod_id: id.clone(),
                    status: PodStatus::Running,
                    error_message: None,
                },
            );

            // Start background monitoring tasks
            let docker_clone = state.docker.clone();
            let cid = state.pods.get(&id).and_then(|p| p.container_id.clone());

            if let (Some(docker), Some(container_id)) = (docker_clone, cid) {
                let cancel = tokio_util::sync::CancellationToken::new();
                state.cancellation_tokens.insert(id.clone(), cancel.clone());
                let interval = state.settings.process_scan_interval;

                let docker_c = docker.clone();
                let cid_c = container_id.clone();
                let id_c = id.clone();
                let app_c = app.clone();
                let cancel_c = cancel.clone();
                tokio::spawn(async move {
                    crate::docker::stats::stream_stats(docker_c, cid_c, id_c, app_c, cancel_c)
                        .await;
                });

                // Container log streaming
                let docker_c = docker.clone();
                let cid_c = container_id.clone();
                let id_c = id.clone();
                let app_c = app.clone();
                let cancel_c = cancel.clone();
                let state_c = inner_state.clone();
                tokio::spawn(async move {
                    crate::docker::logs::stream_logs(
                        docker_c, cid_c, id_c, app_c, state_c, cancel_c, 100,
                    )
                    .await;
                });

                let docker_c = docker.clone();
                let cid_c = container_id.clone();
                let id_c = id.clone();
                let app_c = app.clone();
                let cancel_c = cancel.clone();
                tokio::spawn(async move {
                    crate::docker::processes::poll_processes(
                        docker_c, cid_c, id_c, app_c, cancel_c, interval,
                    )
                    .await;
                });

                // Port scanning
                let docker_c = docker.clone();
                let cid_c = container_id.clone();
                let id_c = id.clone();
                let app_c = app.clone();
                let cancel_c = cancel.clone();
                let port_interval = state.settings.ports_scan_interval;
                let port_overrides: std::collections::HashMap<String, String> = state
                    .settings
                    .port_protocols
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                tokio::spawn(async move {
                    crate::network::scan::poll_ports(
                        docker_c, cid_c, id_c, app_c, cancel_c, port_interval, port_overrides,
                    )
                    .await;
                });
            }
            Ok(())
        }
        Err(e) => {
            {
                let mut s = state.lock().await;
                s.build_cancel_tokens.remove(&id);
            }

            let mut state = state.lock().await;
            if let Some(pod) = state.pods.get_mut(&id) {
                pod.status = PodStatus::Error;
                pod.error_message = Some(e.to_string());
                let _ = app.emit(
                    "pod-status-changed",
                    &PodStatusChanged {
                        pod_id: id.clone(),
                        status: PodStatus::Error,
                        error_message: Some(e.to_string()),
                    },
                );
            }
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn remove_pod(
    state: State<'_, AppState>,
    id: String,
    remove_volumes: bool,
) -> Result<(), String> {
    let (container_id, pod_name, pod_lock) = {
        let mut state = state.lock().await;
        if !state.pods.contains_key(&id) {
            return Err(format!("Pod not found: {}", id));
        }

        // Cancel any running tasks
        if let Some(cancel) = state.cancellation_tokens.remove(&id) {
            cancel.cancel();
        }

        // Cancel all port proxies for this pod
        let proxy_keys: Vec<(String, u16)> = state
            .port_proxy_tokens
            .keys()
            .filter(|(pid, _)| pid == &id)
            .cloned()
            .collect();
        for key in proxy_keys {
            if let Some(cancel) = state.port_proxy_tokens.remove(&key) {
                cancel.cancel();
            }
        }

        let container_id = state.pods.get(&id).unwrap().container_id.clone();
        let pod_name = state.pods.get(&id).unwrap().name.clone();
        let lock = state.get_or_create_pod_lock(&id);
        (
            container_id,
            pod_name,
            lock,
        )
    };

    let _guard = pod_lock.lock().await;

    // Remove Docker container if exists
    if let Some(cid) = &container_id {
        let state_guard = state.lock().await;
        if let Some(docker) = &state_guard.docker {
            let _ = containers::remove_container(docker, cid, remove_volumes).await;
        }
    }

    // Remove from state
    let mut state = state.lock().await;
    state.pods.remove(&id);
    state.pod_locks.remove(&id);

    // Remove config file
    let _ = PodStateConfig::delete(&pod_name);

    Ok(())
}

#[tauri::command]
pub async fn open_terminal(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    // Extract pod info while holding the lock
    let (container_id, pod_name, default_shell, global_terminal, remote_user, remote_workspace_folder, project_path) = {
        let state = state.lock().await;
        let pod = state
            .pods
            .get(&id)
            .ok_or_else(|| format!("Pod not found: {}", id))?;

        let cid = pod
            .container_id
            .clone()
            .ok_or_else(|| "No container ID".to_string())?;

        (
            cid,
            pod.name.clone(),
            pod.default_shell.clone(),
            state.settings.terminal.clone(),
            pod.remote_user.clone(),
            pod.remote_workspace_folder.clone(),
            pod.project_path.clone(),
        )
    };

    // Load per-pod config for shell/terminal overrides
    let pod_config = PodStateConfig::load(&pod_name)
        .ok()
        .flatten();

    // Determine the effective user: pod config override > pod.remote_user > Docker label meta > devcontainer.json > Docker inspect Config.User
    let config_user = pod_config
        .as_ref()
        .map(|c| c.remote_user.as_str())
        .unwrap_or("")
        .to_string();

    let effective_user = if !config_user.is_empty() {
        Some(config_user)
    } else if let Some(ref u) = remote_user {
        Some(u.clone())
    } else {
        // Try to detect via Docker inspect Config.User
        let state_guard = state.lock().await;
        if let Some(docker) = &state_guard.docker {
            let inspect_user = containers::inspect_container(docker, &container_id)
                .await
                .ok()
                .and_then(|info| info.config)
                .and_then(|c| c.user)
                .filter(|u| !u.is_empty());
            inspect_user
        } else {
            None
        }
    };

    // Determine working directory: remote_workspace_folder > pod config working_dir > /workspaces/{dir_name}
    let effective_workdir = if let Some(ref w) = remote_workspace_folder {
        Some(w.clone())
    } else if let Some(ref cfg) = pod_config {
        if !cfg.working_dir.is_empty() {
            Some(cfg.working_dir.clone())
        } else {
            // Fall back to /workspaces/{dir_name}
            let dir_name = std::path::Path::new(&project_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string());
            dir_name.map(|n| format!("/workspaces/{}", n))
        }
    } else {
        let dir_name = std::path::Path::new(&project_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string());
        dir_name.map(|n| format!("/workspaces/{}", n))
    };

    // Determine shell: pod config override > pod.default_shell > detect from container
    // Priority inside detect_shell: $SHELL → fish → zsh → bash → sh
    let shell = if let Some(ref cfg) = pod_config {
        if terminal::shell::is_real_shell(&cfg.shell) {
            cfg.shell.clone()
        } else if terminal::shell::is_real_shell(&default_shell) {
            default_shell
        } else {
            let state_guard = state.lock().await;
            if let Some(docker) = &state_guard.docker {
                terminal::shell::detect_shell(docker, &container_id, effective_user.as_deref()).await
            } else {
                "/bin/sh".to_string()
            }
        }
    } else if terminal::shell::is_real_shell(&default_shell) {
        default_shell
    } else {
        let state_guard = state.lock().await;
        if let Some(docker) = &state_guard.docker {
            terminal::shell::detect_shell(docker, &container_id, effective_user.as_deref()).await
        } else {
            "/bin/sh".to_string()
        }
    };

    // Determine terminal: pod config override > global settings > auto-detect
    let terminal_override = pod_config
        .as_ref()
        .map(|c| c.terminal_override.as_str())
        .unwrap_or("");
    let effective_override = if !terminal_override.is_empty() {
        terminal_override
    } else {
        &global_terminal
    };

    let terminal_name = terminal::detect_terminal(effective_override)
        .map_err(|e| e.to_string())?;

    terminal::launch_terminal(
        &terminal_name,
        &container_id,
        &shell,
        effective_user.as_deref(),
        effective_workdir.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn expose_port(
    state: State<'_, AppState>,
    pod_id: String,
    container_port: u16,
    host_port: u16,
) -> Result<ExposedPort, String> {
    // Get container IP via Docker inspect
    let container_ip = {
        let state_guard = state.lock().await;
        let container_id = state_guard
            .pods
            .get(&pod_id)
            .and_then(|p| p.container_id.clone())
            .ok_or_else(|| "No container ID for pod".to_string())?;
        let docker = state_guard
            .docker
            .as_ref()
            .ok_or_else(|| "Docker not connected".to_string())?;
        let info = containers::inspect_container(docker, &container_id)
            .await
            .map_err(|e| e.to_string())?;
        let ns = info.network_settings;
        let direct_ip = ns
            .as_ref()
            .and_then(|n| n.ip_address.as_ref())
            .filter(|ip| !ip.is_empty())
            .cloned();
        let bridge_ip = ns
            .as_ref()
            .and_then(|n| n.networks.as_ref())
            .and_then(|nets| nets.values().next())
            .and_then(|net| net.ip_address.clone())
            .filter(|ip| !ip.is_empty());
        direct_ip
            .or(bridge_ip)
            .ok_or_else(|| "Could not determine container IP".to_string())?
    };

    // Try to start the port proxy
    let cancel = tokio_util::sync::CancellationToken::new();
    let proxy_cancel = cancel.clone();
    let proxy_ip = container_ip.clone();

    // Test bind first to detect busy port without spawning
    let status = match std::net::TcpListener::bind(format!("0.0.0.0:{}", host_port)) {
        Ok(_listener) => {
            // Port is free, drop the test listener and spawn the async proxy
            drop(_listener);
            let cancel_c = proxy_cancel.clone();
            tokio::spawn(async move {
                if let Err(e) =
                    crate::network::expose::start_port_proxy(host_port, proxy_ip, container_port, cancel_c)
                        .await
                {
                    tracing::error!("Port proxy error: {}", e);
                }
            });
            crate::types::PortStatus::Active
        }
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            tracing::warn!("Host port {} is busy", host_port);
            crate::types::PortStatus::HostPortBusy
        }
        Err(e) => {
            return Err(format!("Failed to bind port {}: {}", host_port, e));
        }
    };

    let mut state = state.lock().await;
    let pod = state
        .pods
        .get_mut(&pod_id)
        .ok_or_else(|| format!("Pod not found: {}", pod_id))?;

    // Determine protocol from detected ports
    let protocol = pod
        .detected_ports
        .iter()
        .find(|p| p.container_port == container_port)
        .map(|p| p.protocol.clone())
        .unwrap_or_default();

    let exposed = ExposedPort {
        container_port,
        host_port,
        protocol,
        status,
        auto_expose: true,
    };

    // Remove from detected
    pod.detected_ports
        .retain(|p| p.container_port != container_port);

    pod.exposed_ports.push(exposed.clone());

    // Store the cancellation token for this proxy
    if exposed.status == crate::types::PortStatus::Active {
        state
            .port_proxy_tokens
            .insert((pod_id.clone(), container_port), cancel);
    }

    Ok(exposed)
}

#[tauri::command]
pub async fn unexpose_port(
    state: State<'_, AppState>,
    pod_id: String,
    container_port: u16,
) -> Result<(), String> {
    let mut state = state.lock().await;
    let pod = state
        .pods
        .get_mut(&pod_id)
        .ok_or_else(|| format!("Pod not found: {}", pod_id))?;

    pod.exposed_ports
        .retain(|p| p.container_port != container_port);

    // Cancel the port proxy
    if let Some(cancel) = state.port_proxy_tokens.remove(&(pod_id.clone(), container_port)) {
        cancel.cancel();
    }

    Ok(())
}

#[tauri::command]
pub async fn ignore_port(
    state: State<'_, AppState>,
    pod_id: String,
    container_port: u16,
) -> Result<(), String> {
    let mut state = state.lock().await;
    let pod = state
        .pods
        .get_mut(&pod_id)
        .ok_or_else(|| format!("Pod not found: {}", pod_id))?;

    pod.detected_ports
        .retain(|p| p.container_port != container_port);

    Ok(())
}

#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> Result<crate::config::settings::Settings, String> {
    let state = state.lock().await;
    Ok(state.settings.clone())
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings: crate::config::settings::Settings,
) -> Result<(), String> {
    settings.save().map_err(|e| e.to_string())?;
    let mut state = state.lock().await;
    state.settings = settings;
    Ok(())
}

#[tauri::command]
pub async fn check_dependencies() -> Result<Vec<DependencyCheck>, String> {
    let mut checks = Vec::new();

    // Check Docker
    let docker_ok = bollard::Docker::connect_with_local_defaults().is_ok();
    let docker_ping = if docker_ok {
        if let Ok(docker) = bollard::Docker::connect_with_local_defaults() {
            docker.ping().await.is_ok()
        } else {
            false
        }
    } else {
        false
    };

    checks.push(DependencyCheck {
        name: "Docker daemon".to_string(),
        satisfied: docker_ping,
        details: if docker_ping {
            "Running".to_string()
        } else {
            "Not running or not accessible".to_string()
        },
        fix_hint: if docker_ping {
            None
        } else {
            Some("Start Docker Desktop or Docker daemon".to_string())
        },
    });

    // Check devcontainer CLI
    let devcontainer_ok = cli::is_devcontainer_available().await;
    checks.push(DependencyCheck {
        name: "devcontainer CLI".to_string(),
        satisfied: devcontainer_ok,
        details: if devcontainer_ok {
            "Installed".to_string()
        } else {
            "Not found".to_string()
        },
        fix_hint: if devcontainer_ok {
            None
        } else {
            Some("Install: npm install -g @devcontainers/cli".to_string())
        },
    });

    // Check terminal
    let terminal_result = terminal::detect_terminal("");
    let terminal_ok = terminal_result.is_ok();
    checks.push(DependencyCheck {
        name: "Terminal".to_string(),
        satisfied: terminal_ok,
        details: if let Ok(t) = terminal_result {
            t
        } else {
            "Not found".to_string()
        },
        fix_hint: if terminal_ok {
            None
        } else {
            Some("Install a terminal emulator (alacritty, kitty, etc.)".to_string())
        },
    });

    Ok(checks)
}

#[tauri::command]
pub async fn get_detected_terminal(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let state = state.lock().await;
    let terminal_name = terminal::detect_terminal(&state.settings.terminal)
        .map_err(|e| e.to_string())?;
    Ok(terminal_name)
}

#[tauri::command]
pub async fn get_default_settings() -> Result<crate::config::settings::Settings, String> {
    Ok(crate::config::settings::Settings::default())
}

#[tauri::command]
pub async fn check_docker_health(state: State<'_, AppState>) -> Result<bool, String> {
    let state = state.lock().await;
    if let Some(docker) = &state.docker {
        Ok(docker.ping().await.is_ok())
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn get_pod_logs(
    state: State<'_, AppState>,
    id: String,
    tail: Option<usize>,
    filter: Option<String>,
) -> Result<Vec<crate::types::LogEntry>, String> {
    let state = state.lock().await;
    if let Some(buf) = state.log_buffers.get(&id) {
        if let Some(keyword) = filter {
            Ok(buf.search(&keyword))
        } else {
            Ok(buf.tail(tail.unwrap_or(500)))
        }
    } else {
        Ok(Vec::new())
    }
}

#[tauri::command]
pub async fn clear_pod_logs(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let mut state = state.lock().await;
    if let Some(buf) = state.log_buffers.get_mut(&id) {
        buf.clear();
    }
    Ok(())
}

#[tauri::command]
pub async fn cancel_build(
    state: State<'_, AppState>,
    app: AppHandle,
    id: String,
) -> Result<(), String> {
    let mut state = state.lock().await;
    if let Some(cancel) = state.build_cancel_tokens.remove(&id) {
        cancel.cancel();
    }
    if let Some(pod) = state.pods.get_mut(&id) {
        pod.status = PodStatus::Stopped;
        pod.error_message = None;
        let _ = app.emit(
            "pod-status-changed",
            &PodStatusChanged {
                pod_id: id.clone(),
                status: PodStatus::Stopped,
                error_message: None,
            },
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn get_pod_settings(
    state: State<'_, AppState>,
    id: String,
) -> Result<PodStateConfig, String> {
    let state = state.lock().await;
    let pod = state
        .pods
        .get(&id)
        .ok_or_else(|| format!("Pod not found: {}", id))?;

    PodStateConfig::load(&pod.name)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Pod config not found".to_string())
}

#[tauri::command]
pub async fn save_pod_settings(
    state: State<'_, AppState>,
    id: String,
    config: PodStateConfig,
) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())?;

    // Sync config overrides to pod runtime state
    let mut state = state.lock().await;
    if let Some(pod) = state.pods.get_mut(&id) {
        if config.alias.is_empty() {
            pod.alias = None;
        } else {
            pod.alias = Some(config.alias.clone());
        }
        // Sync remote_user override
        if !config.remote_user.is_empty() {
            pod.remote_user = Some(config.remote_user.clone());
        }
        // Sync working_dir override
        if !config.working_dir.is_empty() {
            pod.remote_workspace_folder = Some(config.working_dir.clone());
        }
        // Sync shell override
        if !config.shell.is_empty() {
            pod.default_shell = config.shell.clone();
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn open_in_file_manager(path: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    let cmd = "xdg-open";
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(target_os = "windows")]
    let cmd = "explorer";

    std::process::Command::new(cmd)
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open file manager: {}", e))?;

    Ok(())
}

struct DevcontainerUpResult {
    container_id: Option<String>,
    remote_user: Option<String>,
    remote_workspace_folder: Option<String>,
}

fn parse_devcontainer_output(output: &str) -> DevcontainerUpResult {
    let mut result = DevcontainerUpResult {
        container_id: None,
        remote_user: None,
        remote_workspace_folder: None,
    };

    fn extract_fields(value: &serde_json::Value, result: &mut DevcontainerUpResult) -> bool {
        let mut found = false;
        if let Some(id) = value.get("containerId").and_then(|v| v.as_str()) {
            result.container_id = Some(id.to_string());
            found = true;
        }
        if let Some(user) = value.get("remoteUser").and_then(|v| v.as_str()) {
            result.remote_user = Some(user.to_string());
        }
        if let Some(folder) = value.get("remoteWorkspaceFolder").and_then(|v| v.as_str()) {
            result.remote_workspace_folder = Some(folder.to_string());
        }
        found
    }

    // Try parsing full output as JSON
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(output) {
        if extract_fields(&value, &mut result) {
            return result;
        }
    }

    // Try line-by-line for JSON fragments
    for line in output.lines() {
        let trimmed = line.trim();
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if extract_fields(&value, &mut result) {
                return result;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_devcontainer_output_full_json() {
        let output = r#"{"outcome":"success","containerId":"abc123def456","remoteUser":"vscode","remoteWorkspaceFolder":"/workspaces/my-project"}"#;
        let result = parse_devcontainer_output(output);
        assert_eq!(result.container_id.as_deref(), Some("abc123def456"));
        assert_eq!(result.remote_user.as_deref(), Some("vscode"));
        assert_eq!(result.remote_workspace_folder.as_deref(), Some("/workspaces/my-project"));
    }

    #[test]
    fn test_parse_devcontainer_output_minimal_json() {
        let output = r#"{"containerId":"abc123"}"#;
        let result = parse_devcontainer_output(output);
        assert_eq!(result.container_id.as_deref(), Some("abc123"));
        assert_eq!(result.remote_user, None);
        assert_eq!(result.remote_workspace_folder, None);
    }

    #[test]
    fn test_parse_devcontainer_output_multiline() {
        let output = "Building container...\nStep 1/5: FROM node:20\n\n{\"outcome\":\"success\",\"containerId\":\"deadbeef\",\"remoteUser\":\"node\"}\n";
        let result = parse_devcontainer_output(output);
        assert_eq!(result.container_id.as_deref(), Some("deadbeef"));
        assert_eq!(result.remote_user.as_deref(), Some("node"));
        assert_eq!(result.remote_workspace_folder, None);
    }

    #[test]
    fn test_parse_devcontainer_output_no_json() {
        let output = "Some random output\nwith no JSON\n";
        let result = parse_devcontainer_output(output);
        assert_eq!(result.container_id, None);
        assert_eq!(result.remote_user, None);
        assert_eq!(result.remote_workspace_folder, None);
    }
}
