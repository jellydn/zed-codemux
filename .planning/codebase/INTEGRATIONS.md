# External Integrations

**Analysis Date:** 2026-05-02

## APIs & External Services

**Terminal Multiplexers:**
- **tmux** - Primary supported multiplexer via `tmux` CLI commands
  - Commands: `tmux list-sessions`, `tmux new-session`
  - Integration: `src/tmux.rs` via `std::process::Command`
- **zellij** - Secondary supported multiplexer via `zellij` CLI commands
  - Commands: `zellij list-sessions`, `zellij attach`
  - Integration: `src/zellij.rs` via `std::process::Command`

**Zed Editor:**
- Integration via `settings.json` or `tasks.json` configuration
- No API integration - pure binary execution model

## Data Storage

**Databases:**
- None - Stateless CLI tool

**File Storage:**
- Local filesystem only
- Config file: `~/.config/codemux/config.toml` (XDG compliant)

**Caching:**
- None - No caching layer

## Authentication & Identity

**Auth Provider:**
- None - No authentication required

## Monitoring & Observability

**Error Tracking:**
- None - Errors logged to stderr only when `CODEMUX_DEBUG=1`

**Logs:**
- Debug logging controlled via `CODEMUX_DEBUG` environment variable
- Logs to stderr with `[codemux]` prefix
- No structured logging framework

## CI/CD & Deployment

**Hosting:**
- GitHub (source only, no prebuilt binaries in v1)
- Build from source: `cargo build --release`

**CI Pipeline:**
- GitHub Actions (`.github/workflows/ci.yml`)
- Matrix builds: ubuntu-latest, macos-latest, windows-latest
- Steps: build, test, clippy (with `-D warnings`), fmt check
- No automated releases (deferred to v1.1)

## Environment Configuration

**Required env vars:**
- None - All environment variables are optional

**Optional env vars:**
- `CODEMUX_MULTIPLEXER` - "tmux" or "zellij"
- `CODEMUX_AUTO_ATTACH` - "true" or "false"
- `CODEMUX_DEBUG` - "1" to enable debug output
- `SHELL` - Unix fallback shell path
- `COMSPEC` - Windows fallback shell path
- `XDG_CONFIG_HOME` - Config directory override

**Secrets location:**
- N/A - No secrets required

**Config file location:**
- Linux/macOS: `~/.config/codemux/config.toml`
- Windows: `%APPDATA%/codemux/config.toml`

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

---

*Integration audit: 2026-05-02*
