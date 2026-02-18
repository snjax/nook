use bollard::exec::{CreateExecOptions, StartExecOptions};
use bollard::Docker;
use futures_util::StreamExt;

/// Detect the default shell inside a container.
///
/// Priority: $SHELL → fish → zsh → bash → sh
/// (Pod settings override is handled by the caller in open_terminal.)
pub async fn detect_shell(
    docker: &Docker,
    container_id: &str,
    remote_user: Option<&str>,
) -> String {
    // Try $SHELL (run as the target user so we get their env)
    if let Some(shell) = exec_get_env_shell(docker, container_id, remote_user).await {
        if is_real_shell(&shell) && verify_shell_exists(docker, container_id, &shell).await {
            return shell;
        }
    }

    // Try common shells in preference order: fish → zsh → bash
    for shell in &["/usr/bin/fish", "/bin/fish", "/usr/bin/zsh", "/bin/zsh", "/bin/bash"] {
        if verify_shell_exists(docker, container_id, shell).await {
            return shell.to_string();
        }
    }

    // Final fallback
    "/bin/sh".to_string()
}

/// Returns false for shells that are essentially "no real shell" (sh, nologin, false).
pub fn is_real_shell(shell: &str) -> bool {
    let base = shell.rsplit('/').next().unwrap_or(shell);
    !matches!(base, "sh" | "nologin" | "false" | "")
}


async fn exec_get_env_shell(
    docker: &Docker,
    container_id: &str,
    user: Option<&str>,
) -> Option<String> {
    let output = docker_exec_as(docker, container_id, &["sh", "-lc", "echo $SHELL"], user).await?;
    let shell = output.trim().to_string();
    if shell.is_empty() || shell == "$SHELL" {
        None
    } else {
        Some(shell)
    }
}

async fn verify_shell_exists(docker: &Docker, container_id: &str, shell: &str) -> bool {
    // `test -x` produces no output, so use `sh -c` with echo to get a non-empty result on success
    let cmd = format!("test -x {} && echo ok", shell);
    docker_exec_as(docker, container_id, &["sh", "-c", &cmd], None)
        .await
        .map(|o| o.trim() == "ok")
        .unwrap_or(false)
}

async fn docker_exec_as(
    docker: &Docker,
    container_id: &str,
    cmd: &[&str],
    user: Option<&str>,
) -> Option<String> {
    let exec = docker
        .create_exec(
            container_id,
            CreateExecOptions::<&str> {
                cmd: Some(cmd.to_vec()),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                user,
                ..Default::default()
            },
        )
        .await
        .ok()?;

    let output = docker
        .start_exec(&exec.id, Some(StartExecOptions { detach: false, ..Default::default() }))
        .await
        .ok()?;

    let mut result = String::new();
    if let bollard::exec::StartExecResults::Attached { mut output, .. } = output {
        while let Some(Ok(chunk)) = output.next().await {
            result.push_str(&chunk.to_string());
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

/// Detect the username inside a container via `id -un`
pub async fn exec_id_username(
    docker: &Docker,
    container_id: &str,
) -> Option<String> {
    let output = docker_exec_as(docker, container_id, &["id", "-un"], None).await?;
    let user = output.trim().to_string();
    if user.is_empty() {
        None
    } else {
        Some(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_real_shell_zsh() {
        assert!(is_real_shell("/bin/zsh"));
        assert!(is_real_shell("/usr/bin/zsh"));
    }

    #[test]
    fn test_is_real_shell_bash() {
        assert!(is_real_shell("/bin/bash"));
    }

    #[test]
    fn test_is_real_shell_fish() {
        assert!(is_real_shell("/usr/bin/fish"));
        assert!(is_real_shell("/bin/fish"));
    }

    #[test]
    fn test_is_real_shell_rejects_sh() {
        assert!(!is_real_shell("/bin/sh"));
        assert!(!is_real_shell("/usr/bin/sh"));
    }

    #[test]
    fn test_is_real_shell_rejects_nologin() {
        assert!(!is_real_shell("/usr/sbin/nologin"));
        assert!(!is_real_shell("/sbin/nologin"));
    }

    #[test]
    fn test_is_real_shell_rejects_false() {
        assert!(!is_real_shell("/bin/false"));
        assert!(!is_real_shell("/usr/bin/false"));
    }

    #[test]
    fn test_is_real_shell_rejects_empty() {
        assert!(!is_real_shell(""));
    }
}
