#![cfg(feature = "test-api")]

use tauri::{AppHandle, Emitter, State};

use crate::state::AppState;
use crate::test_api::mock_data::MockPod;
use crate::state::LogBuffer;
use crate::types::{
    Confidence, DetectedPort, DetectionMethod, LogBatchEvent, LogEntry, PodStatsUpdate,
    PodStatusChanged, PortDetectedEvent,
};

#[tauri::command]
pub async fn test_inject_pods(
    state: State<'_, AppState>,
    app: AppHandle,
    pods: Vec<MockPod>,
) -> Result<(), String> {
    let mut state = state.lock().await;
    state.pods.clear();

    for mock_pod in pods {
        let pod_id = mock_pod.id.clone();
        let pod: crate::types::Pod = mock_pod.into();
        let status_event = PodStatusChanged {
            pod_id: pod_id.clone(),
            status: pod.status.clone(),
            error_message: pod.error_message.clone(),
        };
        state.pods.insert(pod_id, pod);
        let _ = app.emit("pod-status-changed", &status_event);
    }

    Ok(())
}

#[tauri::command]
pub async fn test_inject_detected_port(
    state: State<'_, AppState>,
    app: AppHandle,
    pod_id: String,
    port: u16,
    protocol: String,
    process_name: Option<String>,
) -> Result<(), String> {
    let mut state = state.lock().await;

    let detected = DetectedPort {
        container_port: port,
        protocol: protocol.clone(),
        process_name: process_name.unwrap_or_default(),
        detection_method: DetectionMethod::WellKnown,
        confidence: Confidence::High,
    };

    if let Some(pod) = state.pods.get_mut(&pod_id) {
        pod.detected_ports.push(detected.clone());
    }

    let event = PortDetectedEvent {
        pod_id,
        port: detected,
    };
    app.emit("port-detected", &event).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn test_set_stats(
    state: State<'_, AppState>,
    app: AppHandle,
    pod_id: String,
    cpu_percent: f64,
    memory_used: u64,
    memory_limit: u64,
) -> Result<(), String> {
    let mut state = state.lock().await;

    if let Some(pod) = state.pods.get_mut(&pod_id) {
        pod.cpu_percent = cpu_percent;
        pod.memory_used = memory_used;
        pod.memory_limit = memory_limit;
    }

    let update = PodStatsUpdate {
        pod_id,
        cpu_percent,
        memory_used,
        memory_limit,
    };
    app.emit("pod-stats-update", &update)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn test_inject_logs(
    state: State<'_, AppState>,
    app: AppHandle,
    pod_id: String,
    entries: Vec<LogEntry>,
) -> Result<(), String> {
    let mut state = state.lock().await;
    let buf = state.log_buffers.entry(pod_id.clone()).or_insert_with(LogBuffer::new);
    buf.push_batch(entries.clone());

    let event = LogBatchEvent {
        pod_id,
        entries,
    };
    app.emit("pod-log-update", &event).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn test_reset(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut state = state.lock().await;
    state.pods.clear();
    state.cancellation_tokens.clear();
    state.protocol_cache.clear();
    state.log_buffers.clear();
    Ok(())
}
