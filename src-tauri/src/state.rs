use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use bollard::Docker;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::config::settings::Settings;
use crate::types::{LogEntry, Pod};

const MAX_LOG_LINES: usize = 10_000;

#[derive(Default)]
pub struct LogBuffer {
    entries: VecDeque<LogEntry>,
}

impl LogBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, entry: LogEntry) {
        if self.entries.len() >= MAX_LOG_LINES {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    pub fn push_batch(&mut self, entries: Vec<LogEntry>) {
        for entry in entries {
            self.push(entry);
        }
    }

    pub fn tail(&self, n: usize) -> Vec<LogEntry> {
        self.entries.iter().rev().take(n).rev().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn search(&self, keyword: &str) -> Vec<LogEntry> {
        let lower = keyword.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.message.to_lowercase().contains(&lower))
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub struct AppStateInner {
    pub docker: Option<Docker>,
    pub pods: HashMap<String, Pod>,
    pub pod_locks: HashMap<String, Arc<Mutex<()>>>,
    pub cancellation_tokens: HashMap<String, CancellationToken>,
    pub port_proxy_tokens: HashMap<(String, u16), CancellationToken>,
    pub build_cancel_tokens: HashMap<String, CancellationToken>,
    pub protocol_cache: HashMap<(u16, String), String>,
    pub log_buffers: HashMap<String, LogBuffer>,
    pub settings: Settings,
}

impl AppStateInner {
    pub fn new(settings: Settings) -> Self {
        Self {
            docker: None,
            pods: HashMap::new(),
            pod_locks: HashMap::new(),
            cancellation_tokens: HashMap::new(),
            port_proxy_tokens: HashMap::new(),
            build_cancel_tokens: HashMap::new(),
            protocol_cache: HashMap::new(),
            log_buffers: HashMap::new(),
            settings,
        }
    }

    pub fn get_or_create_pod_lock(&mut self, pod_id: &str) -> Arc<Mutex<()>> {
        self.pod_locks
            .entry(pod_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }
}

pub type AppState = Arc<Mutex<AppStateInner>>;

pub fn create_app_state(settings: Settings) -> AppState {
    Arc::new(Mutex::new(AppStateInner::new(settings)))
}
