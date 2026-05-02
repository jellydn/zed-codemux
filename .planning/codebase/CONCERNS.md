# Codebase Concerns

**Analysis Date:** 2026-05-02

## Tech Debt

**Platform-Specific Code Duplication:**
- Issue: Similar but slightly different implementations for Unix vs Windows in multiple places
- Files: `src/main.rs` (shell detection, exec_command, run_fallback_shell)
- Impact: Maintenance burden when adding new platforms
- Fix approach: Extract platform abstraction layer or use conditional compilation more aggressively

**Regex Compilation:**
- Issue: Regex patterns are compiled on every `sanitize_session_name()` call
- Files: `src/sanitize.rs` - 4 regex patterns compiled per call
- Impact: Unnecessary overhead (though minimal for CLI tool)
- Fix approach: Use `lazy_static` or `once_cell` for compiled regexes, or switch to string operations

**Shell Escape Edge Cases:**
- Issue: POSIX shell escaping may not handle all edge cases (e.g., null bytes, specific control characters)
- Files: `src/shell_escape.rs`
- Impact: Potential security issue with specially crafted session names
- Fix approach: Add more comprehensive test cases, consider using a dedicated shell-escape crate

## Known Bugs

**None identified** - No TODO, FIXME, or HACK comments in codebase

## Security Considerations

**Shell Injection:**
- Risk: Session names and paths are passed to shell commands
- Files: `src/tmux.rs`, `src/zellij.rs` - `build_command()` functions
- Current mitigation: `shell_escape()` function wraps values in single quotes
- Recommendations: 
  - Add tests for injection attempts (`'; rm -rf /; '`)
  - Consider using `std::process::Command` with args instead of shell strings
  - Audit all user-controlled inputs

**Path Traversal:**
- Risk: CWD is used in shell commands
- Files: `src/main.rs`
- Current mitigation: Path is shell-escaped before use
- Recommendations: Validate paths don't contain null bytes or control characters

**Config File:**
- Risk: Config file is in user-writable location
- Files: `src/config.rs`
- Current mitigation: Graceful fallback on parse failure (doesn't panic)
- Recommendations: Consider file permissions check, warn if config is world-writable

## Performance Bottlenecks

**Regex Overhead:**
- Problem: 4 regex compilations per sanitize call
- Files: `src/sanitize.rs`
- Cause: Regex::new() in function body rather than static/lazy
- Improvement path: Use `lazy_regex` or compile once with `once_cell::sync::Lazy`

**Process Spawning:**
- Problem: Multiple subprocess calls (list-sessions, then new-session/attach)
- Files: `src/tmux.rs`, `src/zellij.rs`
- Cause: Architecture requires querying existing sessions
- Improvement path: Consider caching session list, though minimal impact for CLI tool

## Fragile Areas

**PATH Dependency:**
- Files: `src/detect.rs`, `src/tmux.rs`, `src/zellij.rs`
- Why fragile: Relies on tmux/zellij being in PATH at runtime
- Safe modification: Use full paths if needed, or improve error messages
- Test coverage: Tests check NotFound behavior but don't verify actual multiplexer interaction

**Windows Support:**
- Files: `src/main.rs` (multiple `#[cfg(windows)]` blocks)
- Why fragile: Less tested than Unix path (CI runs but no actual multiplexers on Windows)
- Safe modification: Test on Windows with actual zellij/tmux installations
- Test coverage: Limited - many Windows paths not covered by tests

**Session Name Edge Cases:**
- Files: `src/sanitize.rs`
- Why fragile: Complex regex-based algorithm must match vscode-mux exactly
- Safe modification: Add more test cases, maintain compatibility test suite
- Test coverage: Good unit tests, but no integration tests against actual vscode-mux

## Scaling Limits

**Session Name Length:**
- Current capacity: Unlimited (but shell-escaped)
- Limit: tmux/zellij may have their own limits
- Scaling path: Add validation and truncation

**Concurrent Executions:**
- Current capacity: No internal limits
- Limit: Race condition possible between list-sessions and new-session
- Scaling path: Accept race as design limitation (vscode-mux has same behavior)

## Dependencies at Risk

**Regex Crate:**
- Risk: Compiles on every call (performance, not security)
- Impact: Minor overhead
- Migration plan: Switch to `lazy_regex` or manual string operations

**Which Crate:**
- Risk: None identified - stable and widely used

**Clap Crate:**
- Risk: Major version changes could require updates
- Impact: CLI argument parsing would need updates
- Migration plan: Clap has good migration guides, currently on v4

## Missing Critical Features

**Kill Subcommand:**
- Feature gap: No `kill` subcommand to terminate sessions
- Blocks: Users must use `tmux kill-session` directly
- Note: Intentionally deferred to v2 per README

**Per-Workspace Config:**
- Feature gap: No `.codemux.toml` in project directories
- Blocks: Per-project multiplexer preferences
- Note: Intentionally deferred to v2 per README

**Pane/Layout Management:**
- Feature gap: No control over pane layout or window arrangement
- Blocks: Advanced terminal layouts
- Note: Out of scope for v1

## Test Coverage Gaps

**Integration with Real Multiplexers:**
- What's not tested: Actual tmux/zellij command execution
- Files: `src/tmux.rs`, `src/zellij.rs`
- Risk: Commands could fail with new multiplexer versions
- Priority: Medium - tested implicitly via `list_sessions()` return types

**Windows Shell Fallback:**
- What's not tested: Windows-specific shell detection and execution paths
- Files: `src/main.rs` (Windows cfg blocks)
- Risk: Windows fallback may not work correctly
- Priority: Medium - affects Windows users without multiplexers

**Cross-Platform Config Path:**
- What's not tested: Windows config directory resolution
- Files: `src/config.rs`
- Risk: Config not found on Windows
- Priority: Low - uses standard `dirs` crate

---

*Concerns audit: 2026-05-02*
