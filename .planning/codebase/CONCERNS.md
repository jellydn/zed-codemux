# Codebase Concerns

**Analysis Date:** 2026-05-02

## Tech Debt

**Custom TOML Parser:**
- Issue: config.rs implements a simplified TOML parser instead of using a standard crate like `toml`. The parser has documented limitations: no arrays, no escaped characters, no multi-line strings, no dotted keys.
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/config.rs` (lines 66-114)
- Impact: Future config file extensions may break or require parser rewrites
- Fix approach: Migrate to `toml` crate if config complexity increases

**Process Model - Unix exec() error handling:**
- Issue: On Unix, if `exec()` fails, the error message format includes the raw error which may contain unescaped shell path
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/main.rs` (lines 232-236)
- Impact: Error message construction could theoretically be exploited if SHELL env var contains malicious input
- Fix approach: Apply shell_escape to the shell path in error messages

**Zellij CWD limitation:**
- Issue: zellij.rs build_command ignores the `_cwd` parameter in non-auto-attach mode (line 67-68 comment confirms this). Zellij doesn't support `-c` for setting cwd when creating new sessions via `zellij -s`.
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/zellij.rs` (lines 58-69)
- Impact: Users launching new zellij sessions (auto_attach=false) won't land in the expected directory
- Fix approach: Document this zellij limitation or contribute upstream feature

## Known Bugs

**None identified** - No TODO, FIXME, HACK, or XXX comments found in the codebase. The codebase appears clean of known bugs based on code comments.

## Security Considerations

**Shell Escape Function:**
- Risk: The `shell_escape` function uses POSIX single-quote escaping. While it has comprehensive tests, any inconsistency in usage could lead to command injection.
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/main.rs` (lines 26-33)
- Current mitigation: All dynamic values (session names, cwd) are escaped before shell command construction. 15 security-focused unit tests cover edge cases including null bytes, control characters, and command injection attempts.
- Recommendations: Consider using `std::process::Command` with arguments array instead of shell string construction to eliminate the escaping risk entirely

**Sanitize Session Name:**
- Risk: Session names could contain path traversal sequences or shell metacharacters
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/sanitize.rs` (lines 8-54)
- Current mitigation: Replaces all non-alphanumeric characters (except hyphen) with dashes. Has tests for path traversal patterns (`../../../etc/passwd` becomes `etc-passwd`).
- Recommendations: Current implementation is solid; 15 unit tests cover security edge cases

**Release Profile Security Trade-offs:**
- Risk: `overflow-checks = false` in release profile (line 34 of Cargo.toml) removes protection against integer overflow
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/Cargo.toml`
- Current mitigation: Codebase doesn't perform complex arithmetic; primarily string manipulation
- Recommendations: Enable overflow-checks if any arithmetic operations are added in future

**Environment Variable Injection:**
- Risk: CODEMUX_MULTIPLEXER, CODEMUX_AUTO_ATTACH values are read without validation beyond case-insensitive string matching
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/detect.rs`, `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/main.rs`
- Current mitigation: Values are only used for matching against allowed strings ("tmux", "zellij", "true", "false")
- Recommendations: Current validation is adequate for the threat model

## Performance Bottlenecks

**Linear Session Search:**
- Problem: `get_unique_session_name` uses O(n) linear search through session list
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/sanitize.rs` (lines 62-78)
- Cause: Intentional design choice - sessions typically < 10 items, HashSet overhead not worth it
- Improvement path: Already optimal for expected use case; documented in code comment

**PATH Probing:**
- Problem: `find_in_path` iterates through all PATH directories on every binary launch when env/config not set
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/detect.rs` (lines 23-46)
- Cause: No caching of detection results
- Improvement path: Add memoization if this becomes measurable overhead

## Fragile Areas

**Extension Stub Implementation:**
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/extension/src/lib.rs`
- Why fragile: The Zed extension is a minimal stub (13 lines) that only provides discoverability. It doesn't actually bundle or interface with the binary.
- Safe modification: Any changes to extension structure must maintain compatibility with `zed_extension_api`
- Test coverage: No tests for the extension crate

