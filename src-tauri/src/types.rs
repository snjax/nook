use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PodStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pod {
    pub id: String,
    pub name: String,
    pub project_path: String,
    pub image: String,
    pub status: PodStatus,
    pub uptime_secs: Option<u64>,
    pub cpu_percent: f64,
    pub memory_used: u64,
    pub memory_limit: u64,
    pub default_shell: String,
    pub exposed_ports: Vec<ExposedPort>,
    pub detected_ports: Vec<DetectedPort>,
    pub processes: Vec<Process>,
    pub error_message: Option<String>,
    pub container_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<i64>,
    pub alias: Option<String>,
    pub remote_user: Option<String>,
    pub remote_workspace_folder: Option<String>,
    pub container_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PortStatus {
    Active,
    HostPortBusy,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExposedPort {
    pub container_port: u16,
    pub host_port: u16,
    pub protocol: String,
    pub status: PortStatus,
    pub auto_expose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DetectionMethod {
    WellKnown,
    ProcessName,
    Nmap,
    BannerGrab,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Confidence {
    High,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedPort {
    pub container_port: u16,
    pub protocol: String,
    pub process_name: String,
    pub detection_method: DetectionMethod,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Process {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_bytes: u64,
}

// Log types

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum LogSource {
    Build,
    Container,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub timestamp: i64,
    pub message: String,
    pub source: LogSource,
    pub level: LogLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogBatchEvent {
    pub pod_id: String,
    pub entries: Vec<LogEntry>,
}

// Event payloads

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodStatsUpdate {
    pub pod_id: String,
    pub cpu_percent: f64,
    pub memory_used: u64,
    pub memory_limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortDetectedEvent {
    pub pod_id: String,
    pub port: DetectedPort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessListUpdate {
    pub pod_id: String,
    pub processes: Vec<Process>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodStatusChanged {
    pub pod_id: String,
    pub status: PodStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyCheck {
    pub name: String,
    pub satisfied: bool,
    pub details: String,
    pub fix_hint: Option<String>,
}
