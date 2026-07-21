use crate::{shell_escape, MuxLauncher};
use std::io::{Error, ErrorKind};
use std::process::Command;

/// Returns the socket directory for zellij, using ZELLIJ_SOCKET_DIR if set,
/// otherwise defaulting to /tmp/z to avoid long TMPDIR paths on macOS.
/// The zellij IPC socket has a 103-byte limit, and macOS TMPDIR can be ~50 chars.
pub(crate) fn get_socket_dir() -> String {
    std::env::var("ZELLIJ_SOCKET_DIR").unwrap_or_else(|_| "/tmp/z".to_string())
}

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
        let socket_dir = get_socket_dir();
        let output = Command::new("zellij")
            .env("ZELLIJ_SOCKET_DIR", &socket_dir)
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

    fn build_command(&self, name: &str, cwd: &str, auto_attach: bool) -> String {
        let escaped_name = shell_escape(name);
        let socket_dir = shell_escape(&get_socket_dir());

        if auto_attach {
            // Auto-attach mode: attach to existing session or create new
            // Note: zellij attach with -c creates the session if it doesn't exist
            // Prepend ZELLIJ_SOCKET_DIR to avoid long TMPDIR paths on macOS
            format!(
                "ZELLIJ_SOCKET_DIR={} zellij attach {} -c",
                socket_dir, escaped_name
            )
        } else {
            // Always create new session
            // Note: zellij doesn't have a -c option for setting cwd in this mode.
            // The cwd parameter is intentionally ignored here - zellij will start
            // in the current working directory. This differs from tmux behavior.
            if !cwd.is_empty() {
                eprintln!(
                    "[codemux] Note: zellij cannot set CWD in non-auto-attach mode; using the current directory"
                );
            }
            // Prepend ZELLIJ_SOCKET_DIR to avoid long TMPDIR paths on macOS
            format!(
                "ZELLIJ_SOCKET_DIR={} zellij -s {}",
                socket_dir, escaped_name
            )
        }
    }

    // TODO: Detect when running inside a zellij session by checking the ZELLIJ
    // environment variable (analogous to tmux's $TMUX). When inside zellij,
    // `is_inside_session()` should return true and `build_inside_command()`
    // should use `zellij action new-tab` (or similar) to create a new tab
    // instead of attaching a nested session. See:
    // https://github.com/jellydn/zed-codemux/pull/12
    //
    // The current default implementation falls back to `build_command(name, cwd, true)`
    // which would run `zellij attach <name> -c` — the wrong behavior when inside zellij.
}
