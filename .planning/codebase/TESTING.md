# Testing

**Analysis Date:** 2026-07-23

## Framework

**Pure `#[test]` with `cargo test`.** No external test framework or mocking library. Tests rely on dependency injection (closures for env lookup, explicit PATH strings) rather than mocking.

## Test Structure

### Unit Tests (inline modules)

Each source module has a corresponding `*_tests.rs` file, loaded via `#[path]` attribute in `main.rs`:

| Source | Test File | Focus |
|--------|-----------|-------|
| `src/main.rs` | `src/main_tests.rs` | Session name resolution, shell escaping, auto-attach resolution |
| `src/config.rs` | `src/config_tests.rs` | TOML parsing, edge cases |
| `src/detect.rs` | `src/detect_tests.rs` | Multiplexer detection priority, PATH probing |
| `src/sanitize.rs` | `src/sanitize_tests.rs` | Session name sanitization, gap-filling |
| `src/tmux.rs` | `src/tmux_tests.rs` | Command building |
| `src/zellij.rs` | `src/zellij_tests.rs` | Command building, socket dir |
| `src/upgrade.rs` | (inline `#[cfg(test)] mod tests`) | Version parsing, prerelease stripping |

**Total unit tests:** ~123 (from `src/main_tests.rs` and inline modules)

### Integration Tests

| File | Focus |
|------|-------|
| `tests/cli.rs` | End-to-end CLI tests: `--version`, `--help`, `--init` output validation |

**Total integration tests:** 5

## Test Patterns

### Dependency Injection

Functions that depend on environment variables accept them as parameters for testability:

```rust
// Production
debug_enabled(&env_map)

// Testable
fn debug_enabled(env: &HashMap<String, String>) -> bool
```

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
| Upgrade (version parsing) | Medium — prerelease suffixes, malformed input |
| CLI integration | Low — only `--version`, `--help`, `--init` |
| Upgrade (CLI flags) | None — `--upgrade`, `--check-version` not tested |
| Network (GitHub API) | None — no mock HTTP server |
| Binary replacement | None — requires filesystem state |
