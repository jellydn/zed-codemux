# Technology Stack

**Analysis Date:** 2026-05-02

## Languages

**Primary:**
- Rust (Edition 2021, Minimum Rust Version 1.70) - Main CLI binary implementation (`src/*.rs`)
- TOML - Configuration files (`Cargo.toml`, `config.toml`, `prek.toml`, `extension.toml`)

**Secondary:**
- YAML - CI/CD workflow definitions (`.github/workflows/*.yml`)
- Shell (Just) - Task runner scripts (`justfile`)

## Runtime

**Environment:**
- Native binary (no runtime required)
- Cross-platform: macOS, Linux, Windows

**Package Manager:**
- Cargo - Rust package manager and build system
- Lockfile: `Cargo.lock` present (version 3)

## Frameworks

**Core:**
- Standard library only (`std::process`, `std::env`, `std::io`, `std::collections`)
- No external runtime frameworks

**Testing:**
- Built-in Rust test framework (`#[cfg(test)]` modules)
- `tempfile` 3.27.0 - Temporary file/directory creation for tests

**Build/Dev:**
- Cargo - Build system (`cargo build`, `cargo test`)
- Clippy - Linting (`cargo clippy`)
- rustfmt - Code formatting (`cargo fmt`)
- Just - Task runner (`just build`, `just test`, `just check`)
- Prek - Git hooks framework (`prek run --all-files`)

## Key Dependencies

**Main Binary (`/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/Cargo.toml`):**
- None (zero runtime dependencies - pure std library)

**Development Dependencies:**
- `tempfile` 3.27.0 - Cross-platform temporary file utilities for testing

**Extension (`/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/extension/Cargo.toml`):**
- `zed_extension_api` 0.1 - Zed editor extension API for WASM-based plugin

**Build-time Dependencies (from `Cargo.lock`):**
- `wit-bindgen` 0.51.0/0.57.1 - WIT bindings generation for WASM components
- `wasm-encoder` 0.244.0 - WebAssembly binary encoding
- `wasmparser` 0.244.0 - WebAssembly binary parsing

## Configuration

**Environment Variables:**
- `CODEMUX_MULTIPLEXER` - Force multiplexer selection (`tmux` or `zellij`)
- `CODEMUX_AUTO_ATTACH` - Control session auto-attachment (`true`/`false`)
- `CODEMUX_DEBUG` - Enable debug logging to stderr (`1` for enabled)
- `SHELL` - Fallback shell path (Unix)
- `COMSPEC` - Fallback shell path (Windows)
- `XDG_CONFIG_HOME` - Config directory override
- `HOME`/`APPDATA` - Platform-specific config directories

**Config Files:**
- `~/.config/codemux/config.toml` - User configuration (multiplexer preference, auto_attach)
- `Cargo.toml` - Build configuration with optimized release profile
- `justfile` - Development task definitions
- `prek.toml` - Git hook configuration (fmt, clippy, test)

**Release Profile (`Cargo.toml` lines 28-35):**
- `opt-level = "s"` - Optimize for size
- `lto = true` - Link-time optimization enabled
- `strip = true` - Strip symbols
- `panic = "abort"` - Abort on panic
- `codegen-units = 1` - Single codegen unit for maximum optimization
- `overflow-checks = false` - Disable overflow checks in release

## Platform Requirements

**Development:**
- Rust toolchain (stable, >= 1.70)
- `clippy` and `rustfmt` components
- Just task runner (`cargo install just`)
- Prek git hooks framework

**Production:**
- No runtime dependencies
- Requires `tmux` or `zellij` in PATH for multiplexer functionality
- Falls back to system shell if neither multiplexer is available

**Target Platforms:**
- x86_64-unknown-linux-gnu (Linux x64)
- x86_64-apple-darwin (macOS Intel)
- aarch64-apple-darwin (macOS Apple Silicon)
- x86_64-pc-windows-msvc (Windows x64)

---

*Stack analysis: 2026-05-02*
