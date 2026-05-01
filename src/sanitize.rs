use regex::Regex;

/// Sanitizes a workspace name into a tmux/zellij-safe session name.
/// Matches the exact vscode-mux algorithm:
/// - Replace any char not in [a-zA-Z0-9-] with '-'
/// - Collapse consecutive '-' into one
/// - Strip leading/trailing '-'
/// - Return 'session' if result is empty
#[allow(dead_code)]
pub fn sanitize_session_name(name: &str) -> String {
    // Step 1: Replace any character not in [a-zA-Z0-9-] with '-'
    let invalid_char_re = Regex::new(r"[^a-zA-Z0-9-]").unwrap();
    let replaced = invalid_char_re.replace_all(name, "-");

    // Step 2: Collapse consecutive '-' into single '-'
    let collapse_re = Regex::new(r"-{2,}").unwrap();
    let collapsed = collapse_re.replace_all(&replaced, "-");

    // Step 3: Strip leading '-'
    let leading_re = Regex::new(r"^-+").unwrap();
    let no_leading = leading_re.replace_all(&collapsed, "");

    // Step 4: Strip trailing '-'
    let trailing_re = Regex::new(r"-+$").unwrap();
    let result = trailing_re.replace_all(&no_leading, "");

    // Step 5: Return 'session' if result is empty
    if result.is_empty() {
        "session".to_string()
    } else {
        result.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
