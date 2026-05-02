# Testing Patterns

**Analysis Date:** 2026-05-02

## Test Framework

**Runner:**
- Built-in Rust testing (`cargo test`) - Rust 2021 edition
- Config: `Cargo.toml` with `[dev-dependencies]` section

**Assertion Library:**
- Standard Rust assertions (`assert!`, `assert_eq!`, `assert_ne!`)
- No external assertion libraries used

**Run Commands:**
```bash
cargo test              # Run all tests
just test              # Via justfile recipe
cargo test <filter>    # Run specific test by name
```

## Test File Organization

**Location:**
- Co-located with source code using `#[cfg(test)]` modules
- Each source file has its own inline test module at the bottom
- No separate `tests/` directory for integration tests

**Naming:**
- Test functions named `test_<description>` in snake_case
- Descriptive names indicating what's being tested

**Structure:**
```
src/
├── main.rs           (contains tests for main module)
├── config.rs         (contains tests for config module)
├── detect.rs         (contains tests for detect module)
├── sanitize.rs       (contains tests for sanitize module)
├── tmux.rs           (contains tests for tmux module)
└── zellij.rs         (contains tests for zellij module)
```

## Test Structure

**Suite Organization:**
Each module has a `#[cfg(test)]` module at the end:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    // additional imports for testing

    #[test]
    fn test_specific_behavior() {
        // Arrange
        let input = ...;
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

**Patterns:**
- **Setup pattern:** Direct variable creation in each test (no shared setup functions)
- **Teardown pattern:** No explicit teardown needed (Rust's ownership handles cleanup)
- **Assertion pattern:** Direct assertions with `assert_eq!`, `assert!`, `assert_ne!`

**Example from `src/config.rs` (lines 116-265):**
```rust
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
    // ... more tests
}
```

## Mocking

**Framework:** No external mocking framework used

**Patterns:**
- Dependency injection via closures (e.g., `env_lookup` parameter in `detect_with_env_lookup`)
- HashMap-based environment simulation for testing
- Process execution not mocked - tests check behavior when commands not found

**Example from `src/detect.rs` (lines 106-111):**
```rust
/// Detects which multiplexer to use, with explicit environment variable injection.
fn detect_multiplexer_with_env(
    config: &Config,
    env: &HashMap<String, String>,
) -> Option<Multiplexer> {
    detect_with_env_lookup(config, |name| env.get(name).cloned())
}
```

**What to Mock:**
- Environment variables via HashMap injection
- Configuration state via struct construction

**What NOT to Mock:**
- External process execution (tmux/zellij commands) - tests check graceful handling when commands unavailable
- File system operations (actual temp files used in dev-dependencies)

## Fixtures and Factories

**Test Data:**
- Inline test data defined in each test
- Raw string literals for TOML content (`r#"..."#`)
- Direct struct construction with field initialization

**Example pattern:**
```rust
let mut env = HashMap::new();
env.insert("CODEMUX_MULTIPLEXER".to_string(), "tmux".to_string());
let config = Config::default();
```

**Location:** Test data is defined inline within test functions

**Dev Dependencies:**
```toml
[dev-dependencies]
tempfile = "3.10"   # For temporary file operations in tests
```

## Coverage

**Requirements:** None enforced (no coverage tool configured)

**View Coverage:**
```bash
# Install cargo-tarpaulin for coverage
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

**Coverage approach:**
- Comprehensive unit tests for all public functions
- Edge case testing (empty strings, special characters, null bytes)
- Security-focused tests for shell injection prevention
- Platform-specific code tested with conditional compilation (`#[cfg(unix)]`, `#[cfg(windows)]`)

## Test Types

**Unit Tests:**
- Scope: All functions tested in isolation
- Approach: Input/output verification with edge cases
- Location: Inline `#[cfg(test)]` modules in each source file

**Integration Tests:**
- Not explicitly used - all tests are unit-style
- Some tests verify integration between components (e.g., env var → config resolution)

**E2E Tests:**
- Not used - no end-to-end process spawning tests
- Tests for external commands (`list_sessions`) verify graceful handling when commands unavailable

## Common Patterns

**Async Testing:**
- Not applicable - codebase is synchronous
- No async/await patterns used

**Error Testing:**
```rust
#[test]
fn test_list_sessions_when_tmux_not_installed() {
    let launcher = TmuxLauncher::new();
    let result = launcher.list_sessions();
    assert!(result.is_ok());  // Should return empty list, not error
}
```

**Platform-specific Testing:**
```rust
#[test]
fn test_decide_fallback_shell_unix_env() {
    let mut env = HashMap::new();
    env.insert("SHELL".to_string(), "/bin/zsh".to_string());

    #[cfg(unix)]
    assert_eq!(decide_fallback_shell(&env), "/bin/zsh");
}
```

**Security Testing:**
```rust
#[test]
fn test_shell_escape_prevents_command_injection() {
    let malicious = "'; rm -rf /; '";
    let escaped = shell_escape(malicious);
    assert!(escaped.contains("'\"'\"'"));
    assert_ne!(escaped, "'; rm -rf /; '");
}
```

**Edge Case Testing:**
- Empty strings
- Null bytes in input
- Unicode characters
- Control characters
- Path traversal attempts
- Very long strings (1000+ characters)

## CI/CD Testing

**GitHub Actions workflow (`.github/workflows/ci.yml`):**
- Tests run on matrix: ubuntu-latest, macos-latest, windows-latest
- Commands:
  - `cargo test` (line 35)
  - `cargo clippy -- -D warnings` (line 38)
  - `cargo fmt --check` (line 41)

**Pre-commit hooks (`prek.toml`):**
- `cargo test` runs as pre-commit hook
- Tests must pass before commits are accepted

---

*Testing analysis: 2026-05-02*
