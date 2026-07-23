mod config;
mod detect;
mod sanitize;
mod tmux;
mod upgrade;
mod zellij;

use crate::config::{create_default_config, load_config, Config, ConfigInitResult};
use crate::detect::{detect_multiplexer, Multiplexer};

use crate::sanitize::{get_unique_session_name, sanitize_session_name, sanitize_session_name_full};
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

    /// Returns true if currently running inside the multiplexer (e.g., inside a tmux session).
    /// Defaults to false for multiplexers that don't support this detection.
    fn is_inside_session(&self) -> bool {
        false
    }

    /// Build the shell command string for creating a new window/session when already
    /// inside the multiplexer (e.g., `tmux new-window` instead of `tmux new-session`).
    /// Defaults to `build_command(name, cwd, true)`.
    fn build_inside_command(&self, name: &str, cwd: &str) -> String {
        self.build_command(name, cwd, true)
    }
}

/// POSIX shell escape: wraps input in single quotes, replacing internal `'` with `'"'"'`.
/// If input is empty, returns `''`.
///
/// # Security
/// This prevents command injection when session names or paths contain special characters
/// (quotes, semicolons, backticks, variable substitutions, etc.) by ensuring the entire
/// string is treated as a single literal argument to the shell.
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
            "--check-version" => match upgrade::check_version_only() {
                Ok(latest) => {
                    println!("Latest version: v{} (current: v{})", latest, VERSION);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("codemux: {}", e);
                    std::process::exit(1);
                }
            },
            "--upgrade" => {
                let check_only = args.iter().any(|a| a == "--check");
                let yes = args.iter().any(|a| a == "--yes");
                match upgrade::upgrade(check_only, yes) {
                    Ok(_) => std::process::exit(0),
                    Err(e) => {
                        eprintln!("codemux: {}", e);
                        std::process::exit(1);
                    }
                }
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
    let extra_args = parse_args();

    // Get current working directory
    let cwd = std::env::current_dir()?;

    // Compute base session name from CWD basename
    let base_name = get_base_name(&cwd);
    let sanitized_name = sanitize_session_name(&base_name);
    let full_sanitized_name = sanitize_session_name_full(&base_name);

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
            run_with_launcher(
                &launcher,
                &sanitized_name,
                &full_sanitized_name,
                &cwd,
                auto_attach,
                debug,
                &extra_args,
            )?;
        }
        Some(Multiplexer::Zellij) => {
            let launcher = ZellijLauncher::new();
            run_with_launcher(
                &launcher,
                &sanitized_name,
                &full_sanitized_name,
                &cwd,
                auto_attach,
                debug,
                &extra_args,
            )?;
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
            run_fallback_shell(&env_map, &extra_args)?;
        }
    }

    // exec replaces the process, so we should never reach here on Unix
    // On Windows, the child process has already exited
    Ok(())
}

/// Resolves the session name to use, based on existing sessions and auto-attach mode.
/// Returns the existing session name if a match is found (either by truncated or
/// full sanitized name), otherwise generates a unique name.
///
/// This is a pure function extracted for testability — it does no I/O.
fn resolve_session_name(
    base_name: &str,
    full_sanitized: &str,
    sessions: &[String],
    auto_attach: bool,
) -> String {
    let full_matches = full_sanitized != base_name && sessions.iter().any(|s| s == full_sanitized);

    if auto_attach {
        if sessions.iter().any(|s| s == base_name) {
            // Exact match on truncated name
            base_name.to_string()
        } else if full_matches {
            // Match on full (untruncated) sanitized name — session exists
            // with a longer name that exceeds the 32-char limit
            full_sanitized.to_string()
        } else {
            // Base doesn't exist, but we still need to check for collisions
            // with other suffixed names (edge case: someone manually created 'myapp-2')
            get_unique_session_name(base_name, sessions)
        }
    } else {
        // Not in auto-attach mode: always get a unique name
        get_unique_session_name(base_name, sessions)
    }
}

