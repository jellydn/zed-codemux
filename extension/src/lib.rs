use zed_extension_api::Extension;

/// CodeMux Zed Extension
///
/// This extension provides discoverability for the CodeMux CLI binary,
/// which enables automatic tmux/zellij session management for Zed terminals.
///
/// # Requirements
///
/// The `codemux` binary must be installed and available in your PATH.
/// You can install it via:
/// - Homebrew: `brew install codemux`
/// - Cargo: `cargo install codemux`
/// - GitHub releases: Download from https://github.com/jellydn/zed-codemux/releases
///
/// Without the binary in PATH, this extension will not function.
struct CodeMuxExtension;

impl Extension for CodeMuxExtension {
    fn new() -> Self {
        Self
    }
}

zed_extension_api::register_extension!(CodeMuxExtension);
