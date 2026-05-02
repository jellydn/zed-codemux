# CodeMux for Zed

> Open Zed terminals inside **tmux** or **zellij** — port of [vscode-mux](https://github.com/jellydn/vscode-mux) to the [Zed editor](https://zed.dev).

[![tmux](https://img.shields.io/badge/multiplexer-tmux-1BB91F?logo=tmux)](https://github.com/tmux/tmux) [![zellij](https://img.shields.io/badge/multiplexer-zellij-orange)](https://zellij.dev) [![Zed](https://img.shields.io/badge/editor-Zed-084CCF?logo=zedindustries)](https://zed.dev) [![Rust](https://img.shields.io/badge/built%20with-Rust-DEA584?logo=rust)](https://www.rust-lang.org)

A small native CLI binary, in the spirit of [`fff-gpui`](https://github.com/th0jensen/fff-gpui) — **no Zed extension, no WASM, no marketplace** — just a Rust binary you point Zed at via `settings.json` or `tasks.json`. Same name, same behavior, same session names as `vscode-mux`, so jumping between editors lands you in the same tmux/zellij session.

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

## Design philosophy

This project follows the same lean approach as [`th0jensen/fff-gpui`](https://github.com/th0jensen/fff-gpui):

| Decision | Why |
|---|---|
| **Pure native binary, no Zed extension** | Zed's extension API doesn't expose terminal-profile hooks (yet). Going binary-first is simpler, faster, and works today. |
| **Zed integration via user config** | Use Zed's existing `settings.json` / `tasks.json` / `keymap.json` — no new APIs to learn, no extension manifests, no marketplace review. |
| **One Rust crate, one binary** | No `[lib]`, no WASM target. Just `cargo build --release`. |
| **Zero dependencies** | Pure stdlib — no external crates for the core CLI. |
| **`exec` model** | Replaces itself with the multiplexer process — no lingering parent. |

```diagram
╭──────────────────────╮      ╭─────────────────────╮      ╭──────────────────╮
│   Zed terminal /     │      │  codemux CLI binary │      │  tmux  /  zellij │
│   task::Spawn        │─────▶│  (Rust, single bin) │─────▶│  session named   │
│   settings.json or   │      │  exec-style replace │      │  after workspace │
│   tasks.json         │      ╰─────────────────────╯      ╰──────────────────╯
╰──────────────────────╯
```

---

## Features (v1)

- **Auto-launch** — opens directly into tmux or zellij when a Zed terminal is created.
- **Workspace-based naming** — sessions named after the workspace directory.
- **Session persistence** — closing the terminal doesn't kill the session.
- **Multi-window support** — multiple windows on the same workspace get suffixed sessions (`myapp`, `myapp-2`, `myapp-3`, gap-filling), exactly matching `vscode-mux`.
- **Cross-editor session sharing** — identical session naming with `vscode-mux`.
- **Graceful fallback** — when neither tmux nor zellij is installed, falls back to `$SHELL`.
- **Cross-platform** — macOS, Linux, Windows.

### Out of scope for v1
- `kill` subcommand → use `tmux kill-session` / `zellij kill-session` directly. Deferred to v2.
- Per-workspace `.codemux.toml` overrides → deferred to v2.
- Pane / layout management.
- Multi-root workspace handling.

---

## Installation

### Homebrew (macOS & Linux)

The easiest way to install codemux on macOS and Linux:

```bash
brew tap jellydn/tap
brew install codemux
```

### Prebuilt binaries

Download prebuilt binaries from [GitHub Releases](https://github.com/jellydn/zed-codemux/releases):

```bash
# macOS (Apple Silicon)
curl -L -o codemux https://github.com/jellydn/zed-codemux/releases/latest/download/codemux-macos-arm64
chmod +x codemux
sudo mv codemux /usr/local/bin/

# macOS (Intel)
curl -L -o codemux https://github.com/jellydn/zed-codemux/releases/latest/download/codemux-macos-x64
chmod +x codemux
sudo mv codemux /usr/local/bin/

# Linux (x64)
curl -L -o codemux https://github.com/jellydn/zed-codemux/releases/latest/download/codemux-linux-x64
chmod +x codemux
sudo mv codemux /usr/local/bin/
```

### Build from source

Requires Rust stable:

```sh
git clone https://github.com/jellydn/zed-codemux
cd zed-codemux
cargo build --release
```

The binary will be at `target/release/codemux`. Move it onto your `$PATH`:

```sh
install -m 0755 target/release/codemux /usr/local/bin/codemux
```

---

## Zed integration

There are **two ways** to wire `codemux` into Zed. Pick whichever fits your workflow.

### Option A — Default terminal shell (recommended)

`codemux` becomes the program Zed launches whenever you open a terminal pane.

**`~/.config/zed/settings.json`** (or workspace-local `.zed/settings.json`):

```json
{
  "terminal": {
    "shell": {
      "program": "/usr/local/bin/codemux"
    }
  }
}
```

Open a new terminal in Zed (`` Ctrl+` ``) — you're now in a tmux/zellij session named after your workspace.

### Option B — Custom task + keybind (fff-gpui-style)

If you'd rather summon a multiplexer-attached terminal with a hotkey instead of replacing every terminal, use Zed's task system — exactly the pattern [fff-gpui uses for its picker](https://github.com/th0jensen/fff-gpui#zed-integration).

**`~/.config/zed/tasks.json`**:

```json
{
  "label": "Open codemux terminal",
  "command": "/usr/local/bin/codemux",
  "use_new_terminal": true,
  "allow_concurrent_runs": false,
  "reveal": "always",
  "reveal_target": "dock",
  "hide": "never",
  "shell": "system",
  "show_summary": false,
  "show_command": false,
  "save": "none"
}
```

**`~/.config/zed/keymap.json`**:

```json
{
  "context": "Workspace",
  "bindings": {
    "cmd-j": [
      "task::Spawn",
      { "task_name": "Open codemux terminal" }
    ]
  }
}
```

Now `Cmd+J` summons a terminal already attached to the current workspace's tmux/zellij session.

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

### Quick setup with `--init`

Create a default config file interactively:

```bash
codemux --init
```

This creates `~/.config/codemux/config.toml` with sensible defaults. The command will fail gracefully if the config already exists.

### Environment variables

| Variable | Default | Description |
|---|---|---|
| `CODEMUX_MULTIPLEXER` | (auto-detect) | Force `tmux` or `zellij` |
| `CODEMUX_AUTO_ATTACH` | `true` | If `false`, every window gets its own suffixed session |
| `CODEMUX_DEBUG` | `0` | Set to `1` to print debug logs to stderr |

---

## Session naming (vscode-mux parity)

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

When `auto_attach = false`, CodeMux finds the **first available** suffixed name starting at `-2`:

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
- **Build** Rust stable (only required if you build from source)

---

## Project layout

```
.
├── README.md                    ← this file
├── Cargo.toml
├── Cargo.lock
├── src/
│   ├── main.rs                  ← entry point, exec dispatch
│   ├── sanitize.rs              ← session-name sanitization
│   ├── shell_escape.rs          ← POSIX shell escape
│   ├── config.rs                ← TOML config loader
│   ├── detect.rs                ← multiplexer detection
│   ├── launcher.rs              ← MuxLauncher trait
│   ├── tmux.rs                  ← tmux implementation
│   ├── zellij.rs                ← zellij implementation
│   └── unique_name.rs           ← getUniqueSessionName
├── .zed/
│   └── tasks.json               ← project-local "Build and run codemux" task
├── .github/
│   └── workflows/
│       └── ci.yml               ← cross-platform CI
├── tasks/
│   └── prd-zed-mux.md           ← full PRD
└── scripts/
    └── ralph/
        ├── prd.json             ← Ralph autonomous-executor task list
        └── ralph.sh
```

The repo includes a `.zed/tasks.json` with a one-key build-and-run task, mirroring the convenience pattern from [fff-gpui's `.zed/tasks.json`](https://github.com/th0jensen/fff-gpui/blob/main/.zed/tasks.json):

```json
{
  "label": "Build and run codemux",
  "command": "cargo build --release && ./target/release/codemux",
  "hide": "on_success",
  "save": "all"
}
```

---

## Differences from vscode-mux

| | vscode-mux | codemux for Zed |
|---|---|---|
| Distribution | VS Code Marketplace + Open VSX | Source build (`cargo build --release`) |
| Hook into editor | VS Code `TerminalProfile` API | Zed `settings.json terminal.shell.program` *or* `tasks.json` + keymap |
| Implementation | TypeScript (Reactive VS Code) | Rust binary, exec-style replacement |
| Kill command | ✅ Command palette | ❌ v1 — use native `tmux/zellij kill-session` |
| Multi-root workspaces | ✅ first folder | ❌ v1 — uses Zed's terminal CWD |
| Session naming, sanitization, indexing | identical | identical (verified by shared fixture tests) |

---

## Roadmap

See [`tasks/prd-zed-mux.md`](tasks/prd-zed-mux.md) for the full PRD and [`scripts/ralph/prd.json`](scripts/ralph/prd.json) for the Ralph task list driving autonomous implementation.

| Version | Highlights |
|---|---|
| **v1.0** | Drop-in CLI binary, vscode-mux parity (sanitization + indexing + commands), source build, two Zed integration patterns, Homebrew tap |
| **v1.1** | `cargo install codemux`, prebuilt GitHub Release binaries |
| **v2.0** | `codemux kill <name>` subcommand; per-workspace `.codemux.toml`; optional Zed extension manifest if/when Zed adds terminal-profile API |

---

## Credits

- [`vscode-mux`](https://github.com/jellydn/vscode-mux) by [@jellydn](https://github.com/jellydn) — the original VS Code extension this project ports.
- [`fff-gpui`](https://github.com/th0jensen/fff-gpui) by [@th0jensen](https://github.com/th0jensen) — design and packaging inspiration: pure native binary, Zed integration purely via user config, no extension manifest.
- The Zed team for [Zed](https://zed.dev) and the [tasks system](https://zed.dev/docs/tasks).

## License

MIT — see [LICENSE](LICENSE).
