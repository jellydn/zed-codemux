use std::path::PathBuf;

/// Configuration for codemux, loaded from `~/.config/codemux/config.toml` (or platform equivalent).
#[derive(Debug, Default, PartialEq)]
pub struct Config {
    /// Preferred multiplexer: "tmux" or "zellij"
    pub multiplexer: Option<String>,
    /// Whether to auto-attach to existing sessions
    pub auto_attach: Option<bool>,
}

/// Loads the config from the platform-specific config directory.
/// Returns default Config if file is missing or unreadable.
pub fn load_config() -> Config {
    let config_path = get_config_path();
    match std::fs::read_to_string(&config_path) {
        Ok(contents) => parse_config_str(&contents),
        Err(_) => Config::default(),
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
/// NOTE: This is a simplified parser that only supports basic key-value pairs.
/// It does NOT support:
///   - Arrays, tables, or inline tables
///   - Escaped characters in strings
///   - Multi-line strings
///   - Dotted keys
///
/// Supported formats:
///   multiplexer = "value"   (or 'value')
///   auto_attach = true/false/yes/no/1/0
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

    #[test]
    fn test_parse_with_comments() {
        let toml = r#"
# This is a comment
multiplexer = "tmux"
# Another comment
auto_attach = true
"#;
        let config = parse_config_str(toml);
        assert_eq!(config.multiplexer, Some("tmux".to_string()));
        assert_eq!(config.auto_attach, Some(true));
    }

    #[test]
    fn test_parse_with_whitespace() {
        let toml = r#"
  multiplexer   =   "zellij"  
  auto_attach   =   false  
"#;
        let config = parse_config_str(toml);
        assert_eq!(config.multiplexer, Some("zellij".to_string()));
        assert_eq!(config.auto_attach, Some(false));
    }

    #[test]
    fn test_parse_single_quotes() {
        let toml = r#"multiplexer = 'tmux'"#;
        let config = parse_config_str(toml);
        assert_eq!(config.multiplexer, Some("tmux".to_string()));
    }

    #[test]
    fn test_parse_auto_attach_variations() {
        // Test true variations
        assert_eq!(
            parse_config_str("auto_attach = true").auto_attach,
            Some(true)
        );
        assert_eq!(
            parse_config_str("auto_attach = yes").auto_attach,
            Some(true)
        );
        assert_eq!(parse_config_str("auto_attach = 1").auto_attach, Some(true));

        // Test false variations
        assert_eq!(
            parse_config_str("auto_attach = false").auto_attach,
            Some(false)
        );
        assert_eq!(
            parse_config_str("auto_attach = no").auto_attach,
            Some(false)
        );
        assert_eq!(parse_config_str("auto_attach = 0").auto_attach, Some(false));
    }

    #[test]
    fn test_parse_trailing_comments() {
        // Trailing comments should be stripped
        let toml = r#"multiplexer = "tmux" # this is a comment"#;
        let config = parse_config_str(toml);
        assert_eq!(config.multiplexer, Some("tmux".to_string()));

        let toml2 = r#"auto_attach = true # enable auto attach"#;
        let config2 = parse_config_str(toml2);
        assert_eq!(config2.auto_attach, Some(true));
    }

    #[test]
    fn test_parse_invalid_boolean_ignored() {
        // Invalid boolean values should be ignored (not treated as false)
        let toml = r#"
multiplexer = "tmux"
auto_attach = invalid_value
"#;
        let config = parse_config_str(toml);
        assert_eq!(config.multiplexer, Some("tmux".to_string()));
        assert_eq!(config.auto_attach, None); // Not Some(false), but None (ignored)
    }
}
