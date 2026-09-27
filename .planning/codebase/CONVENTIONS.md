# Coding Conventions

**Analysis Date:** 2026-07-23

## Naming Patterns

**Files:**
- Snake_case for all source files (`config.rs`, `sanitize.rs`, `tmux.rs`, `zellij.rs`, `upgrade.rs`)
- Test files use `<module>_tests.rs` suffix (`config_tests.rs`, `main_tests.rs`)

**Functions:**
- Snake_case for all function names (`load_config`, `sanitize_session_name`, `detect_multiplexer`, `shell_escape`)
- Private helpers prefixed with descriptive verbs (`decide_fallback_shell`, `debug_enabled`, `parse_args`, `handle_external_upgrade`)
- Public API functions have doc comments (`///`) explaining purpose and behavior

**Variables:**
- Snake_case (`config`, `auto_attach`, `sessions`, `sanitized_name`)
- Short, descriptive names for iterators (`c` for characters, `s` for strings)

**Types:**
- PascalCase for structs, enums, traits (`Config`, `TmuxLauncher`, `ZellijLauncher`, `Multiplexer`, `MuxLauncher`, `UpgradeResult`, `UpgradeError`)
- Enum variants use PascalCase (`Multiplexer::Tmux`, `Multiplexer::Zellij`, `InstallMethod::Cargo`)
- The upgrade module defines a custom `UpgradeError` enum (departs from the `io::Error` pattern used elsewhere — intentional, as it provides richer error categorization)

**Constants:**
- `SCREAMING_SNAKE_CASE` (`VERSION`, `MAX_SESSION_NAME_LENGTH`, `DEFAULT_CONFIG_CONTENT`)

## Code Style

**Formatting:**
- Tool: `cargo fmt` (standard Rust formatter, no custom `rustfmt.toml`)
- CI: `cargo fmt --check`, pre-commit: `cargo fmt -- --check`

**Linting:**
- Tool: `cargo clippy` with `-D warnings` (warnings-as-errors)
- CI and pre-commit both enforce this

**Release Profile:**
- Size-optimized: `opt-level = "s"`, LTO, stripped, `panic = "abort"`, single codegen unit

## Import Organization

**Order:**
1. Standard library imports (`std::...`) first
2. Crate-level imports (`use crate::...`) second
3. Alphabetical within groups

**Module declarations** at top of `main.rs` (alphabetical):
```rust
mod config;
mod detect;
mod sanitize;
mod tmux;
mod upgrade;
mod zellij;
```

## Error Handling

**Primary patterns:**
- `io::Error` / `io::Result<T>` for most modules (config, detect, tmux, zellij)
- Custom `UpgradeError` enum in `src/upgrade.rs` for richer error categorization
- `From<io::Error> for UpgradeError` for `?` operator compatibility
- Graceful degradation: returns sensible defaults when config/command fails, rather than panicking
- `#[cfg]`-gated error paths for platform differences

**Example (graceful degradation):**
```rust
Err(e) if e.kind() == ErrorKind::NotFound => Ok(Vec::new())
```

## Debug Logging

- Controlled by `CODEMUX_DEBUG=1` environment variable
- All debug output via `eprintln!` with `[codemux]` prefix
- Used in `main.rs`, `zellij.rs`, and `upgrade.rs`
- Private `debug_enabled()` helper in each module that uses it

## Doc Comments

- `///` for all public items (structs, enums, functions, traits)
- Field-level doc comments for struct fields (`/// The version before the upgrade.`)
- Inline `//` comments for complex algorithms (sanitization steps, detection priority)
- Platform-specific comments on `#[cfg]` blocks

## Function Design

- Functions are small and focused (10-30 lines typical; `upgrade()` was refactored to meet this)
- `&str` preferred over `String` for inputs
- References used to avoid cloning
- `io::Result<T>` or custom `Result` types for fallible operations
- Environment injection pattern (`detect_with_env_lookup`, `debug_enabled`) for testability

## Zero-Dependency Philosophy

The root crate has zero external runtime dependencies — pure `std::fmt`, `std::io`, `std::path`, `std::process`, `std::collections`. Only dev-dependency is `tempfile` for integration tests. Manual JSON parsing in `upgrade.rs` avoids pulling in `serde`.
