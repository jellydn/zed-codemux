# Technology Stack

**Analysis Date:** 2026-05-02

## Languages

**Primary:**
- Rust (Edition 2021, MSRV 1.70) - Entire codebase

**Secondary:**
- TOML - Configuration files (`Cargo.toml`, `config.toml`)
- YAML - CI/CD workflows (`.github/workflows/ci.yml`)
- Shell - Justfile task runner recipes

## Runtime

**Environment:**
- Rust native binary (no runtime required)
- Cross-platform: macOS, Linux, Windows

**Package Manager:**
- Cargo (built-in Rust package manager)
- Lockfile: `Cargo.lock` present

## Frameworks

**Core:**
- `clap` 4.5 - CLI argument parsing with derive macros
- `serde` 1.0 - Serialization for config parsing
- `anyhow` 1.0 - Error handling and propagation
- `regex` 1.10 - Session name sanitization

**Testing:**
- Built-in `cargo test` with standard Rust test framework
- `tempfile` 3.10 - Test utilities (dev dependency)

**Build/Dev:**
- `just` - Task runner (justfile for common commands)
- `prek` - Git hooks framework for pre-commit checks
- `cargo fmt` - Code formatting
- `cargo clippy` - Linting (warnings as errors in CI)

## Key Dependencies

**Critical:**
- `clap` 4.5 - CLI parsing with derive macro support for clean command definitions
- `which` 6.0 - PATH probing to detect tmux/zellij installations
- `dirs` 5.0 - Cross-platform config directory detection
- `regex` 1.10 - Session name sanitization (matches vscode-mux algorithm exactly)

**Infrastructure:**
- `serde` + `toml` 0.8 - Config file parsing
- `anyhow` - Ergonomic error handling with `Result<T>`

## Configuration

**Environment:**
- `CODEMUX_MULTIPLEXER` - Override preferred multiplexer (tmux/zellij)
- `CODEMUX_AUTO_ATTACH` - Override auto-attach behavior (true/false)
- `CODEMUX_DEBUG` - Enable debug logging (set to `1`)
- `SHELL` / `COMSPEC` - Fallback shell detection
- `XDG_CONFIG_HOME` - Config directory override

**Build:**
- `Cargo.toml` - Main manifest with release profile (LTO + strip)
- `justfile` - Task definitions (build, test, clippy, fmt)
- `prek.toml` - Pre-commit hooks configuration

## Platform Requirements

**Development:**
- Rust 1.70 or later
- tmux or zellij installed (optional, for testing)
- just (optional, for task running)
- prek (optional, for git hooks)

**Production:**
- Target: Native binary executable
- No runtime dependencies
- Falls back to system shell if no multiplexer found

---

*Stack analysis: 2026-05-02*
