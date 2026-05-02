use serde::Deserialize;
use std::path::PathBuf;

/// Configuration for codemux, loaded from `~/.config/codemux/config.toml` (or platform equivalent).
#[derive(Debug, Deserialize, Default, PartialEq)]
pub struct Config {
    /// Preferred multiplexer: "tmux" or "zellij"
    pub multiplexer: Option<String>,
    /// Whether to auto-attach to existing sessions
    pub auto_attach: Option<bool>,
}

/// Loads the config from the platform-specific config directory.
/// Returns default Config if file is missing or unreadable.
#[allow(dead_code)]
pub fn load_config() -> Config {
    let config_path = get_config_path();
    match std::fs::read_to_string(&config_path) {
        Ok(contents) => parse_config_str(&contents),
        Err(_) => Config::default(),
    }
}

/// Gets the platform-specific config directory.
fn platform_config_dir() -> Option<PathBuf> {
    // Check $XDG_CONFIG_HOME first (Linux/macOS standard)
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg_config));
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: ~/Library/Application Support
        if let Ok(home) = std::env::var("HOME") {
            let mut path = PathBuf::from(home);
            path.push("Library");
            path.push("Application Support");
            return Some(path);
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: ~/.config
        if let Ok(home) = std::env::var("HOME") {
            let mut path = PathBuf::from(home);
            path.push(".config");
            return Some(path);
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: %APPDATA%
        if let Ok(appdata) = std::env::var("APPDATA") {
            return Some(PathBuf::from(appdata));
        }
    }

    // Fallback for other Unix systems
    #[cfg(all(unix, not(target_os = "macos")))]
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

/// Parses a TOML config string into a Config struct.
/// Returns defaults if parsing fails.
#[allow(dead_code)]
pub fn parse_config_str(contents: &str) -> Config {
    toml::from_str(contents).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_toml() {
        let toml = r#"
multiplexer = "tmux"
auto_attach = true
"#;
        let config = parse_config_str(toml);
        assert_eq!(config.multiplexer, Some("tmux".to_string()));
        assert_eq!(config.auto_attach, Some(true));
    }

    #[test]
    fn test_parse_zellij_config() {
        let toml = r#"
multiplexer = "zellij"
auto_attach = false
"#;
        let config = parse_config_str(toml);
        assert_eq!(config.multiplexer, Some("zellij".to_string()));
        assert_eq!(config.auto_attach, Some(false));
    }

    #[test]
    fn test_parse_empty_string() {
        let config = parse_config_str("");
        assert_eq!(config.multiplexer, None);
        assert_eq!(config.auto_attach, None);
    }

    #[test]
    fn test_parse_invalid_toml() {
        // Invalid TOML should return defaults, not panic
        let config = parse_config_str("not valid toml [ broken");
        assert_eq!(config.multiplexer, None);
        assert_eq!(config.auto_attach, None);
    }

    #[test]
    fn test_parse_partial_config() {
        // Only multiplexer specified, auto_attach omitted
        let toml = r#"multiplexer = "tmux""#;
        let config = parse_config_str(toml);
        assert_eq!(config.multiplexer, Some("tmux".to_string()));
        assert_eq!(config.auto_attach, None);
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.multiplexer, None);
        assert_eq!(config.auto_attach, None);
    }

    #[test]
    fn test_parse_config_with_extra_fields() {
        // Extra fields should be ignored
        let toml = r#"
multiplexer = "tmux"
auto_attach = true
unknown_field = "ignored"
"#;
        let config = parse_config_str(toml);
        assert_eq!(config.multiplexer, Some("tmux".to_string()));
        assert_eq!(config.auto_attach, Some(true));
    }
}
