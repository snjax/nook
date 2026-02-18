#![cfg(feature = "test-api")]

use serde::{Deserialize, Serialize};

use crate::types::{DetectedPort, ExposedPort, PodStatus, Process};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MockPod {
    pub id: String,
    pub name: String,
    pub project_path: String,
    pub image: String,
    pub status: PodStatus,
    #[serde(default)]
    pub uptime_secs: Option<u64>,
    #[serde(default)]
    pub cpu_percent: f64,
    #[serde(default)]
    pub memory_used: u64,
    #[serde(default)]
    pub memory_limit: u64,
    #[serde(default)]
    pub default_shell: String,
    #[serde(default)]
    pub exposed_ports: Vec<ExposedPort>,
    #[serde(default)]
    pub detected_ports: Vec<DetectedPort>,
    #[serde(default)]
    pub processes: Vec<Process>,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub started_at: Option<i64>,
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(default)]
    pub remote_user: Option<String>,
    #[serde(default)]
    pub remote_workspace_folder: Option<String>,
    #[serde(default)]
    pub container_name: Option<String>,
}

impl From<MockPod> for crate::types::Pod {
    fn from(mock: MockPod) -> Self {
        Self {
            id: mock.id.clone(),
            name: mock.name,
            project_path: mock.project_path,
            image: mock.image,
            status: mock.status,
            uptime_secs: mock.uptime_secs,
            cpu_percent: mock.cpu_percent,
            memory_used: mock.memory_used,
            memory_limit: mock.memory_limit,
            default_shell: mock.default_shell,
            exposed_ports: mock.exposed_ports,
            detected_ports: mock.detected_ports,
            processes: mock.processes,
            error_message: mock.error_message,
            container_id: Some(mock.id),
            started_at: mock.started_at,
            alias: mock.alias,
            remote_user: mock.remote_user,
            remote_workspace_folder: mock.remote_workspace_folder,
            container_name: mock.container_name,
        }
    }
}
