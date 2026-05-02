use crate::config::Config;
use crate::detect::{find_in_path_with_env, Multiplexer};
use std::collections::HashMap;

/// Detects which multiplexer to use, with explicit environment variable injection.
/// This helper is useful for testing to avoid mutating the real environment.
fn detect_multiplexer_with_env(
    config: &Config,
    env: &HashMap<String, String>,
) -> Option<Multiplexer> {
    use crate::detect::detect_with_env_lookup;
    detect_with_env_lookup(config, |name| env.get(name).cloned())
}

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
    use crate::detect::Multiplexer;
    assert_eq!(Multiplexer::from_name("tmux"), Some(Multiplexer::Tmux));
    assert_eq!(Multiplexer::from_name("zellij"), Some(Multiplexer::Zellij));
    assert_eq!(Multiplexer::from_name("TMUX"), Some(Multiplexer::Tmux));
    assert_eq!(Multiplexer::from_name("ZELLIJ"), Some(Multiplexer::Zellij));
    assert_eq!(Multiplexer::from_name("invalid"), None);
    assert_eq!(Multiplexer::from_name(""), None);
}

// Tests for find_in_path_with_env (testable PATH probing)

#[test]
fn test_find_in_path_finds_binary() {
    use std::io::Write;

    // Create a temp directory with a mock binary
    let temp_dir = std::env::temp_dir().join(format!("codemux_test_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let binary_name = if cfg!(windows) { "tmux.exe" } else { "tmux" };
    let binary_path = temp_dir.join(binary_name);
    let mut file = std::fs::File::create(&binary_path).unwrap();
    file.write_all(b"#!/bin/sh\necho mock").unwrap();

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }

    // Build PATH string
    let path_env = temp_dir.to_string_lossy().to_string();

    // Should find the binary (use platform-appropriate separator)
    let path_sep = if cfg!(windows) { ';' } else { ':' };
    assert!(find_in_path_with_env("tmux", &path_env, path_sep));

    // Cleanup
    std::fs::remove_file(&binary_path).unwrap();
    std::fs::remove_dir(&temp_dir).unwrap();
}

#[test]
fn test_find_in_path_missing_binary() {
    // Empty temp directory path that doesn't exist
    let path_env = "/nonexistent/path";

    // Should not find tmux
    assert!(!find_in_path_with_env("tmux", path_env, ':'));
}

#[test]
fn test_find_in_path_empty_path_entries() {
    // PATH with empty entries (double colons) should be handled
    let path_env = "/valid/path::/another/path";

    // Should not panic on empty entries (even though no binary exists)
    assert!(!find_in_path_with_env("nonexistent", path_env, ':'));
}

#[test]
fn test_find_in_path_multiple_directories() {
    use std::io::Write;

    // Create two temp directories
    let temp_dir1 = std::env::temp_dir().join(format!("codemux_test1_{}", std::process::id()));
    let temp_dir2 = std::env::temp_dir().join(format!("codemux_test2_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir1).unwrap();
    std::fs::create_dir_all(&temp_dir2).unwrap();

    // Put binary in second directory
    let binary_name = if cfg!(windows) {
        "zellij.exe"
    } else {
        "zellij"
    };
    let binary_path = temp_dir2.join(binary_name);
    let mut file = std::fs::File::create(&binary_path).unwrap();
    file.write_all(b"#!/bin/sh\necho mock").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms).unwrap();
    }

    // Build PATH with both directories (platform-appropriate separator)
    let path_sep = if cfg!(windows) { ';' } else { ':' };
    let path_env = format!(
        "{}{}{}",
        temp_dir1.to_string_lossy(),
        path_sep,
        temp_dir2.to_string_lossy()
    );

    // Should find zellij in second directory
    assert!(find_in_path_with_env("zellij", &path_env, path_sep));

    // Cleanup
    std::fs::remove_file(&binary_path).unwrap();
    std::fs::remove_dir(&temp_dir1).unwrap();
    std::fs::remove_dir(&temp_dir2).unwrap();
}
