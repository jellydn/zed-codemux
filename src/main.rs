mod config;
mod detect;
mod sanitize;
mod tmux;
mod zellij;

use crate::config::{create_default_config, load_config, Config, ConfigInitResult};
use crate::detect::{detect_multiplexer, Multiplexer};

use crate::sanitize::{get_unique_session_name, sanitize_session_name};
use crate::tmux::TmuxLauncher;
use crate::zellij::ZellijLauncher;
use std::collections::HashMap;
use std::io;
use std::io::Error;

/// Trait for multiplexer launchers (tmux, zellij)
pub trait MuxLauncher {
    /// List all active sessions for this multiplexer
    fn list_sessions(&self) -> Result<Vec<String>, Error>;

    /// Build the shell command string to launch/attach to a session
    fn build_command(&self, name: &str, cwd: &str, auto_attach: bool) -> String;
}

/// POSIX shell escape: wraps input in single quotes, replacing internal `'` with `'"'"'`.
/// If input is empty, returns `''`.
#[inline]
pub(crate) fn shell_escape(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if !value.contains('\'') {
        return format!("'{}'", value);
    }
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Simple CLI parser for --version, --help, and --init
fn parse_args() -> Vec<String> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    for arg in &args {
        match arg.as_str() {
            "-v" | "--version" | "-V" => {
                println!("codemux {}", VERSION);
                std::process::exit(0);
            }
            "-h" | "--help" | "-?" => {
                println!("codemux {}", VERSION);
                println!();
                println!("Drop-in CLI binary that opens Zed terminals inside tmux or zellij.");
                println!();
                println!("Usage: codemux [OPTIONS] [ARGS]...");
                println!();
                println!("Arguments:");
                println!("  [ARGS]...  Additional arguments to pass to the shell");
                println!();
                println!("Options:");
                println!("  -h, --help     Print help");
                println!(
                    "  --init         Create default config file at ~/.config/codemux/config.toml"
                );
                println!("  -V, --version  Print version");
                std::process::exit(0);
            }
            "--init" => match create_default_config() {
                Ok(ConfigInitResult::Created(path)) => {
                    println!("Created default config at: {}", path.display());
                    std::process::exit(0);
                }
                Ok(ConfigInitResult::AlreadyExists(path)) => {
                    println!("Config already exists at: {}", path.display());
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Error creating config: {}", e);
                    std::process::exit(1);
                }
            },
            _ => {}
        }
    }

    args
}

/// Checks if debug mode is enabled via CODEMUX_DEBUG=1
pub(crate) fn debug_enabled(env: &HashMap<String, String>) -> bool {
    env.get("CODEMUX_DEBUG").map(|v| v == "1").unwrap_or(false)
}

/// Resolves auto_attach setting: env var overrides config overrides default (true)
pub(crate) fn resolve_auto_attach(env: &HashMap<String, String>, config: &Config) -> bool {
    // Priority 1: Environment variable CODEMUX_AUTO_ATTACH
    if let Some(env_val) = env.get("CODEMUX_AUTO_ATTACH") {
        return env_val.to_lowercase() == "true";
    }

    // Priority 2: Config file auto_attach field
    if let Some(config_val) = config.auto_attach {
        return config_val;
    }

    // Priority 3: Default value (true)
    true
}

/// Gets the base name of a path (last component)
pub(crate) fn get_base_name(path: &std::path::Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("session")
        .to_string()
}

/// Decides which shell to use for the fallback when no multiplexer is found
pub(crate) fn decide_fallback_shell(env: &HashMap<String, String>) -> String {
    #[cfg(unix)]
    {
        env.get("SHELL")
            .cloned()
            .unwrap_or_else(|| "/bin/sh".to_string())
    }

    #[cfg(windows)]
    {
        env.get("COMSPEC")
            .cloned()
            .unwrap_or_else(|| "cmd.exe".to_string())
    }

    #[cfg(not(any(unix, windows)))]
    {
        "/bin/sh".to_string()
    }
}

