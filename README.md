# CodeMux for Zed

<p align="center">
  <img src="https://raw.githubusercontent.com/jellydn/vscode-mux/main/res/icon.png" width="128" height="128" alt="CodeMux Logo">
</p>

<p align="center">
  <a href="https://crates.io/crates/codemux" target="__blank"><img src="https://img.shields.io/crates/v/codemux.svg?color=eee&amp;label=crates.io&logo=rust" alt="Crates.io" /></a>
  <a href="https://github.com/jellydn/zed-codemux/releases" target="__blank"><img src="https://img.shields.io/github/v/release/jellydn/zed-codemux.svg?color=eee&amp;label=GitHub%20Releases&logo=github" alt="GitHub Releases" /></a>
  <a href="https://github.com/jellydn/zed-codemux/stargazers" target="__blank"><img src="https://img.shields.io/github/stars/jellydn/zed-codemux?style=flat&logo=github&color=eee" alt="GitHub Stars" /></a>
  <a href="https://github.com/jellydn/vscode-mux" target="__blank"><img src="https://img.shields.io/badge/port%20from-vscode--mux-007ACC?style=flat&labelColor=eee" alt="Port from vscode-mux" /></a>
</p>

<p align="center">
  <a href="https://github.com/tmux/tmux" target="__blank"><img src="https://img.shields.io/badge/multiplexer-tmux-1BB91F?logo=tmux" alt="tmux" /></a>
  <a href="https://zellij.dev" target="__blank"><img src="https://img.shields.io/badge/multiplexer-zellij-orange?style=flat" alt="zellij" /></a>
  <a href="https://zed.dev" target="__blank"><img src="https://img.shields.io/badge/editor-Zed-084CCF?logo=zedindustries" alt="Zed" /></a>
</p>

