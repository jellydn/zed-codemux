use crate::{shell_escape, MuxLauncher};
use std::io::{Error, ErrorKind};
use std::process::Command;

/// Tmux multiplexer launcher
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TmuxLauncher;

impl TmuxLauncher {
    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_auto_attach_true() {
        let launcher = TmuxLauncher::new();
        let cmd = launcher.build_command("myapp", "/home/user/projects/myapp", true);
        assert_eq!(
            cmd,
            "tmux new-session -A -s 'myapp' -c '/home/user/projects/myapp'"
        );
    }

    #[test]
    fn test_build_command_auto_attach_false() {
        let launcher = TmuxLauncher::new();
        let cmd = launcher.build_command("myapp", "/home/user/projects/myapp", false);
        assert_eq!(
            cmd,
            "tmux new-session -s 'myapp' -c '/home/user/projects/myapp'"
        );
    }

    #[test]
    fn test_build_command_with_spaces_in_name() {
        let launcher = TmuxLauncher::new();
        let cmd = launcher.build_command("my app", "/home/user/my projects", true);
        assert_eq!(
            cmd,
            "tmux new-session -A -s 'my app' -c '/home/user/my projects'"
        );
    }

    #[test]
    fn test_build_command_with_quotes() {
        let launcher = TmuxLauncher::new();
        let cmd = launcher.build_command("it's", "/home/user/john's files", false);
        assert_eq!(
            cmd,
            "tmux new-session -s 'it'\"'\"'s' -c '/home/user/john'\"'\"'s files'"
        );
    }

    #[test]
    fn test_list_sessions_when_tmux_not_installed() {
        // This test verifies the behavior when tmux is not in PATH
        // We can't easily mock the Command, but we can verify that
        // a NotFound error results in an empty list
        // In practice, this test passes on systems without tmux
        let launcher = TmuxLauncher::new();
        let result = launcher.list_sessions();
        assert!(result.is_ok());
        // Result should either be empty (tmux not found) or actual sessions
    }
}
