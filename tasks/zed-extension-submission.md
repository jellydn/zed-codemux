# Zed Extension Submission Guide

This document describes how to submit the CodeMux extension to the Zed Extensions registry.

## Prerequisites

- The `codemux` CLI binary is published and available (GitHub Releases, Homebrew, or cargo)
- The extension manifest (`extension.toml`) is complete and tested
- You have a GitHub account

## Submission Steps

### 1. Fork the Zed Extensions Repository

```bash
# Fork via GitHub web interface or CLI
gh repo fork zed-industries/extensions --clone=true
```

Or manually:

1. Visit https://github.com/zed-industries/extensions
2. Click "Fork" button
3. Clone your fork locally

### 2. Add the CodeMux Extension Entry

In your forked repository, add an entry for CodeMux:

```bash
cd extensions
mkdir -p codemux
touch codemux/extension.toml
```

Add the following to `codemux/extension.toml`:

```toml
id = "codemux"
name = "CodeMux"
version = "0.1.0"
schema_version = 1
authors = ["jellydn"]
description = "Open Zed terminals inside tmux or zellij — port of vscode-mux to Zed."
repository = "https://github.com/jellydn/zed-codemux"
```

### 3. Update the Extensions List

Add CodeMux to the main extensions list. Edit `extensions.toml` in the root of the forked repo:

```toml
[codemux]
submodule = "extensions/codemux"
```

### 4. Commit and Push

```bash
git add .
git commit -m "Add CodeMux extension"
git push origin main
```

### 5. Create a Pull Request

1. Visit your fork on GitHub
2. Click "Contribute" → "Open pull request"
3. Fill in the PR description:
   - **Title**: Add CodeMux extension
   - **Description**: Brief description of what CodeMux does and link to the main repository
4. Submit the PR

### 6. Wait for Review

The Zed team will review your PR. Common checks:

- `extension.toml` syntax is valid
- Repository link is accessible
- Description is clear and accurate

### 7. Post-Merge

Once merged, users can install CodeMux via:

1. Open Zed command palette
2. `zed: install extension`
3. Search for "CodeMux"

## Important Notes

- **Do NOT bundle the binary**: Per Zed policy, extensions must not bundle compiled binaries. The extension is for discoverability only.
- **Binary installation**: Users must install the `codemux` CLI binary separately (see README for instructions).
- **Version updates**: When releasing a new version:
  1. Update `version` in `extension.toml` in the main repo
  2. Submit a new PR to zed-industries/extensions with the updated version

## References

- [Zed Extension Developer Guide](https://zed.dev/docs/extensions/developing-extensions)
- [Zed Extensions Repository](https://github.com/zed-industries/extensions)
