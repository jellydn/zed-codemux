use std::io;
use std::path::PathBuf;

/// Configuration for codemux, loaded from `~/.config/codemux/config.toml` (or platform equivalent).
#[derive(Debug, Default, PartialEq)]
pub struct Config {
    /// Preferred multiplexer: "tmux" or "zellij"
    pub multiplexer: Option<String>,
    /// Whether to auto-attach to existing sessions
    pub auto_attach: Option<bool>,
}

/// Default configuration content as a TOML string with comments.
pub(crate) const DEFAULT_CONFIG_CONTENT: &str = r#"# CodeMux configuration file
# Location: ~/.config/codemux/config.toml

# Preferred multiplexer: "tmux" or "zellij"
# If not set, codemux will auto-detect from PATH (prefers tmux if both available)
multiplexer = "tmux"

# Whether to auto-attach to existing sessions
# true = attach to existing session with same name if it exists
# false = always create a new session with unique name
auto_attach = true
"#;

/// Result type for config initialization.
#[derive(Debug)]
pub enum ConfigInitResult {
    /// Config was newly created at this path.
    Created(PathBuf),
    /// Config already existed at this path.
    AlreadyExists(PathBuf),
}

/// Creates a default config file at the platform-specific config directory.
/// Returns the path where the config is located and whether it was created or already existed.
/// Returns an error only if directory creation or file writing fails.
pub fn create_default_config() -> Result<ConfigInitResult, io::Error> {
    let config_path = get_config_path();

    // Check if config already exists
    if config_path.exists() {
        return Ok(ConfigInitResult::AlreadyExists(config_path));
    }

    // Create parent directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write default config content
    std::fs::write(&config_path, DEFAULT_CONFIG_CONTENT)?;

    Ok(ConfigInitResult::Created(config_path))
}

/// Loads the config from the platform-specific config directory.
/// Returns default Config if file is missing or unreadable.
pub fn load_config() -> Config {
    let config_path = get_config_path();
    match std::fs::read_to_string(&config_path) {
        Ok(contents) => parse_config_str(&contents),
        Err(error) => {
            if error.kind() != io::ErrorKind::NotFound || config_path.exists() {
                eprintln!(
                    "[codemux] Warning: config file exists but could not be read: {}",
                    error
                );
            }
            Config::default()
        }
    }
}

/// Gets the platform-specific config directory.
fn platform_config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return Some(PathBuf::from(appdata));
        }
    }

    #[cfg(unix)]
    {
        if let Ok(home) = std::env::var("HOME") {
            let mut path = PathBuf::from(home);
            path.push(".config");
            return Some(path);
        }
    }

    None
}

/// Gets the platform-specific config file path.
fn get_config_path() -> PathBuf {
    // Check $XDG_CONFIG_HOME first, then fall back to platform-specific config dir
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        let mut path = PathBuf::from(xdg_config);
        path.push("codemux");
        path.push("config.toml");
        return path;
    }

    // Use platform-specific config dir
    if let Some(config_dir) = platform_config_dir() {
        let mut path = config_dir;
        path.push("codemux");
        path.push("config.toml");
        return path;
    }

    // Ultimate fallback (should never happen on normal systems)
    PathBuf::from("config.toml")
}

/// Parses a minimal TOML-like config string for our specific format.
///
/// NOTE: This is a simplified, lenient parser that only supports basic key-value pairs.
/// It intentionally deviates from strict TOML to be more user-friendly for simple configs.
///
/// It does NOT support:
///   - Arrays, tables, or inline tables
///   - Escaped characters in strings
///   - Multi-line strings
///   - Dotted keys
///
/// Supported formats:
///   multiplexer = "value"   (or 'value', or bare words like `tmux`)
///   auto_attach = true/false/yes/no/1/0
///
/// Lenient parsing behavior:
///   - Unquoted values like `multiplexer = tmux` are accepted and treated as strings
///   - Values are trimmed and quotes are stripped automatically
///   - This differs from strict TOML where bare words would be invalid
///   - If you need strict TOML compliance, quote all string values
///
/// Returns defaults if parsing fails.
pub fn parse_config_str(contents: &str) -> Config {
    let mut config = Config::default();

    for line in contents.lines() {
        // Remove trailing comments and trim
        let line = line.split('#').next().unwrap_or("").trim();

        if line.is_empty() {
            continue;
        }

        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();

            match key {
                "multiplexer" => {
                    let cleaned = value.trim_matches('"').trim_matches('\'');
                    if !cleaned.is_empty() {
                        config.multiplexer = Some(cleaned.to_string());
                    }
                }
                "auto_attach" => {
                    match value {
                        "true" | "yes" | "1" => config.auto_attach = Some(true),
                        "false" | "no" | "0" => config.auto_attach = Some(false),
                        _ => {} // Ignore invalid values
                    }
                }
                _ => {}
            }
        }
    }

    config
}
