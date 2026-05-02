# Coding Conventions

**Analysis Date:** 2026-05-02

## Naming Patterns

**Files:**
- Snake_case for all source files (e.g., `config.rs`, `sanitize.rs`, `tmux.rs`, `zellij.rs`, `detect.rs`)

**Functions:**
- Snake_case for all function names (e.g., `load_config`, `sanitize_session_name`, `detect_multiplexer`, `shell_escape`)
- Private helper functions prefixed with descriptive verbs (e.g., `decide_fallback_shell`, `debug_enabled`, `parse_args`)

**Variables:**
- Snake_case for all variables (e.g., `config`, `auto_attach`, `sessions`, `sanitized_name`)
- Short, descriptive names for loop variables (e.g., `c` for characters, `s` for strings)

**Types:**
- PascalCase for all types (structs, enums, traits) (e.g., `Config`, `TmuxLauncher`, `ZellijLauncher`, `Multiplexer`, `MuxLauncher`)
- Enum variants use PascalCase (e.g., `Multiplexer::Tmux`, `Multiplexer::Zellij`)

**Constants:**
- SCREAMING_SNAKE_CASE for constants (e.g., `VERSION`)

## Code Style

**Formatting:**
- Tool: `cargo fmt` (standard Rust formatter)
- Configuration: No custom `rustfmt.toml` - uses default Rust formatting
- CI check: `cargo fmt --check` (enforced in `.github/workflows/ci.yml` line 41)
- Pre-commit hook: `cargo fmt -- --check` (enforced in `prek.toml` line 8)

**Linting:**
- Tool: `cargo clippy` (standard Rust linter)
- Strict mode: `-D warnings` treats all warnings as errors
- CI check: `cargo clippy -- -D warnings` (`.github/workflows/ci.yml` line 38)
- Pre-commit hook: `cargo clippy -- -D warnings` (`prek.toml` line 9)

**Release Profile:**
- Optimized for size: `opt-level = "s"` (`Cargo.toml` line 29)
- LTO enabled: `lto = true` (line 30)
- Binary stripped: `strip = true` (line 31)
- Single codegen unit: `codegen-units = 1` (line 33)

## Import Organization

**Order:**
1. Standard library imports (`std::...`) first
2. Crate-level imports (`use crate::...`) second
3. External crate imports (if any) would come third

**Pattern observed in `src/main.rs`:**
```rust
// Standard library
use std::collections::HashMap;
use std::io::Error;

// Crate modules
use crate::config::{load_config, Config};
use crate::detect::{detect_multiplexer, Multiplexer};
```

**Alphabetical organization:**
- Imports are generally sorted alphabetically within groups
- Module declarations at top of file: `mod config; mod detect; mod sanitize; mod tmux; mod zellij;`

## Error Handling

**Patterns:**
- Uses `std::io::Error` and `io::Result<T>` as the primary error types
- Custom error messages via `io::Error::new(ErrorKind::Other, "message")`
- Graceful degradation: returns sensible defaults when config loading fails
- Error propagation using `?` operator throughout

**Example from `src/tmux.rs` (lines 43-54):**
```rust
Err(e) => {
    if e.kind() == ErrorKind::NotFound {
        Ok(Vec::new())  // Graceful: return empty list if tmux not installed
    } else {
        Err(Error::new(
            ErrorKind::Other,
            format!("Failed to run tmux: {}", e),
        ))
    }
}
```

**Platform-specific error handling:**
- Uses `#[cfg(unix)]`, `#[cfg(windows)]`, `#[cfg(not(any(unix, windows)))]` for platform differences
- Each platform has its own `exec_command` implementation with appropriate error handling

## Logging

**Framework:** Standard error output (`eprintln!`)

**Patterns:**
- Debug mode controlled via `CODEMUX_DEBUG=1` environment variable
- All debug output prefixed with `[codemux]` for identification
- Debug messages include context (resolved multiplexer, session names, commands)
- User-facing errors go to stderr with clear messaging

**Example from `src/main.rs` (lines 144-149):**
```rust
if debug {
    eprintln!("[codemux] Resolved multiplexer: {:?}", multiplexer);
    eprintln!("[codemux] Base name: {}", base_name);
    eprintln!("[codemux] Sanitized name: {}", sanitized_name);
    eprintln!("[codemux] Auto attach: {}", auto_attach);
}
```

## Comments

**When to Comment:**
- Public functions, structs, and traits have doc comments explaining purpose and behavior
- Complex algorithms have inline comments explaining the logic
- Platform-specific code has explanatory comments
- Security-critical code (shell escaping) has detailed comments

**Doc Comment Style (///):**
```rust
/// POSIX shell escape: wraps input in single quotes, replacing internal `'` with `'"'"'`.
/// If input is empty, returns `''`.
#[inline]
pub fn shell_escape(value: &str) -> String {
```

**Module-level documentation:**
- `src/config.rs` line 3: Full description of config file location and purpose
- Each public function has doc comments explaining parameters and return values

**Inline comments:**
- Step-by-step comments for multi-step algorithms (see `sanitize_session_name`)
- Priority order comments for configuration resolution
- Security notes for shell escaping functionality

## Function Design

**Size:** Functions are small and focused (typically 10-30 lines)

**Parameters:**
- Descriptive parameter names
- String slices (`&str`) preferred over owned Strings for inputs
- References used where possible to avoid cloning
- Environment injection pattern for testability (e.g., `detect_with_env_lookup`)

**Return Values:**
- `io::Result<T>` for fallible operations
- Option types for nullable returns
- Direct returns without unnecessary variable binding

**Example signature patterns:**
```rust
pub fn sanitize_session_name(name: &str) -> String
pub fn load_config() -> Config
fn detect_with_env_lookup(config: &Config, env_lookup: impl Fn(&str) -> Option<String>) -> Option<Multiplexer>
```

## Module Design

**Structure:**
- Each major feature in its own module (config, detect, sanitize, tmux, zellij)
- Module declarations at top of `main.rs`
- Public API exposed through re-exports

**Exports:**
- `pub` for items intended for external use
- Private (no modifier) for internal implementation details
- Trait definitions (`MuxLauncher`) in main.rs for shared behavior

**No barrel files:** Modules are directly declared in `main.rs` rather than using re-export patterns

---

*Convention analysis: 2026-05-02*
