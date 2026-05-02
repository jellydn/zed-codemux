use crate::config::Config;

/// Represents the available terminal multiplexers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Multiplexer {
    Tmux,
    Zellij,
}

impl Multiplexer {
    /// Parses a multiplexer name string into a Multiplexer enum.
    /// Returns None if the name doesn't match "tmux" or "zellij".
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "tmux" => Some(Multiplexer::Tmux),
            "zellij" => Some(Multiplexer::Zellij),
            _ => None,
        }
    }
}

/// Checks if a binary exists in PATH by searching through PATH directories.
fn find_in_path(binary: &str) -> bool {
    let path_env = std::env::var("PATH").unwrap_or_default();
    let path_sep = if cfg!(windows) { ';' } else { ':' };
    find_in_path_with_env(binary, &path_env, path_sep)
}

/// Testable version of find_in_path that accepts explicit PATH env string.
pub(crate) fn find_in_path_with_env(binary: &str, path_env: &str, path_sep: char) -> bool {
    for dir in path_env.split(path_sep) {
        if dir.is_empty() {
            continue;
        }
        let full_path = std::path::Path::new(dir).join(binary);

        // On Windows, also check for .exe extension if not already present
        #[cfg(windows)]
        {
            if !binary.ends_with(".exe") {
                let with_exe = std::path::Path::new(dir).join(format!("{}.exe", binary));
                if with_exe.is_file() {
                    return true;
                }
            }
        }

        if full_path.is_file() {
            return true;
        }
    }

    false
}

/// Core detection logic using a provided environment lookup.
/// The env_lookup closure takes a var name and returns its value if present.
///
/// NOTE: The PATH-based fallback (`find_in_path`) is not testable via the
/// env_lookup injection since it reads PATH directly. This is intentional:
/// all existing tests provide env/config values, so they return before PATH
/// probing. Making PATH testable would require additional abstraction overhead
/// for little practical benefit in current test scenarios.
pub(crate) fn detect_with_env_lookup(
    config: &Config,
    env_lookup: impl Fn(&str) -> Option<String>,
) -> Option<Multiplexer> {
    // Priority 1: Check environment variable
    if let Some(env_mux) = env_lookup("CODEMUX_MULTIPLEXER") {
        if let Some(mux) = Multiplexer::from_name(&env_mux) {
            return Some(mux);
        }
    }

    // Priority 2: Check config file
    if let Some(ref config_mux) = config.multiplexer {
        if let Some(mux) = Multiplexer::from_name(config_mux) {
            return Some(mux);
        }
    }

    // Priority 3: Probe PATH directly
    // Prefer tmux first, then zellij
    if find_in_path("tmux") {
        return Some(Multiplexer::Tmux);
    }

    if find_in_path("zellij") {
        return Some(Multiplexer::Zellij);
    }

    // No multiplexer found
    None
}

/// Detects which multiplexer to use, following the priority order:
/// 1. Environment variable `CODEMUX_MULTIPLEXER`
/// 2. Config file `multiplexer` field
/// 3. PATH probe (prefer tmux, then zellij)
///
/// Returns `None` if no multiplexer is found.
pub fn detect_multiplexer(config: &Config) -> Option<Multiplexer> {
    detect_with_env_lookup(config, |name| std::env::var(name).ok())
}
