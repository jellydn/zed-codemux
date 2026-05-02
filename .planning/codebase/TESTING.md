# Testing Patterns

**Analysis Date:** 2026-05-02

## Test Framework

**Runner:**
- Built-in Rust test framework (`cargo test`)
- Config: Inline in source files under `#[cfg(test)]`

**Assertion Library:**
- Standard `assert!`, `assert_eq!`, `assert_ne!` macros
- No external assertion library needed

**Run Commands:**
```bash
cargo test              # Run all tests
cargo test test_name    # Run specific test by name pattern
cargo test -- --nocapture  # Show println! output
```

## Test File Organization

**Location:**
- Tests are co-located in source files
- Each `.rs` file has a `#[cfg(test)]` module at the bottom

**Naming:**
- Test functions: `test_<description>`
- Pattern: descriptive snake_case names

**Structure:**
```
src/
├── main.rs          # Has #[cfg(test)] module with tests
├── config.rs        # Has #[cfg(test)] module with tests
├── detect.rs        # Has #[cfg(test)] module with tests
├── sanitize.rs      # Has #[cfg(test)] module with tests
├── shell_escape.rs  # Has #[cfg(test)] module with tests
├── tmux.rs          # Has #[cfg(test)] module with tests
└── zellij.rs        # Has #[cfg(test)] module with tests
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_1() {
        // Arrange
        let input = ...;
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    fn test_case_2() {
        // ...
    }
}
```

**Patterns:**
- **Arrange-Act-Assert**: Clear separation in each test
- **Table-driven**: Multiple similar tests with descriptive names
- **Edge cases**: Empty strings, special characters, boundary conditions

## Mocking

**Framework:** None - manual dependency injection

**Patterns:**
- Environment injection via `HashMap<String, String>` parameter
- No external process mocking (tests check behavior when commands not found)
- Regex patterns are compiled in functions but tested via integration

**Example from `src/detect.rs`:**
```rust
pub fn detect_multiplexer_with_env(
    config: &Config,
    env: &std::collections::HashMap<String, String>,
) -> Option<Multiplexer> {
    // Testable version that accepts env as parameter
}
```

**What to Mock:**
- Environment variables (via injection)
- Config values (via struct construction)

**What NOT to Mock:**
- External commands (tmux, zellij) - tests accept that NotFound returns empty list

## Fixtures and Factories

**Test Data:**
- Inline literals in tests
- No external fixture files
- Example: `let toml = r#"multiplexer = "tmux""#;`

**Common Patterns:**
- `Config::default()` for default config
- `HashMap::new()` for empty environment
- `PathBuf::from("/path")` for path testing

## Coverage

**Requirements:** No explicit coverage target

**View Coverage:**
```bash
# Install tarpaulin (not currently in project)
cargo install cargo-tarpaulin
cargo tarpaulin
```

**Notable Coverage Areas:**
- Config parsing (valid, invalid, partial)
- Multiplexer detection (env, config, PATH priority)
- Session name sanitization (various edge cases)
- Auto_attach resolution (env overrides config overrides default)
- Shell detection (Unix vs Windows)
- Debug mode detection

## Test Types

**Unit Tests:**
- Scope: Individual functions in isolation
- Location: Inline with source code
- Approach: Pure functions with injected dependencies

**Integration Tests:**
- Limited - no `tests/` directory
- Implicit integration via command building tests

**E2E Tests:**
- Not used - would require tmux/zellij installed
- Some tests check behavior when multiplexer not installed

## Common Patterns

**Async Testing:**
- Not applicable - codebase is synchronous

**Error Testing:**
```rust
#[test]
fn test_parse_invalid_toml() {
    // Invalid TOML should return defaults, not panic
    let config = parse_config_str("not valid toml [ broken");
    assert_eq!(config.multiplexer, None);
    assert_eq!(config.auto_attach, None);
}
```

**Environment Testing:**
```rust
#[test]
fn test_env_overrides_config() {
    let mut env = HashMap::new();
    env.insert("CODEMUX_MULTIPLEXER".to_string(), "tmux".to_string());
    let config = Config {
        multiplexer: Some("zellij".to_string()),
        auto_attach: None,
    };

    let result = detect_multiplexer_with_env(&config, &env);
    assert_eq!(result, Some(Multiplexer::Tmux));
}
```

**Edge Case Testing:**
```rust
#[test]
fn test_all_invalid_chars() {
    assert_eq!(sanitize_session_name("..."), "session");
}

#[test]
fn test_empty_string() {
    assert_eq!(sanitize_session_name(""), "session");
}
```

---

*Testing analysis: 2026-05-02*