> Open Zed terminals inside **tmux** or **zellij** — port of [vscode-mux](https://github.com/jellydn/vscode-mux) to the [Zed editor](https://zed.dev).

A small native CLI binary, in the spirit of [`fff-gpui`](https://github.com/th0jensen/fff-gpui) — **no Zed extension, no WASM, no marketplace** — just a Rust binary you point Zed at via `settings.json` or `tasks.json`. Same name, same behavior, same session names as `vscode-mux`, so jumping between editors lands you in the same tmux/zellij session.

## Why CodeMux?

If you rely on tmux or zellij for terminal multiplexing, Zed's default terminal experience breaks your flow:

- **Manual setup** — you create or attach sessions every time you open a terminal
- **Lost context** — workspace context isn't preserved across terminal sessions
- **Repetitive tasks** — repeating this setup across projects is tedious
- **Editor switching** — opening the same project in VS Code and Zed gives you two unrelated terminals

**CodeMux** solves this by making tmux/zellij the default terminal experience in Zed — **one workspace, one persistent session**, shared across editors.

✨ When you also use [vscode-mux](https://github.com/jellydn/vscode-mux), opening a project in either VS Code or Zed lands you in the **same multiplexer session** — sanitized session names, multi-window indexing, and auto-attach behavior are byte-for-byte identical.

## Features

- **Auto-launch** — opens directly into tmux or zellij when a Zed terminal is created
- **Workspace-based naming** — sessions named after the workspace directory
- **Session persistence** — closing the terminal doesn't kill the session
- **Multi-window support** — multiple windows on the same workspace get suffixed sessions (`myapp`, `myapp-2`, `myapp-3`, gap-filling), exactly matching `vscode-mux`
- **Cross-editor session sharing** — identical session naming with `vscode-mux`
- **Graceful fallback** — when neither tmux nor zellij is installed, falls back to `$SHELL`
- **Cross-platform** — macOS, Linux, Windows

## Installation

### cargo install (recommended)

```bash
cargo install codemux
```

Requires [Rust](https://rustup.rs/) toolchain. The binary will be installed to `~/.cargo/bin/codemux`.

### Homebrew (macOS & Linux)

```bash
brew tap jellydn/tap
brew install codemux
```

> **Already tapped?** Run `brew update` first (or `git -C "$(brew --repo jellydn/tap)" pull`) so Homebrew sees the latest formula. If you previously installed a broken `0.2.0`, run `brew upgrade codemux`.

### Prebuilt binaries

Release assets are tarballs (`.tar.gz`) on Unix and a zipped `.exe` on Windows:

```bash
# macOS (Apple Silicon)
curl -L https://github.com/jellydn/zed-codemux/releases/latest/download/codemux-macos-arm64.tar.gz | tar xz
sudo mv codemux /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/jellydn/zed-codemux/releases/latest/download/codemux-macos-x64.tar.gz | tar xz
sudo mv codemux /usr/local/bin/

# Linux (x64)
curl -L https://github.com/jellydn/zed-codemux/releases/latest/download/codemux-linux-x64.tar.gz | tar xz
sudo mv codemux /usr/local/bin/

# Linux (ARM64)
curl -L https://github.com/jellydn/zed-codemux/releases/latest/download/codemux-linux-arm64.tar.gz | tar xz
sudo mv codemux /usr/local/bin/
```

### Build from source (latest dev)

```bash
cargo install --git https://github.com/jellydn/zed-codemux
```

## Usage

### Option A — Default terminal shell (recommended)

`codemux` becomes the program Zed launches whenever you open a terminal pane.

**`~/.config/zed/settings.json`**:

```json
{
  "terminal": {
    "shell": {
      "program": "/usr/local/bin/codemux"
    }
  }
}
```

Open a new terminal in Zed — you're now in a tmux/zellij session named after your workspace.

### Option B — Custom task + keybind

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

## Configuration

Create config file:

```bash
codemux --init
```

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

## Session Naming

Workspace name is derived from the basename of Zed's working directory, then sanitized with the **exact algorithm from [vscode-mux](https://github.com/jellydn/vscode-mux)**:

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

## CLI Reference

```bash
codemux [OPTIONS] [ARGS]...

Arguments:
  [ARGS]...  Additional arguments to pass to the shell

Options:
  -h, --help     Print help
      --init     Create default config file at ~/.config/codemux/config.toml
  -V, --version  Print version
```

## Requirements

- **Zed** 0.150 or higher
- **OS** macOS, Linux, or Windows
- **Multiplexer** `tmux` or `zellij` on `PATH` (CodeMux still works without — falls back to `$SHELL`)
- **Build** Rust stable (only required if you build from source)

## Design Philosophy

| Decision | Why |
|---|---|
| **Pure native binary, no Zed extension** | Zed's extension API doesn't expose terminal-profile hooks (yet). Going binary-first is simpler, faster, and works today. |
| **Zed integration via user config** | Use Zed's existing `settings.json` / `tasks.json` / `keymap.json` — no new APIs to learn, no extension manifests, no marketplace review. |
| **One Rust crate, one binary** | No `[lib]`, no WASM target. Just `cargo build --release`. |
| **Zero dependencies** | Pure stdlib — no external crates for the core CLI. |
| **`exec` model** | Replaces itself with the multiplexer process — no lingering parent. |

## Roadmap

| Version | Status | Highlights |
|---|---|---|
| **v1.0** | ✅ Released | Drop-in CLI binary, vscode-mux parity, Homebrew tap, prebuilt binaries, `--init` flag |
| **v1.1** | ✅ Released | `cargo install codemux` — published to [crates.io](https://crates.io/crates/codemux) |
| **v1.2** | ✅ Released | Atomic version bumps via `cargo set-version`, streamlined `just publish` flow |
| **v2.0** | Planned | `codemux kill <name>` subcommand; per-workspace `.codemux.toml`; Zed extension marketplace |

## Releasing

### First-time setup (repo maintainer only)

```bash
# Install GitHub CLI and authenticate
brew install gh
gh auth login

# Setup required secrets (crates.io token + GitHub PAT)
./scripts/setup-secrets.sh
```

### Creating a release

```bash
# One-command release (bumps patch, checks, publishes, commits, and tags)
just publish
```

Or manually specify the bump level and use the full release flow:

```bash
just release patch    # bump patch → check → publish → commit → tag
just release minor   # bump minor → check → publish → commit → tag
```

The CI workflow automatically builds binaries for all platforms, publishes to crates.io, creates a GitHub Release, and updates the Homebrew formula.

## Credits

- [`vscode-mux`](https://github.com/jellydn/vscode-mux) by [@jellydn](https://github.com/jellydn) — the original VS Code extension this project ports
- [`fff-gpui`](https://github.com/th0jensen/fff-gpui) by [@th0jensen](https://github.com/th0jensen) — design and packaging inspiration
- The Zed team for [Zed](https://zed.dev) and the [tasks system](https://zed.dev/docs/tasks)

## Author

👤 **Huynh Duc Dung**

- Website: [https://productsway.com/](https://productsway.com/)
- Twitter: [@jellydn](https://twitter.com/jellydn)
- GitHub: [@jellydn](https://github.com/jellydn)

## Show your support

If this project has been helpful, please give it a ⭐️.

[![kofi](https://img.shields.io/badge/Ko--fi-F16061?style=flat&logo=ko-fi&logoColor=white)](https://ko-fi.com/dunghd) [![paypal](https://img.shields.io/badge/PayPal-00457C?style=flat&logo=paypal&logoColor=white)](https://paypal.me/dunghd) [![buymeacoffee](https://img.shields.io/badge/Buy_Me_A_Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black)](https://www.buymeacoffee.com/dunghd)

## License

[MIT](./LICENSE) License © 2026