fn main() -> io::Result<()> {
    // Parse CLI arguments (handles --version and --help)
    let _args = parse_args();

    // Get current working directory
    let cwd = std::env::current_dir()?;

    // Compute base session name from CWD basename
    let base_name = get_base_name(&cwd);
    let sanitized_name = sanitize_session_name(&base_name);

    // Load config and detect multiplexer
    let config = load_config();
    let multiplexer = detect_multiplexer(&config);

    // Prepare environment map for testable functions
    let env_map: HashMap<String, String> = std::env::vars().collect();

    // Resolve settings
    let auto_attach = resolve_auto_attach(&env_map, &config);
    let debug = debug_enabled(&env_map);

    // Debug logging
    if debug {
        eprintln!("[codemux] Resolved multiplexer: {:?}", multiplexer);
        eprintln!("[codemux] Base name: {}", base_name);
        eprintln!("[codemux] Sanitized name: {}", sanitized_name);
        eprintln!("[codemux] Auto attach: {}", auto_attach);
    }

    match multiplexer {
        Some(Multiplexer::Tmux) => {
            let launcher = TmuxLauncher::new();
            run_with_launcher(&launcher, &sanitized_name, &cwd, auto_attach, debug)?;
        }
        Some(Multiplexer::Zellij) => {
            let launcher = ZellijLauncher::new();
            run_with_launcher(&launcher, &sanitized_name, &cwd, auto_attach, debug)?;
        }
        None => {
            // No multiplexer found - fallback to shell
            if debug {
                let shell = decide_fallback_shell(&env_map);
                eprintln!(
                    "[codemux] No multiplexer found, falling back to shell: {}",
                    shell
                );
            }
            run_fallback_shell(&env_map)?;
        }
    }

    // exec replaces the process, so we should never reach here on Unix
    // On Windows, the child process has already exited
    Ok(())
}

/// Runs the multiplexer launcher, selecting a unique session name if needed
fn run_with_launcher(
    launcher: &dyn MuxLauncher,
    base_name: &str,
    cwd: &std::path::Path,
    auto_attach: bool,
    debug: bool,
) -> io::Result<()> {
    // Get list of existing sessions
    let sessions = launcher.list_sessions()?;

    // Determine final session name
    let session_name = if auto_attach {
        // In auto-attach mode: if base name exists, use it; otherwise get unique name
        if sessions.contains(&base_name.to_string()) {
            base_name.to_string()
        } else {
            // Base doesn't exist, but we still need to check for collisions
            // with other suffixed names (edge case: someone manually created 'myapp-2')
            get_unique_session_name(base_name, &sessions)
        }
    } else {
        // Not in auto-attach mode: always get a unique name
        get_unique_session_name(base_name, &sessions)
    };

    // Debug logging
    if debug {
        eprintln!("[codemux] Final session name: {}", session_name);
    }

    // Build the command string
    let cwd_str = cwd.to_string_lossy().to_string();
    let command = launcher.build_command(&session_name, &cwd_str, auto_attach);

    // Debug logging
    if debug {
        eprintln!("[codemux] Full command: {}", command);
    }

    // Execute the command
    exec_command(&command)
}

/// Executes a command by exec'ing into the user's shell
#[cfg(unix)]
fn exec_command(command: &str) -> io::Result<()> {
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    let err = Command::new(&shell).args(["-l", "-c", command]).exec();

    // If exec fails, return an error (shell path is escaped for security)
    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("Failed to exec {}: {}", shell_escape(&shell), err),
    ))
}

/// Executes a command by spawning and waiting (Windows version)
#[cfg(windows)]
fn exec_command(command: &str) -> io::Result<()> {
    use std::process::Command;

    let shell = std::env::var("SHELL")
        .or_else(|_| std::env::var("COMSPEC"))
        .unwrap_or_else(|_| "cmd.exe".to_string());

    let status = Command::new(&shell).args(["/C", command]).status()?;

    // Propagate exit code
    std::process::exit(status.code().unwrap_or(1));
}

/// Fallback shell for non-Unix, non-Windows systems
#[cfg(not(any(unix, windows)))]
fn exec_command(command: &str) -> io::Result<()> {
    use std::process::Command;

    let status = Command::new("sh").args(["-c", command]).status()?;

    std::process::exit(status.code().unwrap_or(1));
}

/// Runs the fallback shell when no multiplexer is installed
fn run_fallback_shell(env: &HashMap<String, String>) -> io::Result<()> {
    let shell = decide_fallback_shell(env);

    eprintln!("codemux: tmux/zellij not found on PATH -- falling back to {}. Install tmux or zellij to enable multiplexer mode.", shell);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        use std::process::Command;

        let err = Command::new(&shell).exec();
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to exec shell {}: {}", shell_escape(&shell), err),
        ))
    }

    #[cfg(windows)]
    {
        use std::process::Command;

        let status = Command::new(&shell).status()?;
        std::process::exit(status.code().unwrap_or(1));
    }

    #[cfg(not(any(unix, windows)))]
    {
        use std::process::Command;

        let status = Command::new(&shell).status()?;
        std::process::exit(status.code().unwrap_or(1));
    }
}

#[cfg(test)]
mod main_tests;

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
#[cfg(test)]
#[path = "detect_tests.rs"]
mod detect_tests;
#[cfg(test)]
#[path = "sanitize_tests.rs"]
mod sanitize_tests;
#[cfg(test)]
#[path = "tmux_tests.rs"]
mod tmux_tests;
#[cfg(test)]
#[path = "zellij_tests.rs"]
mod zellij_tests;
