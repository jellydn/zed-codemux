# Coding Conventions

**Analysis Date:** 2026-05-02

## Naming Patterns

**Files:**
- `snake_case.rs` for all Rust source files
- Module name matches filename exactly

**Functions:**
- `snake_case` throughout
- Verb-based names: `load_config()`, `detect_multiplexer()`, `sanitize_session_name()`
- Test functions prefixed with `test_`: `test_parse_valid_toml()`

**Variables:**
- `snake_case` throughout
- Descriptive names: `sanitized_name`, `auto_attach`, `session_name`

**Types:**
- Structs/Enums: `PascalCase` - `Config`, `Multiplexer`, `TmuxLauncher`
- Traits: `PascalCase` - `MuxLauncher`
- Generic parameters: Single uppercase - `T`

## Code Style

**Formatting:**
- `rustfmt` (via `cargo fmt`)
- Enforced in CI: `cargo fmt --check` fails the build

**Linting:**
- `clippy` with warnings as errors: `cargo clippy -- -D warnings`
- CI enforces zero warnings

**Line Length:**
- No explicit limit, but generally follows rustfmt defaults

## Import Organization

**Order:**
1. Standard library imports (`std::`)
2. External crate imports (third-party)
3. Internal module imports (`crate::`)

**Grouping:**
- Separate groups with blank lines
- Alphabetical within groups

**Example from `src/main.rs`:**
```rust
use crate::config::{load_config, Config};
use crate::detect::{detect_multiplexer, Multiplexer};
use crate::launcher::MuxLauncher;
use crate::sanitize::{get_unique_session_name, sanitize_session_name};
use crate::tmux::TmuxLauncher;
use crate::zellij::ZellijLauncher;
use anyhow::Result;
use clap::Parser;
use std::collections::HashMap;
```

**Path Aliases:**
- None used - all imports are explicit

## Error Handling

**Patterns:**
- `anyhow::Result<T>` as primary return type for fallible functions
- `?` operator for error propagation
- `unwrap_or()` and `unwrap_or_else()` for providing defaults
- Graceful degradation rather than panics

**Error Types:**
- `anyhow::Error` for most errors
- `std::io::Error` from `Command` operations

**No Panic Zones:**
- Config parsing failures return defaults
- Missing multiplexer results in fallback shell
- Command not found returns empty list (not error)

## Logging

**Framework:** Console output to stderr only

**Patterns:**
- Debug logging behind `CODEMUX_DEBUG=1` check
- Prefix: `[codemux]` for all debug messages
- `eprintln!()` for debug output (stderr)

**Example:**
```rust
if debug {
    eprintln!("[codemux] Resolved multiplexer: {:?}", multiplexer);
}
```

## Comments

**When to Comment:**
- Doc comments (`///`) on all public items
- Implementation comments for non-obvious logic
- Priority order comments for configuration resolution

**Doc Comments:**
- All structs, enums, traits, and public functions have doc comments
- Example: `/// Configuration for codemux, loaded from...`

**Inline Comments:**
- Used sparingly for algorithm explanations
- Example: "Priority 1: Environment variable"

## Function Design

**Size:**
- Functions are generally small (10-30 lines)
- `main()` orchestrates by calling smaller functions
- Testable helper functions extracted from main logic

**Parameters:**
- Prefer borrowing: `&str`, `&Path`, `&Config`
- Environment injection pattern for testability: `fn foo(env: &HashMap<String, String>)`

**Return Values:**
- `Result<T>` for fallible operations
- Direct values for infallible operations
- `bool` for predicates

## Module Design

**Exports:**
- Each module exports its public types at module level
- Re-exports for common types in consuming modules

**Barrel Files:**
- Not used - Rust doesn't use barrel file pattern
- Each file is its own module

**Module Structure:**
- Main module declares submodules: `mod config; mod detect; ...`
- Each submodule is self-contained with its own tests

---

*Convention analysis: 2026-05-02*
