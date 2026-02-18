use bollard::container::{
    InspectContainerOptions, KillContainerOptions, ListContainersOptions,
    RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::Docker;
use std::collections::HashMap;

use crate::error::{NookError, NookResult};
use crate::types::{Pod, PodStatus};

/// Metadata extracted from devcontainer Docker labels or devcontainer.json
pub struct DevcontainerMeta {
    pub remote_user: Option<String>,
    pub remote_workspace_folder: Option<String>,
}

/// Extract remoteUser and remoteWorkspaceFolder from the `devcontainer.metadata` Docker label.
/// The label is a JSON array of objects, any of which may contain these fields.
pub fn parse_devcontainer_metadata_label(metadata_json: &str) -> DevcontainerMeta {
    let mut result = DevcontainerMeta {
        remote_user: None,
        remote_workspace_folder: None,
    };

    if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(metadata_json) {
        for obj in &arr {
            if result.remote_user.is_none() {
                if let Some(u) = obj.get("remoteUser").and_then(|v| v.as_str()) {
                    if !u.is_empty() {
                        result.remote_user = Some(u.to_string());
                    }
                }
            }
            if result.remote_workspace_folder.is_none() {
                if let Some(f) = obj.get("remoteWorkspaceFolder").and_then(|v| v.as_str()) {
                    if !f.is_empty() {
                        result.remote_workspace_folder = Some(f.to_string());
                    }
                }
            }
        }
    }

    result
}

/// Read remoteUser and remoteWorkspaceFolder from .devcontainer/devcontainer.json
pub fn read_devcontainer_json(project_path: &str) -> DevcontainerMeta {
    let mut result = DevcontainerMeta {
        remote_user: None,
        remote_workspace_folder: None,
    };

    // Try .devcontainer/devcontainer.json, then .devcontainer.json
    let candidates = [
        std::path::Path::new(project_path).join(".devcontainer/devcontainer.json"),
        std::path::Path::new(project_path).join(".devcontainer.json"),
    ];

    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            // Strip JSON comments (// and /* */) before parsing
            let cleaned = strip_json_comments(&content);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&cleaned) {
                if let Some(u) = val.get("remoteUser").and_then(|v| v.as_str()) {
                    result.remote_user = Some(u.to_string());
                }
                if let Some(f) = val.get("workspaceFolder").and_then(|v| v.as_str()) {
                    result.remote_workspace_folder = Some(f.to_string());
                }
                break;
            }
        }
    }

    result
}

/// Minimal JSON comment stripper (handles // line comments and /* block comments */)
fn strip_json_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;

    while let Some(&c) = chars.peek() {
        if in_string {
            out.push(c);
            chars.next();
            if c == '\\' {
                // skip escaped char
                if let Some(&next) = chars.peek() {
                    out.push(next);
                    chars.next();
                }
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
            out.push(c);
            chars.next();
        } else if c == '/' {
            chars.next();
            match chars.peek() {
                Some(&'/') => {
                    // line comment — skip until newline
                    for ch in chars.by_ref() {
                        if ch == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                }
                Some(&'*') => {
                    // block comment — skip until */
                    chars.next();
                    loop {
                        match chars.next() {
                            Some('*') if chars.peek() == Some(&'/') => {
                                chars.next();
                                break;
                            }
                            Some('\n') => out.push('\n'),
                            None => break,
                            _ => {}
                        }
                    }
                }
                _ => {
                    out.push('/');
                }
            }
        } else {
            out.push(c);
            chars.next();
        }
    }
    out
}

pub async fn list_containers(docker: &Docker) -> NookResult<Vec<Pod>> {
    let filters: HashMap<&str, Vec<&str>> = HashMap::from([
        ("label", vec!["devcontainer.local_folder"]),
    ]);

    let options = ListContainersOptions {
        all: true,
        filters,
        ..Default::default()
    };

    let containers = docker.list_containers(Some(options)).await?;
    let mut pods = Vec::new();

    for container in containers {
        let id = container.id.unwrap_or_default();
        let names = container.names.unwrap_or_default();
        let name = names
            .first()
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_else(|| id[..12].to_string());
        let image = container.image.unwrap_or_default();
        let state = container.state.unwrap_or_default();

        let labels = container.labels.unwrap_or_default();
        let project_path = labels
            .get("devcontainer.local_folder")
            .cloned()
            .unwrap_or_default();

        let status = match state.as_str() {
            "running" => PodStatus::Running,
            "exited" | "dead" | "created" => PodStatus::Stopped,
            _ => PodStatus::Stopped,
        };

        let container_name = Some(name.clone());

        // Derive pod name from project_path directory name (same as add_pod)
        let pod_name = std::path::Path::new(&project_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| name.clone());

        // Generate a stable UUID for the pod (not the Docker container ID)
        let pod_id = uuid::Uuid::new_v4().to_string();

        // Extract remoteUser/remoteWorkspaceFolder from Docker label metadata
        let label_meta = labels
            .get("devcontainer.metadata")
            .map(|m| parse_devcontainer_metadata_label(m))
            .unwrap_or(DevcontainerMeta {
                remote_user: None,
                remote_workspace_folder: None,
            });

        // Fall back to reading devcontainer.json from project_path
        let json_meta = if label_meta.remote_user.is_none() || label_meta.remote_workspace_folder.is_none() {
            if !project_path.is_empty() {
                read_devcontainer_json(&project_path)
            } else {
                DevcontainerMeta {
                    remote_user: None,
                    remote_workspace_folder: None,
                }
            }
        } else {
            DevcontainerMeta {
                remote_user: None,
                remote_workspace_folder: None,
            }
        };

        let remote_user = label_meta.remote_user.or(json_meta.remote_user);
        let remote_workspace_folder = label_meta.remote_workspace_folder.or(json_meta.remote_workspace_folder);

        pods.push(Pod {
            id: pod_id,
            name: pod_name,
            project_path,
            image,
            status,
            uptime_secs: None,
            cpu_percent: 0.0,
            memory_used: 0,
            memory_limit: 0,
            default_shell: String::new(),
            exposed_ports: Vec::new(),
            detected_ports: Vec::new(),
            processes: Vec::new(),
            error_message: None,
            container_id: Some(id),
            started_at: None,
            alias: None,
            remote_user,
            remote_workspace_folder,
            container_name,
        });
    }

    Ok(pods)
}

pub async fn start_container(docker: &Docker, container_id: &str) -> NookResult<()> {
    docker
        .start_container(container_id, None::<StartContainerOptions<String>>)
        .await
        .map_err(|e| match e {
            bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            } => NookError::ContainerNotFound(container_id.to_string()),
            other => NookError::Docker(other),
        })?;
    Ok(())
}

