# CodeMux for Zed — Agent Notes

## Project Status

Pre-implementation. Only planning docs exist (no `Cargo.toml`, `src/`, or `extension.toml` yet). The PRD at `tasks/prd-zed-mux.md` is the source of truth for requirements.

## Architecture

Two deliverables, one repo:

1. **`codemux` CLI binary** (Rust, statically linked) — set as Zed's `terminal.shell.program`. Attaches to or creates a tmux/zellij session named after the workspace.
2. **Companion Zed extension** (`extension.toml`) — for discoverability only. Must NOT bundle the binary per Zed policy.

Reference implementation: [vscode-mux](https://github.com/jellydn/vscode-mux) — session naming, sanitization, and multi-window indexing must match it 1:1.

## Key Constraints

- Session name sanitization: replace `[^a-zA-Z0-9-]` with `-`, collapse consecutive `-`, strip leading/trailing `-`, fall back to `"session"` if empty. Must be identical to vscode-mux's `sanitizeSessionName`.
- Multi-window indexing: suffix starts at `-2` (not `-1`), gap-fills. Must match vscode-mux's `getUniqueSessionName`.
- Process model: use `exec`-style replacement on Unix (`CommandExt::exec`) so the multiplexer becomes the foreground process — no lingering `codemux` parent.
- Config file: `~/.config/codemux/config.toml` (TOML format).
- Env var prefix: `CODEMUX_*` (e.g., `CODEMUX_MULTIPLEXER`, `CODEMUX_AUTO_ATTACH`, `CODEMUX_DEBUG`).

## Commands (once implemented)

```bash
cargo build --release          # build
cargo clippy -- -D warnings    # lint (warnings as errors)
cargo fmt --check              # format check
cargo test                     # run tests
```

## Ralph Automation

`scripts/ralph/` contains an autonomous agent loop (`ralph.sh`) that reads `prd.json` and `progress.txt`, picks the next failing story, implements it, and commits. Commit message format: `feat: [Story ID] - [Story Title]`.
