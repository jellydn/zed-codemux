use crate::{
    debug_enabled, decide_fallback_shell, get_base_name, resolve_auto_attach, resolve_session_name,
    shell_escape, Config,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_resolve_auto_attach_env_true() {
    let mut env = HashMap::new();
    env.insert("CODEMUX_AUTO_ATTACH".to_string(), "true".to_string());
    let config = Config::default();

    assert!(resolve_auto_attach(&env, &config));
}

#[test]
fn test_resolve_auto_attach_env_false() {
    let mut env = HashMap::new();
    env.insert("CODEMUX_AUTO_ATTACH".to_string(), "false".to_string());
    let config = Config {
        multiplexer: None,
        auto_attach: Some(true),
    };

    assert!(!resolve_auto_attach(&env, &config));
}

#[test]
fn test_resolve_auto_attach_env_overrides_config() {
    let mut env = HashMap::new();
    env.insert("CODEMUX_AUTO_ATTACH".to_string(), "false".to_string());
    let config = Config {
        multiplexer: None,
        auto_attach: Some(true),
    };

    // Env should override config
    assert!(!resolve_auto_attach(&env, &config));
}

#[test]
fn test_resolve_auto_attach_config_true() {
    let env = HashMap::new();
    let config = Config {
        multiplexer: None,
        auto_attach: Some(true),
    };

    assert!(resolve_auto_attach(&env, &config));
}

#[test]
fn test_resolve_auto_attach_config_false() {
    let env = HashMap::new();
    let config = Config {
        multiplexer: None,
        auto_attach: Some(false),
    };

    assert!(!resolve_auto_attach(&env, &config));
}

#[test]
fn test_resolve_auto_attach_default_true() {
    let env = HashMap::new();
    let config = Config::default();

    // Default should be true
    assert!(resolve_auto_attach(&env, &config));
}

#[test]
fn test_resolve_auto_attach_case_insensitive() {
    let mut env = HashMap::new();
    env.insert("CODEMUX_AUTO_ATTACH".to_string(), "TRUE".to_string());
    let config = Config::default();

    assert!(resolve_auto_attach(&env, &config));

    let mut env2 = HashMap::new();
    env2.insert("CODEMUX_AUTO_ATTACH".to_string(), "False".to_string());
    assert!(!resolve_auto_attach(&env2, &config));
}

#[test]
fn test_get_base_name_simple() {
    let path = PathBuf::from("/home/user/projects/myapp");
    assert_eq!(get_base_name(&path), "myapp");
}

#[test]
fn test_get_base_name_with_spaces() {
    let path = PathBuf::from("/home/user/My Projects");
    assert_eq!(get_base_name(&path), "My Projects");
}

#[test]
fn test_get_base_name_root() {
    let path = PathBuf::from("/");
    // Root has no file_name, should return "session"
    assert_eq!(get_base_name(&path), "session");
}

#[test]
fn test_decide_fallback_shell_unix_env() {
    let mut env = HashMap::new();
    env.insert("SHELL".to_string(), "/bin/zsh".to_string());

    #[cfg(unix)]
    assert_eq!(decide_fallback_shell(&env), "/bin/zsh");
}

#[test]
fn test_decide_fallback_shell_unix_default() {
    let env: HashMap<String, String> = HashMap::new();

    #[cfg(unix)]
    assert_eq!(decide_fallback_shell(&env), "/bin/sh");
}

#[test]
fn test_decide_fallback_shell_windows_env() {
    let mut env = HashMap::new();
    env.insert("COMSPEC".to_string(), "C:\\Windows\\cmd.exe".to_string());

    #[cfg(windows)]
    assert_eq!(decide_fallback_shell(&env), "C:\\Windows\\cmd.exe");
}

#[test]
fn test_decide_fallback_shell_windows_default() {
    let _env: HashMap<String, String> = HashMap::new();

    #[cfg(windows)]
    assert_eq!(decide_fallback_shell(&_env), "cmd.exe");
}

// Tests for debug_enabled

#[test]
fn test_debug_enabled_when_set_to_1() {
    let mut env = HashMap::new();
    env.insert("CODEMUX_DEBUG".to_string(), "1".to_string());

    assert!(debug_enabled(&env));
}

#[test]
fn test_debug_disabled_when_unset() {
    let env: HashMap<String, String> = HashMap::new();

    assert!(!debug_enabled(&env));
}

#[test]
fn test_debug_disabled_when_set_to_0() {
    let mut env = HashMap::new();
    env.insert("CODEMUX_DEBUG".to_string(), "0".to_string());

    assert!(!debug_enabled(&env));
}

#[test]
fn test_debug_disabled_when_set_to_other_value() {
    let mut env = HashMap::new();
    env.insert("CODEMUX_DEBUG".to_string(), "true".to_string());

    assert!(!debug_enabled(&env));
}

#[test]
fn test_debug_disabled_when_set_to_empty() {
    let mut env = HashMap::new();
    env.insert("CODEMUX_DEBUG".to_string(), "".to_string());

    assert!(!debug_enabled(&env));
}

// Tests for shell_escape

#[test]
fn test_shell_escape_empty_string() {
    assert_eq!(shell_escape(""), "''");
}

#[test]
fn test_shell_escape_simple_string() {
    assert_eq!(shell_escape("foo"), "'foo'");
}

#[test]
fn test_shell_escape_with_single_quote() {
    // "it's" → "'it'\"'\"'s'"
    assert_eq!(shell_escape("it's"), "'it'\"'\"'s'");
}

#[test]
fn test_shell_escape_path_with_spaces() {
    assert_eq!(shell_escape("/path with spaces"), "'/path with spaces'");
}

#[test]
fn test_shell_escape_multiple_single_quotes() {
    // "don't" → "'don'\"'\"'t'"
    assert_eq!(shell_escape("don't"), "'don'\"'\"'t'");
}

#[test]
fn test_shell_escape_only_single_quote() {
    // "'" → "''\"'\"''"
    assert_eq!(shell_escape("'"), "''\"'\"''");
}

#[test]
fn test_shell_escape_special_chars_no_quotes() {
    // Characters like $, `, \, etc. should just be wrapped
    assert_eq!(shell_escape("$HOME"), "'$HOME'");
    assert_eq!(shell_escape("`echo hi`"), "'`echo hi`'");
    assert_eq!(shell_escape("back\\slash"), "'back\\slash'");
}

// Security tests for shell injection prevention

#[test]
fn test_shell_escape_prevents_command_injection() {
    // Attempt to break out of single quotes and execute commands
    // Input: '; rm -rf /; ' should become: ''\'''; rm -rf /; '\'''
    let malicious = "'; rm -rf /; '";
    let escaped = shell_escape(malicious);
    // Verify the escaped string cannot break out of quotes
    assert!(escaped.contains("'\"'\"'"));
    assert_ne!(escaped, "'; rm -rf /; '");
}

#[test]
fn test_shell_escape_prevents_variable_expansion() {
    // $() command substitution
    let malicious = "$(rm -rf /)";
    let escaped = shell_escape(malicious);
    assert_eq!(escaped, "'$(rm -rf /)'");

    // Backtick command substitution
    let malicious2 = "`rm -rf /`";
    let escaped2 = shell_escape(malicious2);
    assert_eq!(escaped2, "'`rm -rf /`'");
}

#[test]
fn test_shell_escape_null_bytes() {
    // Null bytes in session names
    let with_null = "test\x00name";
    let escaped = shell_escape(with_null);
    assert!(escaped.contains("\x00"));
    // The null byte should be preserved in the escaped string
}

#[test]
fn test_shell_escape_control_characters() {
    // Various control characters
    let with_tab = "test\tname";
    let escaped = shell_escape(with_tab);
    assert!(escaped.contains("\t"));

    let with_newline = "test\nname";
    let escaped = shell_escape(with_newline);
    assert!(escaped.contains("\n"));
}

#[test]
fn test_shell_escape_multiple_quotes() {
    // Multiple single quotes in various positions
    assert_eq!(shell_escape("'"), "''\"'\"''");
    assert_eq!(shell_escape("''"), "''\"'\"''\"'\"''");
    assert_eq!(shell_escape("a'b'c"), "'a'\"'\"'b'\"'\"'c'");
}

// --- resolve_session_name tests ---

#[test]
fn test_resolve_session_name_truncated_matches_directly() {
    let sessions = vec!["myapp".to_string()];
    let result = resolve_session_name("myapp", "myapp", &sessions, true);
    assert_eq!(result, "myapp");
}

#[test]
fn test_resolve_session_name_full_matches_when_truncated_does_not() {
    let long_session = "2026-02-26-aircarbon-ac-monorepo2-feat-idx-fcr-008-009";
    let sessions = vec![long_session.to_string()];
    let base_name = "2026-02-26-aircarbon-ac-monorepo";
    let result = resolve_session_name(base_name, long_session, &sessions, true);
    assert_eq!(result, long_session);
}

#[test]
fn test_resolve_session_name_full_match_not_in_list() {
    let long_session = "2026-02-26-aircarbon-ac-monorepo2-feat-idx-fcr-008-009";
    let sessions: Vec<String> = vec![];
    let base_name = "2026-02-26-aircarbon-ac-monorepo";
    let result = resolve_session_name(base_name, long_session, &sessions, true);
    // Falls through to get_unique_session_name
    assert_eq!(result, base_name);
}

#[test]
fn test_resolve_session_name_truncated_match_takes_priority_over_full() {
    let long_session = "2026-02-26-aircarbon-ac-monorepo2-feat-idx-fcr-008-009";
    let short_session = "2026-02-26-aircarbon-ac-monorepo";
    let sessions = vec![short_session.to_string(), long_session.to_string()];
    let base_name = "2026-02-26-aircarbon-ac-monorepo";
    let result = resolve_session_name(base_name, long_session, &sessions, true);
    // Truncated match takes priority over full match
    assert_eq!(result, short_session);
}

#[test]
fn test_resolve_session_name_auto_attach_false_skips_matching() {
    let sessions = vec!["myapp".to_string()];
    // Even though 'myapp' exists, auto_attach=false means always unique
    let result = resolve_session_name("myapp", "myapp", &sessions, false);
    assert_eq!(result, "myapp-2");
}

#[test]
fn test_resolve_session_name_no_auto_attach_uses_unique_name() {
    let sessions: Vec<String> = vec![];
    let result = resolve_session_name("myapp", "myapp", &sessions, false);
    assert_eq!(result, "myapp");
}

#[test]
fn test_resolve_session_name_gap_filling_in_no_attach_mode() {
    let sessions = vec![
        "myapp".to_string(),
        "myapp-2".to_string(),
        "myapp-3".to_string(),
        "myapp-5".to_string(),
    ];
    // In non-auto-attach mode, always generate unique via gap-filling
    let result = resolve_session_name("myapp", "myapp", &sessions, false);
    assert_eq!(result, "myapp-4");
}

#[test]
fn test_resolve_session_name_auto_attach_returns_existing() {
    // In auto-attach mode, if the session exists, return it directly
    let sessions = vec!["myapp".to_string()];
    let result = resolve_session_name("myapp", "myapp", &sessions, true);
    assert_eq!(result, "myapp");
}
