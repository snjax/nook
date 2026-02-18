use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::settings::Settings;
use crate::error::{NookError, NookResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoExposePort {
    pub container_port: u16,
    pub host_port: u16,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodNotExposeFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodStateConfig {
    pub name: String,
    pub project_path: String,

    #[serde(default)]
    pub shell: String,

    #[serde(default)]
    pub auto_expose_ports: Vec<AutoExposePort>,

    #[serde(default)]
    pub port_mappings: HashMap<String, u16>,

    #[serde(default)]
    pub alias: String,

    #[serde(default)]
    pub not_expose_filters: Vec<PodNotExposeFilter>,

    #[serde(default)]
    pub manual_expose_ports: Vec<AutoExposePort>,

    #[serde(default)]
    pub terminal_override: String,

    #[serde(default)]
    pub working_dir: String,

    #[serde(default)]
    pub bind_address: String,

    #[serde(default)]
    pub remote_user: String,
}

impl PodStateConfig {
    fn pods_dir() -> NookResult<PathBuf> {
        Ok(Settings::config_dir()?.join("pods"))
    }

    pub fn load(name: &str) -> NookResult<Option<Self>> {
        let path = Self::pods_dir()?.join(format!("{}.toml", name));
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| NookError::ConfigError(format!("Failed to read pod state: {}", e)))?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| NookError::ConfigError(format!("Failed to parse pod state: {}", e)))?;
        Ok(Some(config))
    }

    pub fn save(&self) -> NookResult<()> {
        let dir = Self::pods_dir()?;
        std::fs::create_dir_all(&dir)
            .map_err(|e| NookError::ConfigError(format!("Failed to create pods dir: {}", e)))?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| NookError::Serialization(format!("Failed to serialize pod state: {}", e)))?;
        let path = dir.join(format!("{}.toml", self.name));
        std::fs::write(&path, content)
            .map_err(|e| NookError::ConfigError(format!("Failed to write pod state: {}", e)))?;
        Ok(())
    }

    pub fn delete(name: &str) -> NookResult<()> {
        let path = Self::pods_dir()?.join(format!("{}.toml", name));
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| NookError::ConfigError(format!("Failed to delete pod state: {}", e)))?;
        }
        Ok(())
    }

    pub fn find_by_project_path(path: &str) -> NookResult<Option<Self>> {
        let configs = Self::list_all()?;
        Ok(configs.into_iter().find(|c| c.project_path == path))
    }

    pub fn list_all() -> NookResult<Vec<Self>> {
        let dir = Self::pods_dir()?;
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut configs = Vec::new();
        for entry in std::fs::read_dir(&dir)
            .map_err(|e| NookError::ConfigError(format!("Failed to read pods dir: {}", e)))?
        {
            let entry = entry.map_err(|e| {
                NookError::ConfigError(format!("Failed to read dir entry: {}", e))
            })?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "toml") {
                let content = std::fs::read_to_string(&path).map_err(|e| {
                    NookError::ConfigError(format!("Failed to read pod state: {}", e))
                })?;
                if let Ok(config) = toml::from_str::<PodStateConfig>(&content) {
                    configs.push(config);
                }
            }
        }
        Ok(configs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pod_state_roundtrip() {
        let config = PodStateConfig {
            name: "my-project".to_string(),
            project_path: "/home/user/projects/my-project".to_string(),
            shell: "/bin/zsh".to_string(),
            auto_expose_ports: vec![
                AutoExposePort {
                    container_port: 3000,
                    host_port: 3000,
                    protocol: "http".to_string(),
                },
                AutoExposePort {
                    container_port: 5432,
                    host_port: 5432,
                    protocol: "postgres".to_string(),
                },
            ],
            port_mappings: HashMap::from([("8080".to_string(), 9090)]),
            alias: "My Project".to_string(),
            not_expose_filters: Vec::new(),
            manual_expose_ports: Vec::new(),
            terminal_override: String::new(),
            working_dir: String::new(),
            bind_address: String::new(),
            remote_user: "vscode".to_string(),
        };

        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: PodStateConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.name, "my-project");
        assert_eq!(deserialized.shell, "/bin/zsh");
        assert_eq!(deserialized.auto_expose_ports.len(), 2);
        assert_eq!(deserialized.port_mappings.get("8080"), Some(&9090));
        assert_eq!(deserialized.alias, "My Project");
        assert_eq!(deserialized.remote_user, "vscode");
    }

    #[test]
    fn test_pod_state_minimal() {
        let toml_str = r#"
name = "test"
project_path = "/tmp/test"
"#;
        let config: PodStateConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.name, "test");
        assert!(config.shell.is_empty());
        assert!(config.auto_expose_ports.is_empty());
        assert!(config.port_mappings.is_empty());
        // New fields default correctly
        assert!(config.alias.is_empty());
        assert!(config.not_expose_filters.is_empty());
        assert!(config.manual_expose_ports.is_empty());
        assert!(config.terminal_override.is_empty());
        assert!(config.working_dir.is_empty());
        assert!(config.bind_address.is_empty());
        assert!(config.remote_user.is_empty());
    }
}
