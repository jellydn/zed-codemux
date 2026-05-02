# PRD: CodeMux for Zed — Drop-in Terminal Multiplexer Integration

## 1. Introduction / Overview

**CodeMux for Zed** is a port of the [vscode-mux](https://github.com/jellydn/vscode-mux) extension to the [Zed editor](https://zed.dev). It makes opening Zed's integrated terminal automatically attach to (or create) a tmux or zellij session named after the current workspace, so users can jump between Zed and VS Code (or any other editor) and land in the **same multiplexer session** for the same project.

Because Zed's extension API does **not** expose a "terminal profile" hook like VS Code does — and following the lean packaging approach pioneered by [`th0jensen/fff-gpui`](https://github.com/th0jensen/fff-gpui) — CodeMux for Zed is delivered as a **single native CLI binary**, with **no Zed extension manifest, no marketplace listing, no WASM**.

Users wire it into Zed via existing user configuration files:

- **Option A** — set `terminal.shell.program` to `codemux` in `~/.config/zed/settings.json`.
- **Option B** — define an "Open codemux terminal" task in `~/.config/zed/tasks.json` and bind it to a hotkey in `~/.config/zed/keymap.json` (the same pattern fff-gpui uses).

Same name, same behavior, same session names as `vscode-mux` → seamless switching between VS Code and Zed for the same workspace.

## 2. Goals

- Provide a single drop-in binary (`codemux`) that Zed users set as their terminal `shell` in `settings.json`.
- **Match vscode-mux's session-naming logic exactly** so that a user opening Zed on `~/code/myapp` lands in the *same* tmux session as a VS Code window on the same path.
- Support both **tmux** and **zellij**, configurable via env var or config file.
- Gracefully fall back to the user's default shell when the multiplexer is not installed.
- Work on macOS, Linux, and Windows.
- Ship a companion **Zed extension** in [`zed-industries/extensions`](https://github.com/zed-industries/extensions) so users can `zed: install dev extension` / install from the marketplace.

## 3. User Stories

### US-001: Install the `codemux` binary
**Description:** As a Zed user, I want to install the `codemux` binary via a single command so I can start using it immediately.

**Acceptance Criteria:**
- [ ] Binary published as a release artifact for macOS (x64, arm64), Linux (x64, arm64), and Windows (x64).
- [ ] Install paths covered: `cargo install codemux`, Homebrew tap, direct release download.
- [ ] `codemux --version` prints the version and exits 0.
- [ ] `codemux --help` prints usage information.

### US-002: Configure Zed to use `codemux` as terminal shell
**Description:** As a Zed user, I want clear documentation on how to point Zed's terminal at `codemux` so it Just Works.

**Acceptance Criteria:**
- [ ] README includes a Zed `settings.json` snippet:
  ```json
  { "terminal": { "shell": { "program": "codemux" } } }
  ```
- [ ] After applying the setting and opening a new Zed terminal, the user lands inside a tmux/zellij session named after the workspace.
- [ ] Documentation explains how to scope the setting per-project via `.zed/settings.json`.

### US-003: Companion Zed extension for discoverability
**Description:** As a Zed user, I want to discover and install CodeMux from Zed's extension registry — following [Zed's extension developer guide](https://zed.dev/docs/extensions/developing-extensions) and submitting to [`zed-industries/extensions`](https://github.com/zed-industries/extensions).

**Acceptance Criteria:**
- [ ] Repo contains a valid `extension.toml` per Zed's extension manifest spec (id, name, version, schema_version, authors, description, repository).
- [ ] Repo contains an accepted-license file at the root (per Zed's license validation rules).
- [ ] If the extension ships Rust/WASM code, it includes a `Cargo.toml` with `crate-type = ["cdylib"]` and `zed_extension_api` dependency, plus `src/lib.rs` registering the extension via `zed::register_extension!`.
- [ ] Per Zed's rule "extensions must not ship binaries": the `codemux` binary is downloaded on first use (or the extension instructs the user how to install it via Homebrew / cargo / GitHub release).
- [ ] PR submitted to `zed-industries/extensions` adding this extension to the registry.
- [ ] README documents the install flow end-to-end (extension install → set `terminal.shell.program` → open terminal).

### US-004: Auto-launch into tmux/zellij with workspace-based session name
**Description:** As a developer, I want my Zed terminal to drop me into a multiplexer session named after my project so I don't have to manage sessions manually.

**Acceptance Criteria:**
- [ ] On launch, `codemux` reads its working directory and uses `basename(cwd)` as the raw session name (mirroring vscode-mux's `folder` strategy).
- [ ] Session name is sanitized via the **exact same algorithm as vscode-mux** (see FR-3).
- [ ] If the named session exists, attach to it; otherwise create it and then attach.
- [ ] tmux is launched with: `tmux new-session -A -s <name> -c <cwd>`
- [ ] zellij is launched with: `zellij attach <name> -c`

### US-005: Multi-window indexing matching vscode-mux exactly
**Description:** As a user with the same workspace open in multiple Zed windows (or mixed Zed + VS Code windows), I want sessions named with the **same scheme as vscode-mux** so I can jump between editors and find the right session.

**Acceptance Criteria:**
- [ ] Implements vscode-mux's `getUniqueSessionName` semantics:
  - Query the multiplexer for live session list (`tmux list-sessions -F '#{session_name}'` or `zellij list-sessions -n`).
  - First window: bare `baseName` (no suffix).
  - Subsequent windows: suffix starts at **`-2`**, increments to find the **first available integer** (gap-filling).
- [ ] Behavior matches the table from vscode-mux:
  | Existing sessions | Name assigned |
  |---|---|
  | `[]` | `myapp` |
  | `[myapp]` | `myapp-2` |
  | `[myapp, myapp-2]` | `myapp-3` |
  | `[myapp, myapp-2, myapp-5]` | `myapp-3` (gap-fill) |
- [ ] An "auto-attach" mode (default ON, matching vscode-mux's `autoAttach: true` + `attachIfExists: true`) re-attaches to the existing `baseName` session instead of creating a new index — so multiple windows can share a session, identical to vscode-mux.
- [ ] Auto-attach can be disabled via env var `CODEMUX_AUTO_ATTACH=false` (or config), which triggers the unique-naming path.

### US-006: Multiplexer auto-detection and selection
**Description:** As a user, I want `codemux` to pick the right multiplexer automatically, but allow me to override it.

**Acceptance Criteria:**
- [ ] Detection order: env var `CODEMUX_MULTIPLEXER` → `~/.config/codemux/config.toml` → first available on PATH (prefer `tmux`, then `zellij`).
- [ ] Supported values: `tmux`, `zellij`.
- [ ] Logs the selected multiplexer when `CODEMUX_DEBUG=1`.

### US-007: Graceful fallback when multiplexer not installed
**Description:** As a user without tmux/zellij installed, I want `codemux` to still give me a working terminal.

**Acceptance Criteria:**
- [ ] If neither `tmux` nor `zellij` is on PATH, `codemux` execs the user's default shell (`$SHELL` on Unix, `%COMSPEC%` on Windows).
- [ ] Prints a single-line warning to stderr explaining the fallback and how to install a multiplexer.
- [ ] Exit code matches the spawned shell's exit code.

### US-008: Typecheck / lint / build verification
**Description:** As a maintainer, I want CI to verify every change builds, passes lints, and runs unit tests on all target platforms.

**Acceptance Criteria:**
- [ ] `cargo build --release` succeeds on macOS, Linux, Windows in CI.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] `cargo fmt --check` passes.
- [ ] Unit tests cover:
  - Session-name sanitization (with the same fixture table as vscode-mux: `"My Workspace"` → `"My-Workspace"`, `"my.project"` → `"my-project"`, `"..."` → `"session"`, `""` → `"session"`, etc.).
  - Multi-window indexing (gap-filling, suffix start at 2).
  - Multiplexer detection order.
  - Fallback exec path.

## 4. Functional Requirements

- **FR-1:** Ship as a single statically-linked binary called `codemux`, runnable as Zed's `terminal.shell.program`.
- **FR-2:** Determine the raw session name as `basename(current_working_directory)`. If empty, use literal `"session"`.
- **FR-3:** **Sanitize the session name with the exact algorithm from vscode-mux's `sanitizeSessionName`:**
  ```
  1. Replace any character not in [a-zA-Z0-9-] with '-'   (regex: /[^a-z0-9-]/gi)
  2. Collapse consecutive '-' into a single '-'           (regex: /-+/g)
  3. Strip leading and trailing '-'                       (regex: /^-|-$/g)
  4. If the result is empty, return literal "session"
  ```
- **FR-4:** **Compute unique session name with vscode-mux's `getUniqueSessionName` semantics:**
  ```
  - Query live session list from multiplexer.
  - If baseName not in list → return baseName.
  - Else: start suffix=2; while "{baseName}-{suffix}" in list, suffix++.
  - Return "{baseName}-{suffix}".
  ```
- **FR-5:** Auto-attach behavior matches vscode-mux:
  - Default: `autoAttach=true`, `attachIfExists=true` → if `baseName` already exists, attach to it (no suffix).
  - Otherwise: invoke `getUniqueSessionName` (FR-4) to get a unique name and create that session.
- **FR-6:** Multiplexer detection in priority order: env `CODEMUX_MULTIPLEXER` → `~/.config/codemux/config.toml` → PATH probe (tmux first, zellij second).
- **FR-7:** **tmux command:** `tmux new-session -A -s <escaped-name> -c <escaped-cwd>` (auto-attach mode). Without auto-attach: `tmux new-session -s <escaped-name> -c <escaped-cwd>`. List sessions: `tmux list-sessions -F '#{session_name}'`.
- **FR-8:** **zellij command:** `zellij attach <escaped-name> -c` (auto-attach mode). Without auto-attach: `zellij -s <escaped-name>`. List sessions: `zellij list-sessions -n`. (No `cwd` flag — matches vscode-mux behavior.)
- **FR-9:** Shell escaping for session names + cwd uses **POSIX single-quote escaping** identical to vscode-mux's `shellEscape`: empty string → `''`; otherwise wrap in single quotes and escape internal `'` as `'"'"'`.
- **FR-10:** When no multiplexer is on PATH, `exec` the user's default shell (`$SHELL` on Unix → `/bin/sh` fallback; `%COMSPEC%` on Windows → `cmd.exe` fallback) and print a one-line warning to stderr.
- **FR-11:** Support `--version`, `--help` flags. All other args are forwarded to the inner shell where applicable.
- **FR-12:** When `CODEMUX_DEBUG=1` is set, emit debug logs to stderr; otherwise stay silent.
- **FR-13:** Exit with the same exit code as the spawned multiplexer or shell.
- **FR-14:** Companion Zed extension at the repo root with:
  - `extension.toml` containing required keys (`id`, `name`, `version`, `schema_version`, `authors`, `description`, `repository`).
  - Accepted-license file at root.
  - Submission PR opened against [`zed-industries/extensions`](https://github.com/zed-industries/extensions).
  - The extension itself MUST NOT bundle the binary; instead, it documents/handles binary installation per Zed's policy ("extensions should download or check for availability via the Zed Rust Extension API").

## 5. Non-Goals (Out of Scope)

- **No `codemux kill` subcommand in v1** (deferred to v2 — users can use `tmux kill-session` / `zellij kill-session` directly).
- **No per-workspace config file** (e.g., `.codemux.toml` in workspace root) in v1 (deferred).
- **No pane / layout management** inside tmux or zellij — users use native multiplexer features.
- **No editor-pane sync** with multiplexer panes.
- **No multi-root workspace handling** — uses the directory Zed launches the shell in.
- **No automatic session cleanup** — sessions persist until manually killed.
- **No GUI / settings UI** — configuration is via env var, config file, and Zed's `settings.json`.
- **No `screen` or other multiplexer support** in v1.

## 6. Design Considerations

- **Branding parity with vscode-mux:** Same name (`codemux`), same logo, same README structure. Cross-link the two projects in both READMEs so users discover both.
- **Distribution:**
  - Rust crate on crates.io (`cargo install codemux`).
  - Prebuilt binaries via GitHub Releases (macOS x64/arm64, Linux x64/arm64, Windows x64).
  - Homebrew tap formula.
  - Companion Zed extension submitted to `zed-industries/extensions`.
- **Naming conventions:** Binary `codemux`; project repo `codemux-zed` (or `zed-codemux`); config dir `~/.config/codemux/`; env var prefix `CODEMUX_*`. (Aligns with vscode-mux's `codemux.*` config keys.)
- **Documentation:** README mirrors vscode-mux structure: Why → Features → Usage → Requirements → Limitations → Troubleshooting. Include screenshots/GIFs of Zed terminal opening into a tmux session, plus a side-by-side showing the same session name in VS Code and Zed.

## 7. Technical Considerations

- **Language:** Rust — single statically-linked binary, easy cross-compilation, matches Zed's own implementation language and extension WASM target.
- **Process model:** Use `exec`-style replacement (`std::os::unix::process::CommandExt::exec` on Unix; `CreateProcess` + wait on Windows) so the multiplexer/shell becomes the terminal's foreground process. Avoids leaving `codemux` as a hanging parent.
- **Workspace name source:** `std::env::current_dir()` at startup (Zed sets the terminal's CWD to the workspace root).
- **Multi-window detection:** Identical to vscode-mux — query live multiplexer state via `list-sessions` rather than tracking window IDs out-of-band. This guarantees consistency with vscode-mux.
- **Cross-platform shell fallback:**
  - Unix: `$SHELL` → `/bin/sh`
  - Windows: `%COMSPEC%` → `cmd.exe`
- **Config file format:** TOML.
  ```toml
  multiplexer = "tmux"     # or "zellij"
  auto_attach = true       # default true
  ```
- **Dependencies (minimal):** `clap` (argv), `which` (PATH lookup), `serde` + `toml` (config), `anyhow` (errors).
- **Zed extension constraints (per Zed docs):**
  - Must include `extension.toml` at repo root.
  - Must have an accepted-license file (LICENSE / LICENCE prefix).
  - **Must not bundle the `codemux` binary** in the extension package — install path is documented separately or downloaded on first use.
  - If extension ships any Rust/WASM logic, must use `zed_extension_api` and register via `zed::register_extension!`.

## 8. Success Metrics

- A new user can install the Zed extension + `codemux` binary, set it in `settings.json`, and have a working tmux/zellij session in **under 2 minutes**.
- Opening a Zed terminal lands inside a workspace-named multiplexer session in **< 200 ms** of added latency vs. plain shell.
- **A user opening the same workspace in both VS Code (with vscode-mux) and Zed (with codemux) lands in the *same* multiplexer session by default** — verified by inspecting `tmux list-sessions` output.
- Zero crashes when multiplexer is missing — graceful fallback always succeeds.
- Single binary works unchanged across macOS / Linux / Windows (per-arch builds).
- Extension PR accepted into [`zed-industries/extensions`](https://github.com/zed-industries/extensions).

## 9. Open Questions

All v1 open questions resolved:

1. ✅ Companion Zed extension: **YES** — follow [zed-industries/extensions](https://github.com/zed-industries/extensions) and [Zed extension developer guide](https://zed.dev/docs/extensions/developing-extensions).
2. ✅ Multi-window indexing: **mirror vscode-mux exactly** — `getUniqueSessionName` with suffix start at 2, gap-filling, plus `autoAttach + attachIfExists` re-attach mode.
3. ✅ `kill` subcommand: **deferred to v2**.
4. ✅ Per-workspace config overrides: **deferred to v2**.
5. ✅ Naming: **`codemux`** (parity with VS Code extension).

---

**Saved to:** `tasks/prd-zed-mux.md`
