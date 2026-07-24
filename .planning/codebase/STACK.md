# Technology Stack

**Analysis Date:** 2026-07-23

## Language & Runtime

| Item | Value |
|------|-------|
| Language | Rust |
| Edition | 2021 |
| Minimum Rust | 1.70 |
| Binary name | `codemux` |

## Build System

| Item | Value |
|------|-------|
| Build tool | `cargo` |
| Task runner | `just` ([`justfile`](../../justfile)) |
| Pre-commit hooks | `prek` ([`prek.toml`](../../prek.toml)) |
| Release profile | `opt-level = "s"`, LTO, stripped, `panic = "abort"`, single codegen unit |

## Workspace Structure

```
[workspace]
members = ["extension"]
resolver = "2"
```

- **Root crate** (`codemux`): binary target at `src/main.rs`
- **Extension crate** (`codemux-extension`): cdylib for Zed extension at `extension/src/lib.rs`

## Dependencies

### Runtime (root crate)

**Zero external dependencies.** Uses Rust stdlib exclusively (`std::fmt`, `std::io`, `std::path`, `std::process`, `std::collections`).

### Runtime (extension crate)

| Crate | Version | Purpose |
|-------|---------|---------|
| `zed_extension_api` | 0.7 | Zed extension registration |

### Dev Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tempfile` | 3.10 | Temporary directories for integration tests (`tests/cli.rs`) |

## Configuration

| File | Purpose |
|------|---------|
| `Cargo.toml` | Root package manifest, workspace definition |
| `extension/Cargo.toml` | Extension crate manifest |
| `extension/extension.toml` | Zed extension metadata (id, name, version) |
| `justfile` | Task definitions (build, test, release, publish) |
| `prek.toml` | Pre-commit hook configuration (fmt, clippy, test) |
| `renovate.json` | Dependency update automation |

## Platform Targets

| OS | Arch | Binary Asset |
|----|------|-------------|
| macOS | aarch64 | `codemux-macos-arm64.tar.gz` |
| macOS | x86_64 | `codemux-macos-x64.tar.gz` |
| Linux | aarch64 | `codemux-linux-arm64.tar.gz` |
| Linux | x86_64 | `codemux-linux-x64.tar.gz` |
| Windows | x86_64 | `codemux-windows-x64.exe.zip` |
| WASI | wasm32 | `codemux_extension.wasm` (Zed extension) |
