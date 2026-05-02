use crate::{shell_escape, MuxLauncher};
use std::io::{Error, ErrorKind};
use std::process::Command;

/// Zellij multiplexer launcher
#[derive(Debug, Clone)]
pub struct ZellijLauncher;

impl ZellijLauncher {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ZellijLauncher {
    fn default() -> Self {
        Self::new()
    }
}

impl MuxLauncher for ZellijLauncher {
    fn list_sessions(&self) -> Result<Vec<String>, Error> {
        let output = Command::new("zellij")
            .args(["list-sessions", "-n"])
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
                    // Non-zero exit but zellij is installed - likely no sessions exist
                    // Return empty list instead of error
                    Ok(Vec::new())
                }
            }
            Err(e) => {
                // Command failed to run (zellij not installed or not in PATH)
                // Return empty list - the caller should handle missing multiplexer
                if e.kind() == ErrorKind::NotFound {
                    Ok(Vec::new())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        format!("Failed to run zellij: {}", e),
                    ))
                }
            }
        }
    }

    fn build_command(&self, name: &str, _cwd: &str, auto_attach: bool) -> String {
        let escaped_name = shell_escape(name);

        if auto_attach {
            // Auto-attach mode: attach to existing session or create new
            // Note: zellij attach with -c creates the session if it doesn't exist
            format!("zellij attach {} -c", escaped_name)
        } else {
            // Always create new session
            // Note: zellij doesn't have a -c option for setting cwd in this mode.
            // The _cwd parameter is intentionally ignored here - zellij will start
            // in the current working directory. This differs from tmux behavior.
            format!("zellij -s {}", escaped_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_auto_attach_true() {
        let launcher = ZellijLauncher::new();
        let cmd = launcher.build_command("myapp", "/home/user/projects/myapp", true);
        assert_eq!(cmd, "zellij attach 'myapp' -c");
    }

    #[test]
    fn test_build_command_auto_attach_false() {
        let launcher = ZellijLauncher::new();
        let cmd = launcher.build_command("myapp", "/home/user/projects/myapp", false);
        assert_eq!(cmd, "zellij -s 'myapp'");
    }

    #[test]
    fn test_build_command_with_spaces_in_name() {
        let launcher = ZellijLauncher::new();
        let cmd = launcher.build_command("my app", "/home/user/my projects", true);
        assert_eq!(cmd, "zellij attach 'my app' -c");
    }

    #[test]
    fn test_build_command_with_quotes() {
        let launcher = ZellijLauncher::new();
        let cmd = launcher.build_command("it's", "/home/user/john's files", false);
        assert_eq!(cmd, "zellij -s 'it'\"'\"'s'");
    }

    #[test]
    fn test_list_sessions_when_zellij_not_installed() {
        // This test verifies the behavior when zellij is not in PATH
        // We can't easily mock the Command, but we can verify that
        // a NotFound error results in an empty list
        // In practice, this test passes on systems without zellij
        let launcher = ZellijLauncher::new();
        let result = launcher.list_sessions();
        assert!(result.is_ok());
        // Result should either be empty (zellij not found) or actual sessions
    }
}
