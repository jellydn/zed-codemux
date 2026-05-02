# Architecture

**Analysis Date:** 2026-05-02

## Pattern Overview

**Overall:** Command Pattern with Trait-based Abstraction

**Key Characteristics:**
- Single binary with no library crate (`[[bin]]` only in `Cargo.toml`)
- Platform-specific compilation with conditional code (`#[cfg(unix)]`, `#[cfg(windows)]`)
- Process replacement model (`exec` on Unix) - no lingering parent process
- Trait-based launcher abstraction for multiple multiplexer backends
- Pure function design for testability with dependency injection

## Layers

**CLI Layer:**
- Purpose: Argument parsing and main orchestration
- Location: `src/main.rs`
- Contains: `Cli` struct (clap derive), `main()` function, setting resolution
- Depends on: Config, Detect, Sanitize, Launcher traits
- Used by: OS exec (entry point)

**Configuration Layer:**
- Purpose: Load and parse user configuration
- Location: `src/config.rs`
- Contains: `Config` struct, `load_config()`, `parse_config_str()`
- Depends on: `serde`, `toml`, `dirs`
- Used by: CLI layer for multiplexer and auto_attach settings

**Detection Layer:**
- Purpose: Determine which multiplexer to use
- Location: `src/detect.rs`
- Contains: `Multiplexer` enum, `detect_multiplexer()`, `detect_multiplexer_with_env()`
- Depends on: Config, `which` crate for PATH probing
- Used by: CLI layer

**Sanitization Layer:**
- Purpose: Convert workspace names to valid session names
- Location: `src/sanitize.rs`
- Contains: `sanitize_session_name()`, `get_unique_session_name()`
- Depends on: `regex` crate
- Used by: CLI layer before session creation

**Launcher Abstraction Layer:**
- Purpose: Define common interface for multiplexers
- Location: `src/launcher.rs`
- Contains: `MuxLauncher` trait
- Depends on: `anyhow::Result`
- Used by: Tmux and Zellij implementations

**Multiplexer Implementations:**
- Purpose: Concrete launcher implementations
- Location: `src/tmux.rs`, `src/zellij.rs`
- Contains: `TmuxLauncher`, `ZellijLauncher` structs
- Depends on: `MuxLauncher` trait, `shell_escape`
- Used by: CLI layer via dynamic dispatch

**Utilities:**
- Purpose: Helper functions
- Location: `src/shell_escape.rs`
- Contains: `shell_escape()` for POSIX shell escaping
- Depends on: std library only
- Used by: Tmux, Zellij launchers

## Data Flow

**Terminal Launch Flow:**
1. Parse CLI arguments with clap (`Cli::parse()`)
2. Get current working directory (`std::env::current_dir()`)
3. Compute base session name from CWD basename
4. Sanitize session name (regex-based replacement)
5. Load config from `~/.config/codemux/config.toml`
6. Detect multiplexer (env → config → PATH probe)
7. Resolve settings (auto_attach, debug) with priority: env → config → default
8. List existing sessions via multiplexer command
9. Determine unique session name (with gap-filling)
10. Build shell command string via `MuxLauncher::build_command()`
11. Execute command via `exec_command()` (Unix exec or Windows spawn)

**State Management:**
- No persistent state - all state derived from:
  - Environment variables
  - Config file
  - Current working directory
  - Active multiplexer sessions (queried at runtime)

## Key Abstractions

**MuxLauncher Trait:**
- Purpose: Common interface for tmux and zellij operations
- Pattern: Strategy pattern
- Methods: `list_sessions()`, `build_command()`
- Implementations: `TmuxLauncher`, `ZellijLauncher`

**Multiplexer Enum:**
- Purpose: Type-safe representation of supported multiplexers
- Variants: `Tmux`, `Zellij`
- Conversion: `from_name()` for string parsing (case-insensitive)

**Config Struct:**
- Purpose: Typed representation of TOML config
- Fields: `multiplexer: Option<String>`, `auto_attach: Option<bool>`
- Default: Both fields are `None` (use auto-detection and defaults)

## Entry Points

**Main Binary:**
- Location: `src/main.rs`
- Triggers: Direct execution from shell or Zed terminal profile
- Responsibilities: CLI parsing, configuration resolution, multiplexer dispatch

**Process Replacement:**
- Location: `src/main.rs` - `exec_command()` function
- Unix: Uses `CommandExt::exec()` to replace process with shell + multiplexer
- Windows: Spawns and waits, then exits with child status code

## Error Handling

**Strategy:** `anyhow` for ergonomic error propagation

**Patterns:**
- Top-level `main() -> Result<()>` with `?` operator
- Graceful degradation: missing multiplexer → fallback shell
- Command failures: return empty session list rather than error (tmux/zellij not installed)
- Config failures: return default Config (file missing or invalid TOML)

## Cross-Cutting Concerns

**Logging:**
- Conditional debug logging via `CODEMUX_DEBUG=1`
- Prints to stderr with `[codemux]` prefix
- No structured logging framework

**Validation:**
- Session name validation via regex sanitization
- Multiplexer name validation via `from_name()` (case-insensitive)
- Environment variable validation at resolution time

**Authentication:**
- N/A - No authentication required

---

*Architecture analysis: 2026-05-02*
