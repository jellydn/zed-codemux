# Codebase Structure

**Analysis Date:** 2026-05-02

## Directory Layout

```
./
├── .github/workflows/    # CI/CD configuration
├── .planning/            # Planning documentation (this codemap)
├── .zed/                 # Zed editor configuration
├── assets/               # Static assets (logo, etc.)
├── docs/                 # Additional documentation
├── scripts/ralph/        # Ralph autonomous agent scripts
├── src/                  # Rust source code
├── tasks/                # Task definitions
├── target/               # Cargo build artifacts (gitignored)
├── AGENTS.md             # Agent development guidelines
├── Cargo.lock            # Dependency lockfile
├── Cargo.toml            # Package manifest
├── CLAUDE.md             # Claude-specific notes
├── justfile              # Task runner recipes
├── LICENSE               # MIT License
├── prek.toml             # Pre-commit hooks config
├── progress.md           # Ralph progress tracking
├── README.md             # Project documentation
└── .gitignore            # Git ignore rules
```

## Directory Purposes

**src/:**
- Purpose: All Rust source code
- Contains: 8 source files (main.rs + 7 modules)
- Key files: `src/main.rs` (entry), `src/launcher.rs` (trait), `src/tmux.rs`, `src/zellij.rs`

**.github/workflows/:**
- Purpose: GitHub Actions CI configuration
- Contains: `ci.yml` - Matrix builds for Ubuntu, macOS, Windows

**scripts/ralph/:**
- Purpose: Ralph autonomous agent tooling
- Contains: `src/main.rs` - Agent implementation

**docs/:**
- Purpose: Additional project documentation
- Contains: Reference materials

**assets/:**
- Purpose: Static assets
- Contains: Logo and branding files

## Key File Locations

**Entry Points:**
- `src/main.rs`: CLI entry point, argument parsing, main orchestration

**Configuration:**
- `Cargo.toml`: Package manifest, dependencies, release profile
- `justfile`: Task runner definitions
- `prek.toml`: Pre-commit hook configuration

**Core Logic:**
- `src/main.rs`: Main flow, setting resolution, process execution
- `src/config.rs`: Config loading and parsing
- `src/detect.rs`: Multiplexer detection logic
- `src/sanitize.rs`: Session name sanitization
- `src/launcher.rs`: `MuxLauncher` trait definition
- `src/tmux.rs`: Tmux implementation
- `src/zellij.rs`: Zellij implementation
- `src/shell_escape.rs`: POSIX shell escaping utility

**Testing:**
- Tests are inline in each source file under `#[cfg(test)]` modules
- `cargo test` runs all tests

## Naming Conventions

**Files:**
- `snake_case.rs` for source files
- Module name matches filename (e.g., `config.rs` → `mod config`)

**Directories:**
- `snake_case` for directory names
- Standard Rust project layout

**Structs/Enums:**
- `PascalCase`: `TmuxLauncher`, `Multiplexer`, `Config`

**Functions:**
- `snake_case`: `load_config()`, `detect_multiplexer()`

**Constants:**
- No prominent constants in this codebase

**Traits:**
- `PascalCase`: `MuxLauncher`

## Where to Add New Code

**New Multiplexer Support:**
- Implementation: `src/<multiplexer>.rs`
- Add to `detect.rs`: Detection logic and enum variant
- Add to `main.rs`: Match arm in main flow

**New Configuration Options:**
- Add field to `Config` struct in `src/config.rs`
- Add resolution logic in `src/main.rs`
- Update config template in documentation

**New CLI Options:**
- Add to `Cli` struct in `src/main.rs` (clap derive)

**Utilities:**
- Shared helpers: `src/` as new module or add to existing utility module

## Special Directories

**target/:**
- Purpose: Cargo build artifacts
- Generated: Yes (by cargo build)
- Committed: No (in `.gitignore`)

**.planning/:**
- Purpose: Planning and documentation
- Generated: No (manually created)
- Committed: Optional

---

*Structure analysis: 2026-05-02*
