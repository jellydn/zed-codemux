# CodeMux for Zed

> Open Zed terminals inside **tmux** or **zellij** — port of [vscode-mux](https://github.com/jellydn/vscode-mux) to the [Zed editor](https://zed.dev).

[![tmux](https://img.shields.io/badge/multiplexer-tmux-1BB91F?logo=tmux)](https://github.com/tmux/tmux) [![zellij](https://img.shields.io/badge/multiplexer-zellij-orange)](https://zellij.dev) [![Zed](https://img.shields.io/badge/editor-Zed-084CCF?logo=zedindustries)](https://zed.dev)

---

## Why CodeMux for Zed?

If you rely on tmux or zellij for terminal multiplexing, Zed's default terminal experience breaks your flow:

- **Manual setup** — you create or attach sessions every time you open a terminal.
- **Lost context** — workspace context isn't preserved across terminal sessions.
- **Repetitive tasks** — repeating this setup across projects is tedious.
- **Editor switching** — opening the same project in VS Code and Zed gives you two unrelated terminals.

**CodeMux** solves this by making tmux/zellij the default terminal experience in Zed — **one workspace, one persistent session**, shared across editors.

> ✨ When you also use [vscode-mux](https://github.com/jellydn/vscode-mux), opening a project in either VS Code or Zed lands you in the **same multiplexer session** — sanitized session names, multi-window indexing, and auto-attach behavior are byte-for-byte identical.

---

## How it works

Zed's extension API does **not** currently expose a "terminal profile" hook (unlike VS Code). So CodeMux for Zed ships in two pieces:

```diagram
╭─────────────────────╮       ╭─────────────────────╮
│   Zed Extension     │       │  codemux CLI binary │
│  (discoverability)  │──────▶│  (does the work)    │
│  extension.toml     │       │  Rust, statically   │
│  in zed-industries/ │       │  linked, single bin │
│  extensions         │       ╰──────────┬──────────╯
╰─────────────────────╯                  │
                                         ▼
                              ╭──────────────────────╮
                              │ tmux  /  zellij      │
                              │ session = workspace  │
                              ╰──────────────────────╯
```

1. **`codemux` CLI binary** — set as Zed's `terminal.shell.program`. When Zed opens a terminal, it spawns `codemux`, which attaches to (or creates) a tmux/zellij session named after the workspace.
2. **Companion Zed extension** — published to [`zed-industries/extensions`](https://github.com/zed-industries/extensions) for discoverability. The extension does **not** bundle the binary (per Zed's policy); it documents the install flow.

---

## Features (v1)

- **Auto-launch** — opens directly into tmux or zellij when a Zed terminal is created.
- **Workspace-based naming** — sessions named after the workspace directory.
- **Session persistence** — closing the terminal doesn't kill the session.
- **Multi-window support** — multiple Zed windows on the same workspace get suffixed sessions (`myapp`, `myapp-2`, `myapp-3`, gap-filling), exactly matching `vscode-mux`.
- **Cross-editor session sharing** — identical session naming with `vscode-mux` so jumping between Zed and VS Code lands you in the same session.
- **Graceful fallback** — when neither tmux nor zellij is installed, falls back to `$SHELL`.
- **Cross-platform** — macOS, Linux, Windows.

### Out of scope for v1
- `kill` subcommand (use `tmux kill-session` / `zellij kill-session` directly) — deferred to v2.
- Per-workspace `.codemux.toml` overrides — deferred to v2.
- Pane / layout management.
- Multi-root workspace handling.

---

## Installation

### 1. Install the `codemux` binary

```bash
# Homebrew (macOS / Linux)
brew install jellydn/tap/codemux

# Cargo
cargo install codemux

# Or download a prebuilt binary from the GitHub Releases page.
```

### 2. Install the Zed extension (optional, for discoverability)

In Zed, open the command palette → **`zed: install extension`** → search for **CodeMux**.

### 3. Point Zed's terminal at `codemux`

Open Zed `settings.json` (`zed: open settings`) and add:

```json
{
  "terminal": {
    "shell": {
      "program": "codemux"
    }
  }
}
```

Open a new terminal in Zed (`` Ctrl+` ``) — you're now in a tmux/zellij session named after your workspace. 🎉

---

## Configuration

### Choose a multiplexer

Detection order:

1. Env var `CODEMUX_MULTIPLEXER` (`tmux` | `zellij`)
2. `~/.config/codemux/config.toml`
3. PATH probe — prefers `tmux`, then `zellij`

```toml
# ~/.config/codemux/config.toml
multiplexer = "tmux"     # or "zellij"
auto_attach = true       # default true; same workspace ⇒ shared session
```

### Environment variables

| Variable | Default | Description |
|---|---|---|
| `CODEMUX_MULTIPLEXER` | (auto-detect) | Force `tmux` or `zellij` |
| `CODEMUX_AUTO_ATTACH` | `true` | If `false`, every window gets its own suffixed session |
| `CODEMUX_DEBUG` | `0` | Set to `1` to print debug logs to stderr |

---

## Session naming

Workspace name is derived from the basename of Zed's working directory, then sanitized with the **exact algorithm from [`vscode-mux`](https://github.com/jellydn/vscode-mux)**:

1. Replace any character not in `[a-zA-Z0-9-]` with `-`.
2. Collapse consecutive `-` into a single `-`.
3. Strip leading and trailing `-`.
4. If empty, fall back to literal `"session"`.

| Input | Sanitized |
|---|---|
| `My Workspace` | `My-Workspace` |
| `my.project` | `my-project` |
| `-myproject-` | `myproject` |
| `...` | `session` |

When `auto_attach = false` (or a session with the bare name already exists and you opt out of attach), CodeMux finds the **first available** suffixed name starting at `-2`:

| Existing sessions | Name assigned |
|---|---|
| `[]` | `myapp` |
| `[myapp]` | `myapp-2` |
| `[myapp, myapp-2]` | `myapp-3` |
| `[myapp, myapp-2, myapp-5]` | `myapp-3` (gap-fill) |

---

## Multiplexer commands invoked

| Mode | tmux | zellij |
|---|---|---|
| Auto-attach (default) | `tmux new-session -A -s <name> -c <cwd>` | `zellij attach <name> -c` |
| Always-new | `tmux new-session -s <name> -c <cwd>` | `zellij -s <name>` |
| List sessions | `tmux list-sessions -F '#{session_name}'` | `zellij list-sessions -n` |

---

## Requirements

- **Zed** 0.150 or higher
- **OS** macOS, Linux, or Windows
- **Multiplexer** `tmux` or `zellij` on `PATH` (CodeMux still works without — falls back to `$SHELL`)

---

## Project layout

```
.
├── README.md                    # ← this file
├── tasks/
│   └── prd-zed-mux.md           # full PRD
├── scripts/
│   └── ralph/
│       ├── prd.json             # Ralph-format task list (autonomous executor)
│       ├── ralph.sh
│       └── progress.txt
├── Cargo.toml                   # codemux binary crate
├── src/
│   └── main.rs
├── extension.toml               # Zed extension manifest
└── LICENSE
```

---

## Roadmap

See [`tasks/prd-zed-mux.md`](tasks/prd-zed-mux.md) for the full PRD and [`scripts/ralph/prd.json`](scripts/ralph/prd.json) for the Ralph task list driving autonomous implementation.

| Version | Highlights |
|---|---|
| **v1.0** | Drop-in CLI binary, vscode-mux parity (sanitization + indexing + commands), companion Zed extension, cross-platform |
| **v2.0** | `codemux kill <name>` subcommand; per-workspace `.codemux.toml` overrides |

---

## Credits

- [`vscode-mux`](https://github.com/jellydn/vscode-mux) by [@jellydn](https://github.com/jellydn) — the original VS Code extension this project ports.
- The Zed team for the [extension developer guide](https://zed.dev/docs/extensions/developing-extensions).

## License

MIT — see [LICENSE](LICENSE).
