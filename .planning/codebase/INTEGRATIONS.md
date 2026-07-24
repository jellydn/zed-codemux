# External Integrations

**Analysis Date:** 2026-07-23

## External APIs

| Service | Purpose | Called From | Authentication |
|---------|---------|-------------|----------------|
| GitHub Releases API (`api.github.com/repos/jellydn/zed-codemux/releases/latest`) | Fetch latest release tag for self-upgrade | `src/upgrade.rs::check_latest()` | None (public API) |
| GitHub Releases (download) | Download prebuilt binary tarball | `src/upgrade.rs::do_prebuilt_upgrade()` | None |

## Distribution Channels

| Channel | Details |
|---------|---------|
| **crates.io** | Published as `codemux` crate. Token: `CARGO_REGISTRY_TOKEN` secret in CI |
| **Homebrew** | Formula at `jellydn/homebrew-tap/Formula/codemux.rb`. Token: `HOMEBREW_TAP_TOKEN` |
| **GitHub Releases** | Prebuilt binaries for 5 platform targets, auto-uploaded on tag push |
| **Zed Extension** | `.wasm` binary uploaded as release asset, registered via `zed_extension_api` |

## CI/CD

| Provider | Workflow | Trigger |
|----------|----------|---------|
| GitHub Actions | `.github/workflows/ci.yml` | Push, PR to `main` |
| GitHub Actions | `.github/workflows/release.yml` | Tag push (`v*`) or manual dispatch |

### CI Checks

- Build + test on `ubuntu-latest`, `macos-latest`, `windows-latest`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`
- GitGuardian secret scanning
- Socket Security vulnerability scanning

### Release Pipeline

1. Build binaries for all 5 platform targets (parallel matrix)
2. Publish to crates.io
3. Build Zed extension (wasm32-wasip1)
4. Create GitHub Release with release notes
5. Update Homebrew formula in `jellydn/homebrew-tap`

## Tools Accessed at Runtime

| Tool | Purpose | Fallback |
|------|---------|----------|
| `curl` | GitHub API requests + binary downloads | Falls back to `pwsh.exe` on Windows; errors if not found |
| `tar` | Extract prebuilt binary tarballs | None (required for prebuilt upgrade path) |
| `tmux` | Terminal multiplexer (session creation, listing) | Graceful fallback to zellij or `$SHELL` |
| `zellij` | Terminal multiplexer (session creation, listing) | Graceful fallback to tmux or `$SHELL` |

## Configuration Files

| File | Format | Purpose |
|------|--------|---------|
| `~/.config/codemux/config.toml` | TOML | User configuration (multiplexer preference, auto-attach) |
| `~/.config/zed/settings.json` | JSON | Zed terminal shell configuration (Option A) |
| `~/.config/zed/tasks.json` | JSON | Zed task definitions (Option B) |
| `~/.config/zed/keymap.json` | JSON | Zed keybindings |
