# External Integrations

**Analysis Date:** 2026-05-02

## APIs & External Services

**None** - This is a standalone CLI tool with no network dependencies or external API calls.

## Data Storage

**Databases:**
- None - No database integration

**File Storage:**
- Local filesystem only
- Config file: `~/.config/codemux/config.toml` (or platform equivalent)
- Uses platform-specific directories:
  - Unix: `$XDG_CONFIG_HOME/codemux/config.toml` or `~/.config/codemux/config.toml`
  - Windows: `%APPDATA%\codemux\config.toml`

**Caching:**
- None - No caching layer

## Authentication & Identity

**Auth Provider:**
- None - No authentication required

## External Binaries

**Terminal Multiplexers (optional runtime dependencies):**
- `tmux` - Terminal multiplexer (preferred if available on PATH)
  - Commands used: `tmux new-session -A -s <name> -c <cwd>`, `tmux list-sessions -F '#{session_name}'`
- `zellij` - Modern terminal multiplexer (fallback option)
  - Commands used: `zellij attach <name> -c`, `zellij list-sessions -n`

**Shell:**
- `$SHELL` (Unix) or `$COMSPEC` (Windows) - Fallback when no multiplexer available
- `/bin/sh` - Default fallback on Unix
- `cmd.exe` - Default fallback on Windows

## Monitoring & Observability

**Error Tracking:**
- None - Errors printed to stderr

**Logs:**
- Debug logging to stderr when `CODEMUX_DEBUG=1`
- No persistent log files

## CI/CD & Deployment

**Hosting:**
- GitHub Releases - Binary distribution via `softprops/action-gh-release@v1`
- Future: Homebrew tap (planned v1.1)
- Future: crates.io (`cargo install`) (planned v1.1)

**CI Pipeline (`.github/workflows/ci.yml`):**
- GitHub Actions - triggered on push/PR to main branch
- Matrix builds: ubuntu-latest, macos-latest, windows-latest
- Steps:
  1. `actions/checkout@v4` - Source checkout
  2. `dtolnay/rust-toolchain@stable` - Rust toolchain with clippy, rustfmt
  3. `Swatinem/rust-cache@v2` - Dependency caching
  4. `cargo build --release` - Build
  5. `cargo test` - Run tests
  6. `cargo clippy -- -D warnings` - Linting
  7. `cargo fmt --check` - Format check

**Release Pipeline (`.github/workflows/release.yml`):**
- Triggered on tags (`v*`) or manual dispatch
- Matrix builds for 4 targets:
  - `x86_64-unknown-linux-gnu` → `codemux-linux-x64.tar.gz`
  - `x86_64-apple-darwin` → `codemux-macos-x64.tar.gz`
  - `aarch64-apple-darwin` → `codemux-macos-arm64.tar.gz`
  - `x86_64-pc-windows-msvc` → `codemux-windows-x64.exe.zip`
- Binary stripping on Unix platforms
- Archive creation (tar.gz for Unix, zip for Windows)
- GitHub Release creation with auto-generated notes

**Dependency Updates:**
- Renovate Bot (`.github/renovate.json`)
  - Schema: `https://docs.renovatebot.com/renovate-schema.json`
  - Extends: `config:recommended`

## Environment Configuration

**Required env vars:**
- None required - all have sensible defaults

**Optional env vars:**
| Variable | Default | Description |
|----------|---------|-------------|
| `CODEMUX_MULTIPLEXER` | auto-detect | Force `tmux` or `zellij` |
| `CODEMUX_AUTO_ATTACH` | `true` | Enable session auto-attachment |
| `CODEMUX_DEBUG` | `0` | Enable debug output to stderr |
| `XDG_CONFIG_HOME` | platform-specific | Config directory override |

**Secrets location:**
- None - No secrets required

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Editor Integration

**Zed Editor:**
- Integration via `settings.json` terminal shell configuration
- Alternative: Zed tasks (`tasks.json`) with keybindings (`keymap.json`)
- Extension manifest (`extension/extension.toml`) for discoverability
- WASM extension stub (`extension/src/lib.rs`) using `zed_extension_api`

---

*Integration audit: 2026-05-02*
