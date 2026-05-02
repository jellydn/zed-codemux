# Codebase Structure

**Analysis Date:** 2026-05-02

## Directory Layout

```
/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/
├── src/                         # Main CLI source code
│   ├── main.rs                  # Entry point, MuxLauncher trait, exec dispatch
│   ├── config.rs                # TOML configuration loading and parsing
│   ├── detect.rs                # Multiplexer detection (env/config/PATH)
│   ├── sanitize.rs              # Session name sanitization and unique name generation
│   ├── tmux.rs                  # TmuxLauncher implementation
│   └── zellij.rs                # ZellijLauncher implementation
├── extension/                   # Zed extension (separate crate)
│   ├── src/
│   │   └── lib.rs               # Extension entry point (stub)
│   ├── Cargo.toml               # Extension dependencies (zed_extension_api)
│   └── extension.toml           # Zed extension manifest
├── scripts/                     # Build/utility scripts
│   └── ralph/
│       └── src/main.rs          # Ralph script placeholder
├── Cargo.toml                   # Main crate manifest
├── Cargo.lock                   # Dependency lock file
├── README.md                    # User documentation
├── LICENSE                      # MIT license
├── justfile                     # Just command runner recipes
└── .zed/                        # Zed editor configuration
    └── tasks.json               # Project-local build tasks
```

## Directory Purposes

**src/:**
- Purpose: Core CLI implementation
- Contains: All Rust modules for the codemux binary
- Key files: `main.rs` (entry), `config.rs`, `detect.rs`, `sanitize.rs`, `tmux.rs`, `zellij.rs`

**extension/:**
- Purpose: Zed extension packaging (future-proofing)
- Contains: WASM-compilable extension code, manifests
- Key files: `src/lib.rs`, `Cargo.toml`, `extension.toml`

**scripts/:**
- Purpose: Auxiliary tooling
- Contains: Ralph autonomous agent integration placeholder

**.zed/:**
- Purpose: Editor-specific configuration
- Contains: tasks.json for "Build and run codemux" task

## Key File Locations

**Entry Points:**
- `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/main.rs`: CLI binary entry point
- `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/extension/src/lib.rs`: Zed extension entry point

**Configuration:**
- `~/.config/codemux/config.toml`: User configuration file (runtime)
- `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/Cargo.toml`: Main crate manifest (dependencies, binary target, release profile)
- `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/extension/Cargo.toml`: Extension crate manifest

**Core Logic:**
- `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/main.rs`: MuxLauncher trait definition, orchestration, exec dispatch
- `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/tmux.rs`: Tmux command builder and session listing
- `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/zellij.rs`: Zellij command builder and session listing

**Testing:**
- Inline `#[cfg(test)]` modules in each .rs file (idomatic Rust)

## Naming Conventions

**Files:**
- `snake_case.rs`: Module names (config.rs, detect.rs, sanitize.rs)
- Module name matches file name (e.g., `mod config;` in `config.rs`)

**Directories:**
- `lowercase/`: Directory names (src/, extension/, scripts/)

**Types:**
- `PascalCase`: Structs and traits (TmuxLauncher, MuxLauncher, Config)
- `snake_case`: Functions and variables (load_config, detect_multiplexer)
- `UPPER_SNAKE_CASE`: Constants (VERSION from CARGO_PKG_VERSION)

## Where to Add New Code

**New Multiplexer Support:**
- Implementation: `src/<multiplexer>.rs` following TmuxLauncher/ZellijLauncher pattern
- Registration: Add variant to `detect.rs` Multiplexer enum
- Integration: Update `main.rs` match arms in `main()` function

**New Configuration Options:**
- Schema: Add field to `Config` struct in `src/config.rs`
- Parsing: Update `parse_config_str()` in same file
- Usage: Access via `main.rs` resolution functions

**Utilities:**
- Shared helpers: Add to `main.rs` (shell_escape) or appropriate module

## Special Directories

**target/:**
- Purpose: Cargo build artifacts
- Generated: Yes (by cargo build)
- Committed: No (in .gitignore)

**extension/target/:**
- Purpose: Extension crate build artifacts
- Generated: Yes
- Committed: No

**.context/:**
- Purpose: Claude Code context files
- Generated: Yes
- Committed: Yes (tracked in repo)

---

*Structure analysis: 2026-05-02*
