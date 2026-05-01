/// POSIX shell escape: wraps input in single quotes, replacing internal `'` with `'"'"'`.
/// If input is empty, returns `''`.
#[allow(dead_code)]
pub fn shell_escape(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        assert_eq!(shell_escape(""), "''");
    }

    #[test]
    fn test_simple_string() {
        assert_eq!(shell_escape("foo"), "'foo'");
    }

    #[test]
    fn test_with_single_quote() {
        // "it's" → "'it'\"'\"'s'"
        assert_eq!(shell_escape("it's"), "'it'\"'\"'s'");
    }

    #[test]
    fn test_path_with_spaces() {
        assert_eq!(shell_escape("/path with spaces"), "'/path with spaces'");
    }

    #[test]
    fn test_multiple_single_quotes() {
        // "don't" → "'don'\"'\"'t'"
        assert_eq!(shell_escape("don't"), "'don'\"'\"'t'");
    }

    #[test]
    fn test_only_single_quote() {
        // "'" → "''\"'\"''"
        assert_eq!(shell_escape("'"), "''\"'\"''");
    }

    #[test]
    fn test_special_chars_no_quotes() {
        // Characters like $, `, \, etc. should just be wrapped
        assert_eq!(shell_escape("$HOME"), "'$HOME'");
        assert_eq!(shell_escape("`echo hi`"), "'`echo hi`'");
        assert_eq!(shell_escape("back\\slash"), "'back\\slash'");
    }
}
