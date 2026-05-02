/// Sanitizes a workspace name into a tmux/zellij-safe session name.
/// Matches the exact vscode-mux algorithm:
/// - Replace any char not in [a-zA-Z0-9-] with '-'
/// - Collapse consecutive '-' into one
/// - Strip leading/trailing '-'
/// - Return 'session' if result is empty
#[inline]
pub fn sanitize_session_name(name: &str) -> String {
    // Step 1: Replace any character not in [a-zA-Z0-9-] with '-'
    let mut result: Vec<char> = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();

    // Step 2: Collapse consecutive '-' into single '-'
    let mut collapsed = Vec::with_capacity(result.len());
    let mut prev_was_dash = false;
    for c in result {
        if c == '-' {
            if !prev_was_dash {
                collapsed.push(c);
            }
            prev_was_dash = true;
        } else {
            collapsed.push(c);
            prev_was_dash = false;
        }
    }
    result = collapsed;

    // Step 3 & 4: Strip leading and trailing '-'
    while result.first() == Some(&'-') {
        result.remove(0);
    }
    while result.last() == Some(&'-') {
        result.pop();
    }

    // Step 5: Return 'session' if result is empty
    if result.is_empty() {
        "session".to_string()
    } else {
        result.into_iter().collect()
    }
}

/// Computes a unique session name with gap-filling (matches vscode-mux exactly).
/// If base is not in sessions, returns base unchanged.
/// Otherwise starts at suffix=2 and finds the first available gap.
/// Example: sessions=['myapp','myapp-2','myapp-5'] → returns 'myapp-3'
#[inline]
pub fn get_unique_session_name(base: &str, sessions: &[String]) -> String {
    // Linear search is faster than HashSet for small lists (typical case < 10 sessions)
    let contains = |s: &str| sessions.iter().any(|session| session == s);

    if !contains(base) {
        return base.to_string();
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{base}-{suffix}");
        if !contains(&candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Very long names should still work
        let long_name = "a".repeat(1000);
        let result = sanitize_session_name(&long_name);
        assert_eq!(result.len(), 1000);
        assert_eq!(result, long_name);
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
            sanitize_session_name("..\\\\..\\\\windows\\\\system32"),
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
}
