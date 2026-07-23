# Architecture

**Analysis Date:** 2026-07-23

## System Pattern

**Single binary, modular CLI.** No library crate (`[lib]`), no WASM target in the root crate. The binary is a thin orchestration layer that detects the runtime environment and delegates to platform-specific multiplexer launchers.

## Process Model

**`exec` model on Unix.** The `codemux` process replaces itself with the multiplexer command via `CommandExt::exec()`, leaving no lingering parent process. On Windows, it spawns and waits, then exits with the child's code.

## Module Map

```
src/main.rs         ← Entry point, CLI parsing, MuxLauncher trait, exec dispatch, fallback shell
src/config.rs        ← TOML config loader (~/.config/codemux/config.toml)
src/detect.rs        ← Multiplexer detection (env → config → PATH probe)
src/sanitize.rs      ← Session name sanitization + gap-filling unique name generator
src/tmux.rs          ← Tmux launcher (list sessions, build commands, inside-session detection)
src/zellij.rs        ← Zellij launcher (list sessions, build commands, socket dir handling)
src/upgrade.rs       ← Self-upgrade (GitHub API, download, atomic binary replacement)

extension/
  src/lib.rs         ← Zed extension entry point (discoverability only)
  extension.toml     ← Extension manifest
```

## Data Flow

```
User opens Zed terminal
    ↓
Zed invokes `codemux` (via settings.json or tasks.json)
    ↓
parse_args() → handles --help, --version, --init, --upgrade, --check-version
    ↓
load_config() → reads ~/.config/codemux/config.toml
    ↓
detect_multiplexer() → CODEMUX_MULTIPLEXER env → config → PATH probe
    ↓
resolve_auto_attach() → CODEMUX_AUTO_ATTACH env → config → default (true)
    ↓
launcher.list_sessions() → queries tmux/zellij for existing sessions
    ↓
resolve_session_name() → match existing or generate unique (gap-filling)
    ↓
launcher.build_command() → shell-escaped multiplexer command
    ↓
exec_command() → replaces process with multiplexer (Unix) or spawns (Windows)
```

## Key Abstractions

### `MuxLauncher` Trait

```rust
pub trait MuxLauncher {
    fn list_sessions(&self) -> Result<Vec<String>, Error>;
    fn build_command(&self, name: &str, cwd: &str, auto_attach: bool) -> String;
    fn is_inside_session(&self) -> bool { false }
    fn build_inside_command(&self, name: &str, cwd: &str) -> String { ... }
}
```

Implemented by `TmuxLauncher` and `ZellijLauncher`. Enables uniform handling of both multiplexers.

### `shell_escape()`

POSIX shell escaping: wraps strings in single quotes, handles embedded quotes via `'"'"'`. Prevents command injection from session names or paths.

### Upgrade Module (`src/upgrade.rs`)

Self-contained upgrade lifecycle with three install-method strategies:
- **Cargo/Homebrew**: delegate to the package manager
- **Prebuilt**: download tarball → extract → atomic rename with temp file → verify

## Platform Strategy

| Feature | Unix | Windows |
|---------|------|---------|
| Exec model | `CommandExt::exec()` | `Command::status()` + `process::exit()` |
| Shell fallback | `$SHELL` or `/bin/sh` | `%COMSPEC%` or `cmd.exe` |
| Upgrade (prebuilt) | Supported | `WindowsNotSupported` error |
| Upgrade (cargo/homebrew) | Supported | Supported |
| Temp dir permissions | `0o700` | N/A |
| Binary permissions | `0o755` | N/A |

## Session Naming

Matches `vscode-mux` byte-for-byte: `[^a-zA-Z0-9-]` → `-`, collapse consecutive `-`, strip leading/trailing `-`, fallback to `"session"`. Truncated to 32 chars (zellij IPC socket path limit). Suffix starts at `-2` with gap-filling.
