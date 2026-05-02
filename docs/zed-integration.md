# Zed Integration Guide for CodeMux

This document describes how to integrate **CodeMux** with [Zed](https://zed.dev/), the high-performance multiplayer code editor. CodeMux follows the integration pattern established by [fff-gpui](https://github.com/th0jensen/fff-gpui) — a pure Rust binary that integrates via Zed's configuration files rather than a bundled extension.

## Overview

CodeMux is a drop-in replacement for your terminal shell that automatically launches tmux or zellij sessions. Unlike VS Code extensions that use editor APIs, CodeMux works as a standalone binary that Zed's integrated terminal can invoke.

## Installation

Before configuring Zed, you need to build and install the `codemux` binary:

```bash
# Clone the repository
git clone https://github.com/jellydn/codemux.git
cd codemux

# Build the release binary
cargo build --release

# The binary is now at ./target/release/codemux
# Optionally, install to your PATH:
cargo install --path . --force
```

## Integration Options

There are two recommended ways to integrate CodeMux with Zed:

### Option A: Default Terminal Shell (Recommended)

Configure Zed to use `codemux` as the default shell for all integrated terminals.

Add this to your `~/.config/zed/settings.json`:

```json
{
  "terminal": {
    "shell": {
      "program": "/absolute/path/to/codemux",
      "args": []
    }
  }
}
```

**Notes:**

- Replace `/absolute/path/to/codemux` with the actual path (e.g., `$HOME/.cargo/bin/codemux` if you ran `cargo install`)
- If `codemux` is in your PATH, you can simply use `"program": "codemux"`
- Every new terminal tab will automatically create or attach to a multiplexer session

### Option B: Task-Based Launching

Define a custom task in your user settings for on-demand multiplexer sessions.

Add this to your `~/.config/zed/tasks.json`:

```json
[
  {
    "label": "Open codemux terminal",
    "command": "/absolute/path/to/codemux",
    "use_new_terminal": true,
    "allow_concurrent_runs": false,
    "reveal": "always",
    "shell": "system"
  }
]
```

Then add a keybinding in `~/.config/zed/keymap.json`:

```json
[
  {
    "bindings": {
      "cmd-j": ["task::Spawn", { "task_name": "Open codemux terminal" }]
    }
  }
]
```

**Notes:**

- This pattern is directly inspired by [fff-gpui](https://github.com/th0jensen/fff-gpui#using-with-zed)'s README
- Press `cmd-j` (or your chosen binding) to open a new multiplexer session
- The task runs in a new terminal tab with `allow_concurrent_runs: false` to prevent multiple identical sessions

## Configuration

CodeMux supports configuration via environment variables and a TOML config file.

### Environment Variables

| Variable              | Values           | Description                         |
| --------------------- | ---------------- | ----------------------------------- |
| `CODEMUX_MULTIPLEXER` | `tmux`, `zellij` | Force a specific multiplexer        |
| `CODEMUX_AUTO_ATTACH` | `true`, `false`  | Enable/disable auto-attach behavior |
| `CODEMUX_DEBUG`       | `1`              | Enable debug logging to stderr      |

### Config File

Create `~/.config/codemux/config.toml`:

```toml
multiplexer = "tmux"  # or "zellij"
auto_attach = true
```

The config file is read automatically on each invocation. Environment variables take precedence over config file values.

## Session Naming

CodeMux automatically derives session names from the current working directory:

| Directory Name | Session Name   |
| -------------- | -------------- |
| `my-project`   | `my-project`   |
| `My Workspace` | `My-Workspace` |
| `my.project`   | `my-project`   |
| `...`          | `session`      |

When you open multiple windows for the same project, CodeMux creates uniquely suffixed sessions:

| Existing Sessions             | New Session             |
| ----------------------------- | ----------------------- |
| (none)                        | `myapp`                 |
| `myapp`                       | `myapp-2`               |
| `myapp`, `myapp-2`            | `myapp-3`               |
| `myapp`, `myapp-2`, `myapp-5` | `myapp-3` (gap-filling) |

This matches [vscode-mux](https://github.com/jellydn/vscode-mux) exactly, enabling seamless cross-editor session sharing.

## Multiplexer Commands

CodeMux generates the appropriate commands for your chosen multiplexer:

| Multiplexer | Auto-attach Mode                         | New Session Mode                      |
| ----------- | ---------------------------------------- | ------------------------------------- |
| tmux        | `tmux new-session -A -s <name> -c <cwd>` | `tmux new-session -s <name> -c <cwd>` |
| zellij      | `zellij attach <name> -c`                | `zellij -s <name>`                    |

## Troubleshooting

### Debug Mode

Enable debug logging to see the resolved configuration:

```bash
CODEMUX_DEBUG=1 codemux
```

You'll see output like:

```
[codemux] Resolved multiplexer: Tmux
[codemux] Base name: my-project
[codemux] Sanitized name: my-project
[codemux] Auto attach: true
[codemux] Final session name: my-project
[codemux] Full command: /bin/sh -l -c "tmux new-session -A -s 'my-project' -c '/path/to/my-project'"
```

### No Multiplexer Installed

If neither tmux nor zellij is found, CodeMux prints a warning and falls back to your default shell:

```
codemux: tmux/zellij not found on PATH -- falling back to $SHELL. Install tmux or zellij to enable multiplexer mode.
```

### Session Already Exists

When `auto_attach = true` (the default), CodeMux attaches to existing sessions with the same base name. To create a new session with a unique suffix, set `CODEMUX_AUTO_ATTACH=false` or `auto_attach = false` in your config.

## Comparison: Option A vs Option B

| Feature                         | Option A (settings.json) | Option B (tasks.json) |
| ------------------------------- | ------------------------ | --------------------- |
| Every new terminal uses codemux | ✅ Yes                   | ❌ No (only via task) |
| On-demand invocation            | ❌ No (always on)        | ✅ Yes                |
| Custom keybinding               | ❌ No                    | ✅ Yes                |
| Terminal choice                 | Always integrated        | Can spawn external    |

Most users prefer **Option A** for a seamless experience where every Zed terminal automatically becomes a multiplexer session.

## Credits

This integration pattern is inspired by:

- **[fff-gpui](https://github.com/th0jensen/fff-gpui)** — Pioneered the "pure binary + Zed settings/tasks" approach
- **[vscode-mux](https://github.com/jellydn/vscode-mux)** — The original multiplexer extension that CodeMux ports to Zed
- **[Zed](https://zed.dev/)** — The editor that makes this integration possible through flexible terminal configuration

## See Also

- [Main README](../README.md)
- [VS Code: vscode-mux](https://github.com/jellydn/vscode-mux)
- [fff-gpui Zed integration](https://github.com/th0jensen/fff-gpui#using-with-zed)
