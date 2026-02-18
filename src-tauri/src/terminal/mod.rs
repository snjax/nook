pub mod shell;

use crate::error::{NookError, NookResult};

/// Terminal command templates: (name, exec_template)
/// {command} is replaced with the full docker exec command
const LINUX_TERMINALS: &[(&str, &str)] = &[
    ("alacritty", "alacritty -e {command}"),
    ("kitty", "kitty {command}"),
    ("wezterm", "wezterm start -- {command}"),
    ("foot", "foot {command}"),
    ("gnome-terminal", "gnome-terminal -- {command}"),
    ("konsole", "konsole -e {command}"),
    ("xfce4-terminal", "xfce4-terminal -e \"{command}\""),
    ("xterm", "xterm -e {command}"),
];

#[cfg(target_os = "macos")]
const MACOS_TERMINAL_SCRIPT: &str = r#"tell app "Terminal" to do script "{command}""#;

#[cfg(target_os = "macos")]
const MACOS_ITERM_SCRIPT: &str =
    r#"tell app "iTerm2" to tell current window to create tab with default profile command "{command}""#;

/// Detect the best available terminal
pub fn detect_terminal(user_override: &str) -> NookResult<String> {
    if !user_override.is_empty() {
        return Ok(user_override.to_string());
    }

    // Check $TERMINAL env var
    if let Ok(terminal) = std::env::var("TERMINAL") {
        if !terminal.is_empty() && which_exists(&terminal) {
            return Ok(terminal);
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try x-terminal-emulator (Debian/Ubuntu alternatives system)
        if which_exists("x-terminal-emulator") {
            return Ok("x-terminal-emulator".to_string());
        }

        // Try the desktop environment's default terminal first
        if let Some(de_terminal) = detect_de_terminal() {
            return Ok(de_terminal);
        }

        // Try known terminals
        for (name, _) in LINUX_TERMINALS {
            if which_exists(name) {
                return Ok(name.to_string());
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Terminal.app is always available on macOS
        return Ok("Terminal.app".to_string());
    }

    Err(NookError::TerminalNotFound)
}

/// Launch a terminal with docker exec into a container
pub fn launch_terminal(
    terminal: &str,
    container_id: &str,
    shell: &str,
    user: Option<&str>,
    working_dir: Option<&str>,
) -> NookResult<()> {
    let mut docker_args = vec!["docker", "exec", "-it"];
    let user_flag;
    if let Some(u) = user {
        user_flag = u.to_string();
        docker_args.push("-u");
        docker_args.push(&user_flag);
    }
    let workdir_flag;
    if let Some(w) = working_dir {
        workdir_flag = w.to_string();
        docker_args.push("-w");
        docker_args.push(&workdir_flag);
    }
    docker_args.push(container_id);
    docker_args.push(shell);
    let docker_command = docker_args.join(" ");

    #[cfg(target_os = "linux")]
    {
        let template = get_linux_template(terminal);
        let full_command = template.replace("{command}", &docker_command);

        let parts: Vec<&str> = full_command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(NookError::TerminalNotFound);
        }

        std::process::Command::new(parts[0])
            .args(&parts[1..])
            .spawn()
            .map_err(|e| {
                NookError::Other(format!("Failed to launch terminal {}: {}", terminal, e))
            })?;
    }

    #[cfg(target_os = "macos")]
    {
        let script = if terminal.contains("iTerm") {
            MACOS_ITERM_SCRIPT.replace("{command}", &docker_command)
        } else {
            MACOS_TERMINAL_SCRIPT.replace("{command}", &docker_command)
        };

        std::process::Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| {
                NookError::Other(format!("Failed to launch terminal: {}", e))
            })?;
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn get_linux_template(terminal: &str) -> String {
    for (name, template) in LINUX_TERMINALS {
        if *name == terminal || terminal.contains(name) {
            return template.to_string();
        }
    }
    // Default: try -e flag
    format!("{} -e {{command}}", terminal)
}

/// Detect the desktop environment's default terminal emulator
#[cfg(target_os = "linux")]
fn detect_de_terminal() -> Option<String> {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();

    let candidates: &[(&str, &[&str])] = &[
        ("gnome", &["gnome-terminal", "kgx"]),
        ("unity", &["gnome-terminal"]),
        ("cinnamon", &["gnome-terminal"]),
        ("kde", &["konsole"]),
        ("xfce", &["xfce4-terminal"]),
        ("mate", &["mate-terminal"]),
        ("lxde", &["lxterminal"]),
        ("lxqt", &["qterminal"]),
    ];

    for (de, terminals) in candidates {
        if desktop.contains(de) {
            for term in *terminals {
                if which_exists(term) {
                    return Some(term.to_string());
                }
            }
        }
    }
    None
}

fn which_exists(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
