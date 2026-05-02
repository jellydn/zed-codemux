# Architecture

**Analysis Date:** 2026-05-02

## Pattern Overview

**Overall:** CLI Binary with Modular Plugin Architecture

**Key Characteristics:**
- Single-purpose Rust binary that replaces itself with tmux/zellij via `exec()`
- Trait-based abstraction for multiplexer implementations (Strategy pattern)
- Configuration cascade: environment variables → config file → defaults
- Zero external runtime dependencies (pure std library)
- Cross-platform support (Unix, Windows) with platform-specific exec implementations

## Layers

**CLI/Presentation Layer:**
- Purpose: Argument parsing, help/version display, initial orchestration
- Location: `src/main.rs` (lines 38-68, 121-176)
- Contains: `parse_args()`, `main()` function, process entry point
- Depends on: config, detect, sanitize, tmux, zellij modules
- Used by: User via shell, Zed terminal integration

**Configuration Layer:**
- Purpose: Load and parse user preferences from TOML config file
- Location: `src/config.rs`
- Contains: `Config` struct, `load_config()`, `parse_config_str()` - minimal TOML parser
- Depends on: std::fs, std::path (platform-specific paths)
- Used by: main.rs for multiplexer selection and auto_attach settings

**Detection Layer:**
- Purpose: Determine which multiplexer to use based on priority cascade
- Location: `src/detect.rs`
- Contains: `Multiplexer` enum, `detect_multiplexer()`, `find_in_path()`
- Depends on: config module
- Used by: main.rs to select launcher implementation

**Sanitization Layer:**
- Purpose: Convert workspace names to valid session names
- Location: `src/sanitize.rs`
- Contains: `sanitize_session_name()`, `get_unique_session_name()`
- Depends on: std (no external deps)
- Used by: main.rs for session naming before launching

**Multiplexer Implementation Layer:**
- Purpose: Concrete implementations for tmux and zellij
- Location: `src/tmux.rs`, `src/zellij.rs`
- Contains: `TmuxLauncher`, `ZellijLauncher` structs implementing `MuxLauncher`
- Depends on: main.rs (MuxLauncher trait), std::process::Command
- Used by: main.rs via dynamic dispatch

**Extension Layer (Separate Crate):**
- Purpose: Zed extension stub for future discoverability
- Location: `extension/src/lib.rs`
- Contains: `CodeMuxExtension` implementing Zed's `Extension` trait
- Depends on: zed_extension_api
- Used by: Zed editor (WASM target)

## Data Flow

**Terminal Launch Flow:**
1. Zed invokes `codemux` binary (via settings.json shell.program or tasks.json)
2. `main()` parses CLI args (handles --help, --version)
3. Load config from `~/.config/codemux/config.toml`
4. Detect multiplexer: env var → config → PATH probe (tmux first, then zellij)
5. Sanitize session name from current working directory basename
6. Resolve `auto_attach` setting: env → config → default(true)
7. List existing sessions via `MuxLauncher.list_sessions()`
8. Compute unique session name (with gap-filling for multi-window)
9. Build command string via `MuxLauncher.build_command()`
10. `exec_command()` replaces process with shell executing the multiplexer command

**State Management:**
- No persistent state; all state derived at runtime from:
  - Environment variables (CODEMUX_MULTIPLEXER, CODEMUX_AUTO_ATTACH, CODEMUX_DEBUG)
  - Config file (optional ~/.config/codemux/config.toml)
  - Running multiplexer sessions (queried via tmux/zellij list-sessions)

## Key Abstractions

**MuxLauncher Trait:**
- Purpose: Abstract interface for terminal multiplexer operations
- Location: `src/main.rs` (lines 16-23)
- Pattern: Strategy pattern enabling polymorphic tmux/zellij handling
- Methods: `list_sessions()`, `build_command()`

**Multiplexer Enum:**
- Purpose: Type-safe representation of supported multiplexers
- Location: `src/detect.rs` (lines 4-20)
- Pattern: Enum with parsing from string names
- Variants: `Tmux`, `Zellij`

**TmuxLauncher / ZellijLauncher:**
- Purpose: Concrete implementations of MuxLauncher for each multiplexer
- Location: `src/tmux.rs`, `src/zellij.rs`
- Pattern: Struct with trait implementation
- Differences: tmux uses `-c` for cwd; zellij uses `-c` flag on attach for create-if-missing

**Config Struct:**
- Purpose: Strongly typed configuration container
- Location: `src/config.rs` (lines 4-10)
- Pattern: Optional fields with default fallbacks
- Fields: `multiplexer: Option<String>`, `auto_attach: Option<bool>`

## Entry Points

**CLI Binary (codemux):**
- Location: `src/main.rs`
- Triggers: Direct shell invocation, Zed terminal.shell.program setting, Zed task::Spawn
- Responsibilities: Full orchestration from arg parsing to exec()

**Zed Extension:**
- Location: `extension/src/lib.rs`
- Triggers: Zed extension loading (WASM)
- Responsibilities: Currently minimal stub - future discoverability hook

**Test Entry Points:**
- Location: Inline `#[cfg(test)]` modules in each source file
- Triggers: `cargo test`
- Responsibilities: Unit tests for sanitization, config parsing, detection logic, shell escaping

## Error Handling

**Strategy:** Graceful degradation with fallback chains

**Patterns:**
- Config file missing/unreadable → use defaults
- Multiplexer not found → fallback to $SHELL
- Session listing fails → return empty Vec (assumes no sessions)
- Platform-specific exec implementations with appropriate error propagation

**Safety:**
- Shell injection prevention via `shell_escape()` function (POSIX single-quote escaping)
- Path traversal prevention via session name sanitization (non-alphanumeric replaced with -)

## Cross-Cutting Concerns

**Logging:** Debug logging to stderr via `CODEMUX_DEBUG=1` environment variable

**Validation:** Input sanitization at session name generation; boolean parsing accepts true/yes/1 and false/no/0

**Security:** No authentication; relies on OS-level permissions for tmux/zellij sockets

---

*Architecture analysis: 2026-05-02*
