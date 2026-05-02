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

# Dry-run publish to crates.io (verifies package without uploading)
publish-dry:
    cargo publish --dry-run

# Publish to crates.io (runs checks first, then publishes)
publish: check publish-dry
    @echo "Publishing to crates.io..."
    cargo publish
    @echo "Published successfully!"

# Create a git tag for the current version (run after publish)
tag:
    @VERSION=$(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "codemux") | .version') && \
    git tag -a "v$$VERSION" -m "Release v$$VERSION" && \
    echo "Created tag v$$VERSION"

# Full release flow: check, publish, and tag
release: publish tag
    @echo "Release complete!"