/// Runs the multiplexer launcher when already inside an active multiplexer session.
/// Instead of creating/attaching to a session, it creates a new window/tab.
fn run_inside_session(
    launcher: &dyn MuxLauncher,
    window_name: &str,
    cwd: &std::path::Path,
    debug: bool,
    args: &[String],
) -> io::Result<()> {
    if debug {
        eprintln!(
            "[codemux] Inside multiplexer session, creating new window: {}",
            window_name
        );
    }

    let cwd_str = cwd.to_string_lossy().to_string();
    let command = launcher.build_inside_command(window_name, &cwd_str);

    if debug {
        eprintln!("[codemux] Full command: {}", command);
    }

    exec_command(&command, args)
}

/// Runs the multiplexer launcher, selecting a unique session name if needed.
/// If already inside the multiplexer, delegates to `run_inside_session` instead.
fn run_with_launcher(
    launcher: &dyn MuxLauncher,
    base_name: &str,
    full_sanitized: &str,
    cwd: &std::path::Path,
    auto_attach: bool,
    debug: bool,
    args: &[String],
) -> io::Result<()> {
    // If we're already inside the multiplexer, create a new window instead
    if launcher.is_inside_session() {
        return run_inside_session(launcher, base_name, cwd, debug, args);
    }

    // Get list of existing sessions
    let sessions = launcher.list_sessions()?;

    // Resolve session name using pure function (testable in isolation)
    let session_name = resolve_session_name(base_name, full_sanitized, &sessions, auto_attach);

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
    exec_command(&command, args)
}

/// Executes a command by exec'ing into the user's shell
#[cfg(unix)]
fn exec_command(command: &str, args: &[String]) -> io::Result<()> {
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    // Build the full command: multiplexer command + any extra args
    let full_command = if args.is_empty() {
        command.to_string()
    } else {
        format!("{} {}", command, args.join(" "))
    };

    let err = Command::new(&shell)
        .args(["-l", "-c", &full_command])
        .exec();

    // If exec fails, return an error (shell path is escaped for security)
    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("Failed to exec {}: {}", shell_escape(&shell), err),
    ))
}

/// Executes a command by spawning and waiting (Windows version)
#[cfg(windows)]
fn exec_command(command: &str, args: &[String]) -> io::Result<()> {
    use std::process::Command;

    let shell = std::env::var("SHELL")
        .or_else(|_| std::env::var("COMSPEC"))
        .unwrap_or_else(|_| "cmd.exe".to_string());

    // Build the full command: multiplexer command + any extra args
    let full_command = if args.is_empty() {
        command.to_string()
    } else {
        format!("{} {}", command, args.join(" "))
    };

    let status = Command::new(&shell).args(["/C", &full_command]).status()?;

    // Propagate exit code
    std::process::exit(status.code().unwrap_or(1));
}

/// Fallback shell for non-Unix, non-Windows systems
#[cfg(not(any(unix, windows)))]
fn exec_command(command: &str, args: &[String]) -> io::Result<()> {
    use std::process::Command;

    // Build the full command: command + any extra args
    let full_command = if args.is_empty() {
        command.to_string()
    } else {
        format!("{} {}", command, args.join(" "))
    };

    let status = Command::new("sh").args(["-c", &full_command]).status()?;

    std::process::exit(status.code().unwrap_or(1));
}

/// Runs the fallback shell when no multiplexer is installed
fn run_fallback_shell(env: &HashMap<String, String>, args: &[String]) -> io::Result<()> {
    let shell = decide_fallback_shell(env);

    eprintln!("codemux: tmux/zellij not found on PATH -- falling back to {}. Install tmux or zellij to enable multiplexer mode.", shell);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        use std::process::Command;

        let err = if args.is_empty() {
            Command::new(&shell).exec()
        } else {
            Command::new(&shell)
                .args(["-l", "-c", &args.join(" ")])
                .exec()
        };
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to exec shell {}: {}", shell_escape(&shell), err),
        ))
    }

    #[cfg(windows)]
    {
        use std::process::Command;

        let status = if args.is_empty() {
            Command::new(&shell).status()?
        } else {
            Command::new(&shell)
                .args(["/C", &args.join(" ")])
                .status()?
        };
        std::process::exit(status.code().unwrap_or(1));
    }

    #[cfg(not(any(unix, windows)))]
    {
        use std::process::Command;

        let status = if args.is_empty() {
            Command::new(&shell).status()?
        } else {
            Command::new(&shell)
                .args(["-c", &args.join(" ")])
                .status()?
        };
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
