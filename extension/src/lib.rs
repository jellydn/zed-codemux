use zed_extension_api::Extension;

/// CodeMux Zed Extension
///
/// This extension provides discoverability for the CodeMux CLI binary,
/// which enables automatic tmux/zellij session management for Zed terminals.
struct CodeMuxExtension;

impl Extension for CodeMuxExtension {
    fn new() -> Self {
        Self
    }
}

zed_extension_api::register_extension!(CodeMuxExtension);
