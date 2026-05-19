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

# Uninstall local binary
uninstall:
    cargo uninstall codemux

# Clean build artifacts
clean:
    cargo clean

# Run pre-commit hooks manually via prek
lint:
    prek run --all-files

# Bump version atomically (uses cargo-set-version from cargo-edit)
bump PART:
    cargo set-version --bump {{PART}}

# Set exact version atomically
set-version VERSION:
    cargo set-version {{VERSION}}

# Dry-run publish to crates.io (verifies package without uploading)
publish-dry:
    cargo publish --dry-run

# Publish to crates.io (bumps patch, runs checks, publishes, tags)
publish:
    cargo set-version --bump patch && \
    NEW_VER=$$(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "codemux") | .version') && \
    echo "Bumped to v$$NEW_VER" && \
    cargo publish --allow-dirty && \
    git add -A Cargo.toml Cargo.lock && \
    git commit --no-verify -m "chore: release v$$NEW_VER" && \
    git tag -a "v$$NEW_VER" -m "Release v$$NEW_VER" && \
    echo "Published v$$NEW_VER and created tag!"

# Create a git tag for the current version (run after publish)
tag:
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION=$(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "codemux") | .version')
    git tag -a "v$VERSION" -m "Release v$VERSION"
    echo "Created tag v$VERSION"

# Full release flow: bump, check, publish, and tag
release PART='bump':
    #!/usr/bin/env bash
    set -euo pipefail
    echo "==> Bumping version ({{PART}})..."
    cargo set-version --bump {{PART}}
    NEW_VER=$(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "codemux") | .version')
    echo "==> Version: v$NEW_VER"
    echo "==> Running local checks (fmt)..."
    cargo fmt -- --check
    echo "==> Tagging..."
    git add -A Cargo.toml Cargo.lock
    git commit --no-verify -m "chore: release v$NEW_VER"
    git tag -a "v$NEW_VER" -m "Release v$NEW_VER"
    echo "==> Released v$NEW_VER!"
