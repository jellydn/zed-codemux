use crate::config::{parse_config_str, Config, DEFAULT_CONFIG_CONTENT};

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

#[test]
fn test_create_default_config_content() {
    // Verify the default config content is valid and parseable
    let config = parse_config_str(DEFAULT_CONFIG_CONTENT);
    assert_eq!(config.multiplexer, Some("tmux".to_string()));
    assert_eq!(config.auto_attach, Some(true));
}

#[test]
fn test_create_default_config_content_includes_comments() {
    // Verify comments are preserved in the default config
    assert!(DEFAULT_CONFIG_CONTENT.contains("# CodeMux configuration file"));
    assert!(DEFAULT_CONFIG_CONTENT.contains("# Preferred multiplexer"));
    assert!(DEFAULT_CONFIG_CONTENT.contains("# Whether to auto-attach"));
}
