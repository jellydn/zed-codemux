use crate::sanitize::{get_unique_session_name, sanitize_session_name};

#[test]
fn test_unique_empty_list() {
    assert_eq!(get_unique_session_name("myapp", &[]), "myapp");
}

#[test]
fn test_unique_base_not_in_list() {
    assert_eq!(
        get_unique_session_name("myapp", &["other".to_string()]),
        "myapp"
    );
}

#[test]
fn test_unique_base_exists() {
    assert_eq!(
        get_unique_session_name("myapp", &["myapp".to_string()]),
        "myapp-2"
    );
}

#[test]
fn test_unique_base_and_2_exist() {
    assert_eq!(
        get_unique_session_name("myapp", &["myapp".to_string(), "myapp-2".to_string()]),
        "myapp-3"
    );
}

#[test]
fn test_unique_gap_filling() {
    // Gap-filling: finds myapp-3 before myapp-5
    assert_eq!(
        get_unique_session_name(
            "myapp",
            &[
                "myapp".to_string(),
                "myapp-2".to_string(),
                "myapp-5".to_string()
            ]
        ),
        "myapp-3"
    );
}

#[test]
fn test_unique_large_gap() {
    assert_eq!(
        get_unique_session_name(
            "myapp",
            &[
                "myapp".to_string(),
                "myapp-2".to_string(),
                "myapp-3".to_string(),
                "myapp-5".to_string()
            ]
        ),
        "myapp-4"
    );
}

#[test]
fn test_my_workspace() {
    assert_eq!(sanitize_session_name("My Workspace"), "My-Workspace");
}

#[test]
fn test_my_project() {
    assert_eq!(sanitize_session_name("my.project"), "my-project");
}

#[test]
fn test_my_underscore_project() {
    assert_eq!(sanitize_session_name("my_project"), "my-project");
}

#[test]
fn test_consecutive_dots() {
    assert_eq!(sanitize_session_name("my...project"), "my-project");
}

#[test]
fn test_leading_trailing_dashes() {
    assert_eq!(sanitize_session_name("-myproject-"), "myproject");
}

#[test]
fn test_all_invalid_chars() {
    assert_eq!(sanitize_session_name("..."), "session");
}

#[test]
fn test_empty_string() {
    assert_eq!(sanitize_session_name(""), "session");
}

#[test]
fn test_whitespace() {
    assert_eq!(sanitize_session_name("   "), "session");
}

#[test]
fn test_valid_name_unchanged() {
    assert_eq!(sanitize_session_name("my-project-123"), "my-project-123");
}

#[test]
fn test_multiple_spaces() {
    assert_eq!(sanitize_session_name("My   Project"), "My-Project");
}

#[test]
fn test_mixed_special_chars() {
    assert_eq!(sanitize_session_name("my@project#test$"), "my-project-test");
}

// Security and edge case tests

#[test]
fn test_null_bytes() {
    // Null bytes should be sanitized (though unlikely in practice)
    let result = sanitize_session_name("test\x00name");
    assert!(!result.contains('\x00'));
}

#[test]
fn test_control_characters() {
    // Control characters should be replaced
    assert_eq!(sanitize_session_name("test\x01name"), "test-name");
    assert_eq!(sanitize_session_name("test\x1fname"), "test-name");
    assert_eq!(sanitize_session_name("test\x7fname"), "test-name");
}

#[test]
fn test_very_long_name() {
    // Very long names should be truncated to MAX_SESSION_NAME_LENGTH
    let long_name = "a".repeat(1000);
    let result = sanitize_session_name(&long_name);
    assert_eq!(result.len(), 32);
    assert_eq!(result, "a".repeat(32));
}

#[test]
fn test_unicode_characters() {
    // Unicode characters are sanitized (replaced with '-', then collapsed)
    // Multiple unicode chars collapse to single dash, then may be stripped
    let result = sanitize_session_name("项目");
    assert!(result == "session" || result == "-" || result.is_empty());
    // "🚀rocket" → "rocket" after sanitizing emoji (leading dash stripped)
    assert_eq!(sanitize_session_name("🚀rocket"), "rocket");
    // "café" → "caf" where é is replaced with dash, then trailing dash stripped
    assert_eq!(sanitize_session_name("café"), "caf");
}

#[test]
fn test_path_traversal_attempts() {
    // Path traversal patterns should be sanitized (dots replaced with dashes)
    // Leading dashes are stripped
    assert_eq!(sanitize_session_name("../../../etc/passwd"), "etc-passwd");
    assert_eq!(
        sanitize_session_name("..\\..\\windows\\system32"),
        "windows-system32"
    );
}

#[test]
fn test_shell_metacharacters() {
    // Shell metacharacters should be sanitized (replaced with '-')
    // Leading/trailing dashes are stripped, consecutive dashes collapsed
    assert_eq!(sanitize_session_name("test;rm -rf /"), "test-rm-rf");
    assert_eq!(
        sanitize_session_name("test|cat /etc/passwd"),
        "test-cat-etc-passwd"
    );
    assert_eq!(sanitize_session_name("test&&whoami"), "test-whoami");
    assert_eq!(sanitize_session_name("test||reboot"), "test-reboot");
}

#[test]
fn test_only_special_chars() {
    // Names with only special characters should become "session"
    assert_eq!(sanitize_session_name("!@#$%^&*()"), "session");
    assert_eq!(sanitize_session_name("<>[]{}"), "session");
}

#[test]
fn test_trailing_numbers() {
    // Ensure we don't accidentally match the suffix pattern
    let result = get_unique_session_name("myapp-2", &["myapp-2".to_string()]);
    assert_eq!(result, "myapp-2-2");
}

#[test]
fn test_name_collisions_with_suffix() {
    // Test collision with names that look like our suffixes
    let sessions = vec!["myapp".to_string(), "myapp-2".to_string()];
    assert_eq!(get_unique_session_name("myapp-2", &sessions), "myapp-2-2");
}

#[test]
fn test_truncate_long_name() {
    // Long names should be truncated to MAX_SESSION_NAME_LENGTH (32)
    let long_name = "a".repeat(50);
    let result = sanitize_session_name(&long_name);
    assert_eq!(result.len(), 32);
    assert_eq!(result, "a".repeat(32));
}

#[test]
fn test_truncate_preserves_beginning() {
    // Truncation should preserve the beginning of the name
    let name = "my-very-long-project-name-that-exceeds-limits";
    let result = sanitize_session_name(name);
    assert!(result.len() <= 32);
    assert!(result.starts_with("my-very-long-project-name"));
}

#[test]
fn test_truncate_after_sanitization() {
    // Special chars should be sanitized first, then truncation applied
    let name = "my.project.name.with.many.dots.that.is.very.long";
    let result = sanitize_session_name(name);
    assert!(result.len() <= 32);
    assert_eq!(result, "my-project-name-with-many-dots-t");
}

#[test]
fn test_exactly_max_length_unchanged() {
    // Names exactly at the limit should not be truncated
    let name = "a".repeat(32);
    let result = sanitize_session_name(&name);
    assert_eq!(result, name);
    assert_eq!(result.len(), 32);
}

#[test]
fn test_one_over_max_length_truncated() {
    // Names one char over the limit should be truncated
    let name = "a".repeat(33);
    let result = sanitize_session_name(&name);
    assert_eq!(result.len(), 32);
    assert_eq!(result, "a".repeat(32));
}
