pub mod commands;
pub mod config;
pub mod devcontainer;
pub mod docker;
pub mod error;
pub mod network;
pub mod state;
pub mod terminal;
pub mod test_api;
pub mod types;

use config::settings::Settings;
use state::create_app_state;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Work around WebKitGTK crash on Wayland with DMA-BUF renderer
    #[cfg(target_os = "linux")]
    if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    let settings = Settings::load_or_default().unwrap_or_default();

    let default_filter = format!("nook={}", settings.log_level);
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| default_filter.into()),
        )
        .init();

    let app_state = create_app_state(settings);

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::list_pods,
            commands::add_pod,
            commands::start_pod,
            commands::stop_pod,
            commands::restart_pod,
            commands::force_stop_pod,
            commands::rebuild_pod,
            commands::remove_pod,
            commands::open_terminal,
            commands::expose_port,
            commands::unexpose_port,
            commands::ignore_port,
            commands::get_settings,
            commands::save_settings,
            commands::check_dependencies,
            commands::get_detected_terminal,
            commands::get_default_settings,
            commands::check_docker_health,
            commands::get_pod_logs,
            commands::clear_pod_logs,
            commands::cancel_build,
            commands::get_pod_settings,
            commands::save_pod_settings,
            commands::open_in_file_manager,
        ]);

    #[cfg(feature = "test-api")]
    {
        builder = builder.invoke_handler(tauri::generate_handler![
            commands::list_pods,
            commands::add_pod,
            commands::start_pod,
            commands::stop_pod,
            commands::restart_pod,
            commands::force_stop_pod,
            commands::rebuild_pod,
            commands::remove_pod,
            commands::open_terminal,
            commands::expose_port,
            commands::unexpose_port,
            commands::ignore_port,
            commands::get_settings,
            commands::save_settings,
            commands::check_dependencies,
            commands::get_detected_terminal,
            commands::get_default_settings,
            commands::check_docker_health,
            commands::get_pod_logs,
            commands::clear_pod_logs,
            commands::cancel_build,
            commands::get_pod_settings,
            commands::save_pod_settings,
            commands::open_in_file_manager,
            test_api::commands::test_inject_pods,
            test_api::commands::test_inject_detected_port,
            test_api::commands::test_set_stats,
            test_api::commands::test_reset,
            test_api::commands::test_inject_logs,
        ]);
    }

    builder
        .setup(|app| {
            // Initialize Docker connection
            let state = app.state::<state::AppState>();
            let state_clone = state.inner().clone();
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match bollard::Docker::connect_with_local_defaults() {
                    Ok(docker) => {
                        match docker.ping().await {
                            Ok(_) => {
                                tracing::info!("Docker connection established");

                                // Collect running pods that need monitoring
                                let mut running_pods: Vec<(String, String)> = Vec::new();
                                let interval;
                                let port_interval;
                                let port_overrides: std::collections::HashMap<String, String>;

                                {
                                    let mut state = state_clone.lock().await;
                                    state.docker = Some(docker);

                                    // Load existing pods from Docker
                                    if let Some(ref docker) = state.docker {
                                        match crate::docker::containers::list_containers(docker).await {
                                            Ok(pods) => {
                                                for mut pod in pods {
                                                    // Merge with saved config by project_path
                                                    if let Ok(Some(saved)) =
                                                        crate::config::pod_state::PodStateConfig::find_by_project_path(&pod.project_path)
                                                    {
                                                        if !saved.alias.is_empty() {
                                                            pod.alias = Some(saved.alias);
                                                        }
                                                        if !saved.shell.is_empty() {
                                                            pod.default_shell = saved.shell;
                                                        }
                                                        if pod.remote_user.is_none() && !saved.remote_user.is_empty() {
                                                            pod.remote_user = Some(saved.remote_user);
                                                        }
                                                        if pod.remote_workspace_folder.is_none() && !saved.working_dir.is_empty() {
                                                            pod.remote_workspace_folder = Some(saved.working_dir);
                                                        }
                                                    } else {
                                                        let config = crate::config::pod_state::PodStateConfig {
                                                            name: pod.name.clone(),
                                                            project_path: pod.project_path.clone(),
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
                                                        if let Err(e) = config.save() {
                                                            tracing::warn!("Failed to save default pod config: {}", e);
                                                        }
                                                    }

                                                    // Track running pods for background monitoring
                                                    if pod.status == crate::types::PodStatus::Running {
                                                        if let Some(ref cid) = pod.container_id {
                                                            running_pods.push((pod.id.clone(), cid.clone()));
                                                        }
                                                    }

                                                    state.pods.insert(pod.id.clone(), pod);
                                                }
                                                tracing::info!(
                                                    "Loaded {} pods from Docker",
                                                    state.pods.len()
                                                );
                                            }
                                            Err(e) => {
                                                tracing::warn!(
                                                    "Failed to list containers: {}",
                                                    e
                                                );
                                            }
                                        }
                                    }

                                    interval = state.settings.process_scan_interval;
                                    port_interval = state.settings.ports_scan_interval;
                                    port_overrides = state.settings.port_protocols.clone();
                                } // release state lock

                                // Spawn monitoring tasks for already-running pods
                                for (pod_id, container_id) in running_pods {
                                    let state_guard = state_clone.lock().await;
                                    if let Some(ref docker) = state_guard.docker {
                                        let cancel = tokio_util::sync::CancellationToken::new();
                                        // Must drop guard before re-locking to insert token
                                        let docker = docker.clone();
                                        drop(state_guard);

                                        {
                                            let mut s = state_clone.lock().await;
                                            s.cancellation_tokens.insert(pod_id.clone(), cancel.clone());
                                        }

                                        // Stats streaming
                                        let d = docker.clone();
                                        let cid = container_id.clone();
                                        let pid = pod_id.clone();
                                        let ah = app_handle.clone();
                                        let ct = cancel.clone();
                                        tokio::spawn(async move {
                                            crate::docker::stats::stream_stats(d, cid, pid, ah, ct).await;
                                        });

                                        // Log streaming
                                        let d = docker.clone();
                                        let cid = container_id.clone();
                                        let pid = pod_id.clone();
                                        let ah = app_handle.clone();
                                        let ct = cancel.clone();
                                        let sc = state_clone.clone();
                                        tokio::spawn(async move {
                                            crate::docker::logs::stream_logs(d, cid, pid, ah, sc, ct, 100).await;
                                        });

                                        // Process polling
                                        let d = docker.clone();
                                        let cid = container_id.clone();
                                        let pid = pod_id.clone();
                                        let ah = app_handle.clone();
                                        let ct = cancel.clone();
                                        tokio::spawn(async move {
                                            crate::docker::processes::poll_processes(d, cid, pid, ah, ct, interval).await;
                                        });

                                        // Port scanning
                                        let d = docker.clone();
                                        let cid = container_id.clone();
                                        let pid = pod_id.clone();
                                        let ah = app_handle.clone();
                                        let ct = cancel.clone();
                                        let po = port_overrides.clone();
                                        tokio::spawn(async move {
                                            crate::network::scan::poll_ports(d, cid, pid, ah, ct, port_interval, po).await;
                                        });

                                        tracing::info!("Started monitoring for running pod {}", pod_id);
                                    } else {
                                        drop(state_guard);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Docker ping failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to connect to Docker: {}", e);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
