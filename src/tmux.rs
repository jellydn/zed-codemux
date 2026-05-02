use crate::{shell_escape, MuxLauncher};
use std::io::{Error, ErrorKind};
use std::process::Command;

/// Tmux multiplexer launcher
#[derive(Debug, Clone)]
pub struct TmuxLauncher;

impl TmuxLauncher {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TmuxLauncher {
    fn default() -> Self {
        Self::new()
    }
}

impl MuxLauncher for TmuxLauncher {
    fn list_sessions(&self) -> Result<Vec<String>, Error> {
        let output = Command::new("tmux")
            .args(["list-sessions", "-F", "#{session_name}"])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let sessions: Vec<String> = stdout
                        .lines()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    Ok(sessions)
                } else {
                    // Non-zero exit but tmux is installed - likely no server running
                    // Return empty list instead of error
                    Ok(Vec::new())
                }
            }
            Err(e) => {
                // Command failed to run (tmux not installed or not in PATH)
                // Return empty list - the caller should handle missing multiplexer
                if e.kind() == ErrorKind::NotFound {
                    Ok(Vec::new())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        format!("Failed to run tmux: {}", e),
                    ))
                }
            }
        }
    }

    fn build_command(&self, name: &str, cwd: &str, auto_attach: bool) -> String {
        let escaped_name = shell_escape(name);
        let escaped_cwd = shell_escape(cwd);

        if auto_attach {
            // Auto-attach mode: create new session or attach to existing
            format!("tmux new-session -A -s {} -c {}", escaped_name, escaped_cwd)
        } else {
            // Always create new session
            format!("tmux new-session -s {} -c {}", escaped_name, escaped_cwd)
        }
    }
}
