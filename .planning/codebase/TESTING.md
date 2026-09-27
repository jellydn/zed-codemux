# Testing

**Analysis Date:** 2026-07-23

## Framework

**Pure `#[test]` with `cargo test`.** No external test framework or mocking library. Tests rely on dependency injection (closures for env lookup, explicit PATH strings) rather than mocking.

## Test Structure

### Unit Tests (inline modules)

Most source modules have a corresponding `*_tests.rs` file, loaded via `#[path]` attribute in `main.rs`; `upgrade.rs` keeps its unit tests inline:

| Source | Test File | Focus |
|--------|-----------|-------|
| `src/main.rs` | `src/main_tests.rs` | Session name resolution, shell escaping, auto-attach resolution |
| `src/config.rs` | `src/config_tests.rs` | TOML parsing, edge cases |
| `src/detect.rs` | `src/detect_tests.rs` | Multiplexer detection priority, PATH probing |
| `src/sanitize.rs` | `src/sanitize_tests.rs` | Session name sanitization, gap-filling |
| `src/tmux.rs` | `src/tmux_tests.rs` | Command building |
| `src/zellij.rs` | `src/zellij_tests.rs` | Command building, socket dir |
| `src/upgrade.rs` | (inline `#[cfg(test)] mod tests`) | Version parsing and SemVer precedence |

**Total unit tests:** 126

### Integration Tests

| File | Focus |
|------|-------|
| `tests/cli.rs` | End-to-end CLI, process replacement, and upgrade-option tests |

**Total integration tests:** 14

## Test Patterns

### Dependency Injection

Multiplexer detection accepts an environment lookup closure for testability:

```rust
// Production (detect module)
detect_with_env_lookup(&config, |name| std::env::var(name).ok())

// Testable
pub(crate) fn detect_with_env_lookup(
    config: &Config,
    env_lookup: impl Fn(&str) -> Option<String>,
) -> Option<Multiplexer>
```

### Inline Test Modules (upgrade.rs)

The upgrade module uses an inline `#[cfg(test)] mod tests` block rather than a separate file, following the convention of keeping tests close to the code they test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_strips_prerelease() {
        assert_eq!(parse_version("v1.2.3-rc1"), Some((1, 2, 3)));
    }
}
```

## Test Commands

```bash
cargo test                     # Run all tests
cargo test test_sanitize       # Run a single test by name pattern
cargo test -- --nocapture       # Show stdout/stderr during tests
```

## Test Coverage

| Area | Coverage |
|------|----------|
| Session naming (sanitize) | High — edge cases: empty, special chars, long names, collisions |
| Config parsing | High — valid/invalid TOML, edge cases |
| Multiplexer detection | High — env, config, PATH fallback, missing multiplexer |
| Shell escaping | High — quotes, empty strings, special characters |
| Command building | Medium — template verification |
| Upgrade (version parsing) | Medium — prerelease precedence, build metadata, malformed input |
| CLI integration | Medium — process replacement and built-in options |
| Upgrade (CLI flags) | Medium — mocked curl and package-manager commands |
| Network (GitHub API) | Low — curl is mocked; no HTTP server |
| Binary replacement | None — requires filesystem state |