pub async fn stop_container(docker: &Docker, container_id: &str) -> NookResult<()> {
    let options = StopContainerOptions { t: 10 };
    docker
        .stop_container(container_id, Some(options))
        .await
        .map_err(|e| match e {
            bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            } => NookError::ContainerNotFound(container_id.to_string()),
            other => NookError::Docker(other),
        })?;
    Ok(())
}

pub async fn remove_container(
    docker: &Docker,
    container_id: &str,
    remove_volumes: bool,
) -> NookResult<()> {
    let options = RemoveContainerOptions {
        force: true,
        v: remove_volumes,
        ..Default::default()
    };
    docker
        .remove_container(container_id, Some(options))
        .await
        .map_err(|e| match e {
            bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            } => NookError::ContainerNotFound(container_id.to_string()),
            other => NookError::Docker(other),
        })?;
    Ok(())
}

pub async fn kill_container(docker: &Docker, container_id: &str) -> NookResult<()> {
    let options = KillContainerOptions { signal: "SIGKILL" };
    docker
        .kill_container(container_id, Some(options))
        .await
        .map_err(|e| match e {
            bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            } => NookError::ContainerNotFound(container_id.to_string()),
            other => NookError::Docker(other),
        })?;
    Ok(())
}

pub async fn inspect_container(
    docker: &Docker,
    container_id: &str,
) -> NookResult<bollard::models::ContainerInspectResponse> {
    let response = docker
        .inspect_container(container_id, None::<InspectContainerOptions>)
        .await
        .map_err(|e| match e {
            bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            } => NookError::ContainerNotFound(container_id.to_string()),
            other => NookError::Docker(other),
        })?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_devcontainer_metadata_label_with_remote_user() {
        let metadata = r#"[{"remoteUser":"vscode"},{"remoteWorkspaceFolder":"/workspaces/my-project"}]"#;
        let result = parse_devcontainer_metadata_label(metadata);
        assert_eq!(result.remote_user.as_deref(), Some("vscode"));
        assert_eq!(result.remote_workspace_folder.as_deref(), Some("/workspaces/my-project"));
    }

    #[test]
    fn test_parse_devcontainer_metadata_label_single_object() {
        let metadata = r#"[{"remoteUser":"node","remoteWorkspaceFolder":"/workspaces/app"}]"#;
        let result = parse_devcontainer_metadata_label(metadata);
        assert_eq!(result.remote_user.as_deref(), Some("node"));
        assert_eq!(result.remote_workspace_folder.as_deref(), Some("/workspaces/app"));
    }

    #[test]
    fn test_parse_devcontainer_metadata_label_empty() {
        let metadata = r#"[{}]"#;
        let result = parse_devcontainer_metadata_label(metadata);
        assert_eq!(result.remote_user, None);
        assert_eq!(result.remote_workspace_folder, None);
    }

    #[test]
    fn test_parse_devcontainer_metadata_label_invalid_json() {
        let metadata = "not json";
        let result = parse_devcontainer_metadata_label(metadata);
        assert_eq!(result.remote_user, None);
        assert_eq!(result.remote_workspace_folder, None);
    }

    #[test]
    fn test_strip_json_comments() {
        let input = r#"{
  // This is a comment
  "remoteUser": "vscode",
  /* block comment */
  "workspaceFolder": "/workspaces/app"
}"#;
        let cleaned = strip_json_comments(input);
        let val: serde_json::Value = serde_json::from_str(&cleaned).unwrap();
        assert_eq!(val.get("remoteUser").unwrap().as_str().unwrap(), "vscode");
        assert_eq!(val.get("workspaceFolder").unwrap().as_str().unwrap(), "/workspaces/app");
    }

    #[test]
    fn test_strip_json_comments_in_strings() {
        // Comments inside strings should NOT be stripped
        let input = r#"{"url": "http://example.com"}"#;
        let cleaned = strip_json_comments(input);
        let val: serde_json::Value = serde_json::from_str(&cleaned).unwrap();
        assert_eq!(val.get("url").unwrap().as_str().unwrap(), "http://example.com");
    }
}
