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
    fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "tmux" => Some(Multiplexer::Tmux),
            "zellij" => Some(Multiplexer::Zellij),
            _ => None,
        }
    }
}

/// Detects which multiplexer to use, following the priority order:
/// 1. Environment variable `CODEMUX_MULTIPLEXER`
/// 2. Config file `multiplexer` field
/// 3. PATH probe (prefer tmux, then zellij)
///
/// Returns `None` if no multiplexer is found.
#[allow(dead_code)]
pub fn detect_multiplexer(config: &Config) -> Option<Multiplexer> {
    // Priority 1: Check environment variable
    if let Ok(env_mux) = std::env::var("CODEMUX_MULTIPLEXER") {
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

    // Priority 3: Probe PATH via `which` crate
    // Prefer tmux first, then zellij
    if which::which("tmux").is_ok() {
        return Some(Multiplexer::Tmux);
    }

    if which::which("zellij").is_ok() {
        return Some(Multiplexer::Zellij);
    }

    // No multiplexer found
    None
}

/// Detects which multiplexer to use, with explicit environment variable injection.
/// This helper is useful for testing to avoid mutating the real environment.
#[allow(dead_code)]
pub fn detect_multiplexer_with_env(
    config: &Config,
    env: &std::collections::HashMap<String, String>,
) -> Option<Multiplexer> {
    // Priority 1: Check environment variable
    if let Some(env_mux) = env.get("CODEMUX_MULTIPLEXER") {
        if let Some(mux) = Multiplexer::from_name(env_mux) {
            return Some(mux);
        }
    }

    // Priority 2: Check config file
    if let Some(ref config_mux) = config.multiplexer {
        if let Some(mux) = Multiplexer::from_name(config_mux) {
            return Some(mux);
        }
    }

    // Priority 3: Probe PATH via `which` crate
    // Prefer tmux first, then zellij
    if which::which("tmux").is_ok() {
        return Some(Multiplexer::Tmux);
    }

    if which::which("zellij").is_ok() {
        return Some(Multiplexer::Zellij);
    }

    // No multiplexer found
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_env_var_tmux() {
        let mut env = HashMap::new();
        env.insert("CODEMUX_MULTIPLEXER".to_string(), "tmux".to_string());
        let config = Config::default();

        let result = detect_multiplexer_with_env(&config, &env);
        assert_eq!(result, Some(Multiplexer::Tmux));
    }

    #[test]
    fn test_env_var_zellij() {
        let mut env = HashMap::new();
        env.insert("CODEMUX_MULTIPLEXER".to_string(), "zellij".to_string());
        let config = Config::default();

        let result = detect_multiplexer_with_env(&config, &env);
        assert_eq!(result, Some(Multiplexer::Zellij));
    }

    #[test]
    fn test_env_var_case_insensitive() {
        let mut env = HashMap::new();
        env.insert("CODEMUX_MULTIPLEXER".to_string(), "TMUX".to_string());
        let config = Config::default();

        let result = detect_multiplexer_with_env(&config, &env);
        assert_eq!(result, Some(Multiplexer::Tmux));
    }

    #[test]
    fn test_env_var_invalid_falls_through() {
        // Invalid env var should fall through to config
        let mut env = HashMap::new();
        env.insert("CODEMUX_MULTIPLEXER".to_string(), "invalid".to_string());
        let config = Config {
            multiplexer: Some("zellij".to_string()),
            auto_attach: None,
        };

        let result = detect_multiplexer_with_env(&config, &env);
        assert_eq!(result, Some(Multiplexer::Zellij));
    }

    #[test]
    fn test_config_tmux() {
        let env = HashMap::new();
        let config = Config {
            multiplexer: Some("tmux".to_string()),
            auto_attach: None,
        };

        let result = detect_multiplexer_with_env(&config, &env);
        assert_eq!(result, Some(Multiplexer::Tmux));
    }

    #[test]
    fn test_config_zellij() {
        let env = HashMap::new();
        let config = Config {
            multiplexer: Some("zellij".to_string()),
            auto_attach: None,
        };

        let result = detect_multiplexer_with_env(&config, &env);
        assert_eq!(result, Some(Multiplexer::Zellij));
    }

    #[test]
    fn test_env_overrides_config() {
        // Env var should take priority over config
        let mut env = HashMap::new();
        env.insert("CODEMUX_MULTIPLEXER".to_string(), "tmux".to_string());
        let config = Config {
            multiplexer: Some("zellij".to_string()),
            auto_attach: None,
        };

        let result = detect_multiplexer_with_env(&config, &env);
        assert_eq!(result, Some(Multiplexer::Tmux));
    }

    #[test]
    fn test_config_overrides_path() {
        // Config should be used when env is not set (we can't easily test PATH probing,
        // but we verify that with env unset and config set, we get the config value)
        let env = HashMap::new();
        let config = Config {
            multiplexer: Some("zellij".to_string()),
            auto_attach: None,
        };

        let result = detect_multiplexer_with_env(&config, &env);
        assert_eq!(result, Some(Multiplexer::Zellij));
    }

    #[test]
    fn test_default_config_no_env() {
        let env = HashMap::new();
        let config = Config::default();

        // This will probe PATH - result depends on host system
        // We can't assert a specific value, but we can ensure it doesn't panic
        let _result = detect_multiplexer_with_env(&config, &env);
    }

    #[test]
    fn test_multiplexer_enum_equality() {
        assert_eq!(Multiplexer::Tmux, Multiplexer::Tmux);
        assert_eq!(Multiplexer::Zellij, Multiplexer::Zellij);
        assert_ne!(Multiplexer::Tmux, Multiplexer::Zellij);
    }

    #[test]
    fn test_multiplexer_from_name() {
        assert_eq!(Multiplexer::from_name("tmux"), Some(Multiplexer::Tmux));
        assert_eq!(Multiplexer::from_name("zellij"), Some(Multiplexer::Zellij));
        assert_eq!(Multiplexer::from_name("TMUX"), Some(Multiplexer::Tmux));
        assert_eq!(Multiplexer::from_name("ZELLIJ"), Some(Multiplexer::Zellij));
        assert_eq!(Multiplexer::from_name("invalid"), None);
        assert_eq!(Multiplexer::from_name(""), None);
    }
}
