use std::io::Error;

/// Trait for multiplexer launchers (tmux, zellij)
#[allow(dead_code)]
pub trait MuxLauncher {
    /// List all active sessions for this multiplexer
    fn list_sessions(&self) -> Result<Vec<String>, Error>;

    /// Build the shell command string to launch/attach to a session
    fn build_command(&self, name: &str, cwd: &str, auto_attach: bool) -> String;
}
