# CodeMux for Zed — Agent Notes

## Architecture

Single Rust crate, single `codemux` binary. No `[lib]`, no workspace.

```
src/main.rs         ← entry point, CLI parsing, exec dispatch, fallback shell
src/config.rs        ← TOML config loader (~/.config/codemux/config.toml)
src/detect.rs        ← Multiplexer detection (env → config → PATH)
src/sanitize.rs      ← Session name sanitization + getUniqueSessionName
src/shell_escape.rs  ← POSIX shell-escape helper
src/launcher.rs      ← MuxLauncher trait
src/tmux.rs          ← tmux implementation
src/zellij.rs        ← zellij implementation
```

Reference implementation: [vscode-mux](https://github.com/jellydn/vscode-mux) — session naming, sanitization, and multi-window indexing must match 1:1.

## Commands

```bash
cargo build --release          # build (LTO + strip per Cargo.toml profile.release)
cargo clippy -- -D warnings    # lint (warnings as errors — required by CI)
cargo fmt --check              # format check
cargo test                     # run all tests
cargo test test_sanitize       # run a single test by name pattern
```

Task runner (`just`):

```bash
just build          # cargo build (debug)
just build-release  # cargo build --release
just test           # cargo test
just clippy         # cargo clippy -- -D warnings
just fmt-check      # cargo fmt -- --check
just fmt            # cargo fmt (auto-fix)
just check          # fmt-check + clippy + test (full gate — mirrors CI)
just run *ARGS      # cargo run -- <ARGS>
just install        # cargo install --path . --force
just clean          # cargo clean
just lint           # prek run --all-files (same pre-commit hooks as CI)
```

Task runner (`just`):

```bash
just check    # fmt-check + clippy + test (full gate)
just lint     # prek run --all-files (same pre-commit hooks as CI)
```

## Critical Constraints

- **Sanitization must match vscode-mux exactly**: replace `[^a-zA-Z0-9-]` with `-`, collapse consecutive `-`, strip leading/trailing `-`, fall back to `"session"` if empty.
- **Suffix starts at `-2` (not `-1`), with gap-filling** — `getUniqueSessionName` must produce the same results as the TypeScript original.
- **Process model**: `CommandExt::exec` on Unix — the multiplexer replaces the codemux process, no lingering parent.
- **Config file**: `~/.config/codemux/config.toml` (also respects `$XDG_CONFIG_HOME`).
- **Env var prefix**: `CODEMUX_*` (`CODEMUX_MULTIPLEXER`, `CODEMUX_AUTO_ATTACH`, `CODEMUX_DEBUG`).

## CI / Release

- **CI** (`.github/workflows/ci.yml`): builds + tests on ubuntu-latest, macos-latest, windows-latest. Runs clippy with `-D warnings` and `cargo fmt --check`.
- **Release** (`.github/workflows/release.yml`): triggered on `v*` tags. Cross-compiles for 5 targets (macOS x64+arm64, Linux x64+arm64, Windows x64). Uploads tar.gz/zip artifacts to a GitHub Release.
- **Pre-commit hooks** (`prek.toml`): runs `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` — matches CI exactly.

## Zed Extension

`extension.toml` exists for discoverability only. Per Zed policy, the binary must NOT be bundled in the extension. The extension manifest is intentionally minimal.

## Ralph Automation

`scripts/ralph/` contains an autonomous agent loop (`ralph.sh`) that reads `prd.json` and `progress.txt`, picks the next failing story, and commits. Commit message format: `feat: [Story ID] - [Story Title]`.
