# Justfile for codemux — Zed terminal multiplexer
# https://github.com/casey/just

_default:
    @just --list

# Build debug binary
build:
    cargo build

# Build release binary (optimized, stripped)
build-release:
    cargo build --release

# Run tests
test:
    cargo test

# Run clippy lints
clippy:
    cargo clippy -- -D warnings

# Check code formatting
fmt-check:
    cargo fmt -- --check

# Format code
fmt:
    cargo fmt

# Run all checks (fmt + clippy + test) — or use `prek run --all-files`
check: fmt-check clippy test

# Run codemux locally (dev build)
run *ARGS:
    cargo run -- {{ARGS}}

# Install release binary locally (requires cargo-install)
install:
    cargo install --path . --force

# Clean build artifacts
clean:
    cargo clean

# Run pre-commit hooks manually via prek
lint:
    prek run --all-files
