use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{NookError, NookResult};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PortAction {
    #[default]
    Prompt,
    AutoExpose,
    Ignore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotExposeFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "default_expose_protocols")]
    pub expose_protocols: Vec<String>,

    #[serde(default = "default_not_expose_filters")]
    pub not_expose_filters: Vec<NotExposeFilter>,

    #[serde(default)]
    pub port_protocols: HashMap<String, String>,

    #[serde(default)]
    pub terminal: String,

    #[serde(default = "default_stats_interval")]
    pub stats_interval: u64,

    #[serde(default = "default_ports_scan_interval")]
    pub ports_scan_interval: u64,

    #[serde(default = "default_process_scan_interval")]
    pub process_scan_interval: u64,

    #[serde(default)]
    pub docker_socket_path: String,

    #[serde(default)]
    pub onboarding_complete: bool,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default)]
    pub port_action: PortAction,
}

fn default_expose_protocols() -> Vec<String> {
    vec![
        "http".to_string(),
        "https".to_string(),
        "postgres".to_string(),
        "mysql".to_string(),
        "redis".to_string(),
        "mongodb".to_string(),
    ]
}

fn default_not_expose_filters() -> Vec<NotExposeFilter> {
    vec![
        NotExposeFilter {
            protocol: Some("dns".to_string()),
            port: None,
        },
        NotExposeFilter {
            protocol: None,
            port: Some(22),
        },
    ]
}

fn default_stats_interval() -> u64 {
    2000
}

fn default_ports_scan_interval() -> u64 {
    3000
}

fn default_process_scan_interval() -> u64 {
    5000
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            expose_protocols: default_expose_protocols(),
            not_expose_filters: default_not_expose_filters(),
            port_protocols: HashMap::new(),
            terminal: String::new(),
            stats_interval: default_stats_interval(),
            ports_scan_interval: default_ports_scan_interval(),
            process_scan_interval: default_process_scan_interval(),
            docker_socket_path: String::new(),
            onboarding_complete: false,
            log_level: default_log_level(),
            port_action: PortAction::default(),
        }
    }
}

impl Settings {
    pub fn config_dir() -> NookResult<PathBuf> {
        let base = dirs::config_dir().ok_or_else(|| {
            NookError::ConfigError("Could not determine config directory".to_string())
        })?;
        Ok(base.join("nook"))
    }

    pub fn settings_path() -> NookResult<PathBuf> {
        Ok(Self::config_dir()?.join("settings.toml"))
    }

    pub fn load_or_default() -> NookResult<Self> {
        let path = Self::settings_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path).map_err(|e| {
                NookError::ConfigError(format!("Failed to read settings: {}", e))
            })?;
            toml::from_str(&content).map_err(|e| {
                NookError::ConfigError(format!("Failed to parse settings: {}", e))
            })
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> NookResult<()> {
        let dir = Self::config_dir()?;
        std::fs::create_dir_all(&dir).map_err(|e| {
            NookError::ConfigError(format!("Failed to create config dir: {}", e))
        })?;
        let content = toml::to_string_pretty(self).map_err(|e| {
            NookError::Serialization(format!("Failed to serialize settings: {}", e))
        })?;
        let path = Self::settings_path()?;
        std::fs::write(&path, content).map_err(|e| {
            NookError::ConfigError(format!("Failed to write settings: {}", e))
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert!(settings.expose_protocols.contains(&"http".to_string()));
        assert_eq!(settings.stats_interval, 2000);
        assert!(!settings.onboarding_complete);
    }

    #[test]
    fn test_settings_roundtrip() {
        let settings = Settings::default();
        let serialized = toml::to_string_pretty(&settings).unwrap();
        let deserialized: Settings = toml::from_str(&serialized).unwrap();
        assert_eq!(settings.expose_protocols, deserialized.expose_protocols);
        assert_eq!(settings.stats_interval, deserialized.stats_interval);
    }

    #[test]
    fn test_settings_with_custom_port_protocols() {
        let toml_str = r#"
exposeProtocols = ["http"]
terminal = "alacritty"
statsInterval = 1000
portsScanInterval = 2000
processScanInterval = 3000

[portProtocols]
3000 = "http"
5173 = "http"
8080 = "http"
"#;
        let settings: Settings = toml::from_str(toml_str).unwrap();
        assert_eq!(settings.terminal, "alacritty");
        assert_eq!(settings.port_protocols.get("3000"), Some(&"http".to_string()));
        assert_eq!(settings.stats_interval, 1000);
    }

    #[test]
    fn test_backward_compat_old_toml_without_new_fields() {
        let toml_str = r#"
exposeProtocols = ["http"]
terminal = ""
statsInterval = 2000
portsScanInterval = 3000
processScanInterval = 5000
dockerSocketPath = ""
onboardingComplete = false
"#;
        let settings: Settings = toml::from_str(toml_str).unwrap();
        assert_eq!(settings.log_level, "info");
        assert_eq!(settings.port_action, PortAction::Prompt);
    }

    #[test]
    fn test_port_action_serialization_roundtrip() {
        let mut settings = Settings::default();
        settings.port_action = PortAction::AutoExpose;

        let serialized = toml::to_string_pretty(&settings).unwrap();
        let deserialized: Settings = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.port_action, PortAction::AutoExpose);

        settings.port_action = PortAction::Ignore;
        let serialized = toml::to_string_pretty(&settings).unwrap();
        let deserialized: Settings = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.port_action, PortAction::Ignore);
    }

    #[test]
    fn test_log_level_values() {
        for level in &["trace", "debug", "info", "warn", "error"] {
            let toml_str = format!(
                r#"logLevel = "{}""#,
                level
            );
            let settings: Settings = toml::from_str(&toml_str).unwrap();
            assert_eq!(settings.log_level, *level);
        }
    }

    #[test]
    fn test_new_fields_in_default() {
        let settings = Settings::default();
        assert_eq!(settings.log_level, "info");
        assert_eq!(settings.port_action, PortAction::Prompt);
    }
}