**Platform-Specific Config Path Logic:**
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/config.rs` (lines 23-63)
- Why fragile: Manual platform detection using `#[cfg(target_os = "windows")]` and `#[cfg(unix)]`. XDG_CONFIG_HOME handling is manual.
- Safe modification: Test on all target platforms (macOS, Linux, Windows) when changing
- Test coverage: Only unit tests for parsing; no integration tests for config file discovery

**Windows Process Model Divergence:**
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/main.rs` (lines 240-261)
- Why fragile: Windows uses `Command::status()` + `std::process::exit()` instead of Unix `exec()`. Exit code propagation differs.
- Safe modification: Test Windows builds explicitly; verify exit code behavior
- Test coverage: Windows-specific code paths are cfg-gated and may not execute in CI (GitHub Actions does test on windows-latest)

## Scaling Limits

**Session Name Collisions:**
- Current capacity: Session names can theoretically grow indefinitely with suffixes (myapp-2, myapp-3, ...myapp-N)
- Limit: No hard limit on session count; zellij/tmux may have practical limits
- Scaling path: Document limits if users report issues with hundreds of windows

## Dependencies at Risk

**zed_extension_api:**
- Risk: Version 0.1 is early/pre-release API
- Impact: Extension may break with Zed updates
- Migration plan: Monitor Zed extension API releases; the extension is minimal and easy to update

**Zero runtime dependencies:**
- The main `codemux` binary has no crate dependencies (Cargo.toml shows empty `[dependencies]` section)
- This is actually a strength - minimal supply chain attack surface

## Missing Critical Features

**Kill Subcommand:**
- Problem: No `codemux kill <name>` command to terminate sessions
- Blocks: Users must use native `tmux kill-session` or `zellij kill-session`
- Note: Documented as out of scope for v1, planned for v2.0

**Per-Workspace Configuration:**
- Problem: No `.codemux.toml` in project directories for per-workspace overrides
- Blocks: Project-specific multiplexer settings require manual env var/config file switching
- Note: Documented as v2.0 feature

**Multi-Root Workspace Support:**
- Problem: Only uses terminal CWD, doesn't handle Zed's multi-root workspaces
- Blocks: In multi-root projects, session names may not match expected workspace semantics
- Note: Documented as v2.0 feature

## Test Coverage Gaps

**PATH-Based Detection:**
- What's not tested: The actual `find_in_path` function that scans filesystem for tmux/zellij binaries
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/detect.rs` (lines 23-46, 75-86)
- Risk: PATH parsing edge cases (empty entries, malformed paths, Windows .exe handling)
- Priority: Low - documented as intentional limitation; "Making PATH testable would require additional abstraction overhead for little practical benefit"

**Actual Multiplexer Integration:**
- What's not tested: No tests actually run tmux or zellij commands
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/tmux.rs`, `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/zellij.rs`
- Risk: tmux/zellij CLI changes could break integration
- Priority: Medium - These are external dependencies; integration tests would require test environment setup

**Exec/Process Replacement:**
- What's not tested: The actual `exec_command` and `run_fallback_shell` functions don't have tests for the actual process replacement behavior
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/main.rs` (lines 222-296)
- Risk: Platform-specific exec behavior regressions
- Priority: Low - Platform-specific and hard to test in unit test framework; covered by CI builds

**Windows-Specific Path Handling:**
- What's not tested: Windows COMSPEC fallback, .exe detection in PATH
- Files: `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/detect.rs` (lines 31-38), `/Users/huynhdung/conductor/workspaces/2026-05-01-zed-codemux/brisbane/src/main.rs` (lines 108-118)
- Risk: Windows-specific code paths may have subtle bugs
- Priority: Low - CI runs on windows-latest; manual verification may be needed

---

*Concerns audit: 2026-05-02*
