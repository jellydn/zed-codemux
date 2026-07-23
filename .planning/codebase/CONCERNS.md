# Concerns

**Analysis Date:** 2026-07-23

## Technical Debt

### 1. Simplified TOML Parser (`src/config.rs`)

The config parser is a hand-rolled, lenient TOML parser that:
- Does not support arrays, tables, inline tables, dotted keys, or escaped characters
- Accepts unquoted values (non-standard TOML)
- Silently ignores unknown keys and invalid values

**Risk:** Users may write valid TOML syntax that silently fails to parse. A proper TOML parser (e.g., `toml` crate) would provide validation but would add an external dependency, violating the zero-dependency philosophy.

### 2. Manual JSON Parsing (`src/upgrade.rs`)

`parse_tag_name()` manually extracts `tag_name` from the GitHub API JSON response using string matching. While it handles whitespace around the colon, it:
- Cannot handle nested objects or unexpected JSON structures
- Is fragile if GitHub changes the response format
- Would fail silently if `"tag_name"` appears in a nested context

**Mitigation:** The GitHub Releases API response format is stable. The function returns `Option` and surfaces a `ParseError` on failure.

### 3. `--upgrade` CLI Flags Not Tested

The `--upgrade`, `--check-version`, `--check`, and `--yes` flags are wired into `parse_args()` but have no integration tests in `tests/cli.rs`. The upgrade module's unit tests only cover `parse_version`.

**Risk:** Flag parsing regressions would only be caught manually.

### 4. Duplicate `VERSION` Constant

Both `src/main.rs` and `src/upgrade.rs` define:
```rust
const VERSION: &str = env!("CARGO_PKG_VERSION");
```

**Risk:** If version handling changes in one file, the other could become inconsistent. Should be consolidated into a shared constant.

### 5. `run_command()` Simplistic Splitting

`run_command()` splits the command string on whitespace:
```rust
let mut parts = cmd.split_whitespace();
```

This works for the two known commands (`cargo install codemux --force`, `brew upgrade codemux`) but would break on quoted arguments or arguments containing spaces.

**Mitigation:** Only used for hardcoded command strings. Not exposed for arbitrary user input.

### 6. Zellij Inside-Session Detection (TODO in `src/zellij.rs`)

The `ZellijLauncher` does not implement `is_inside_session()`:
```rust
// TODO: Detect when running inside a zellij session ...
// See: https://github.com/jellydn/zed-codemux/pull/12
```

**Impact:** When running inside zellij, creating a new terminal triggers `zellij attach <name> -c` (nested session) instead of `zellij action new-tab`.

## Security

### 1. curl `--fail` Now Present (Fixed)

`--fail` and `--max-time` have been added to both curl calls in `src/upgrade.rs`. Previously, HTTP errors (404, 500) would silently succeed and produce unexpected output.

### 2. Temporary Directory Permissions (Fixed)

The prebuilt upgrade temp directory now uses:
- Unique name with PID: `codemux-upgrade-{PID}`
- Restrictive permissions: `0o700` on Unix

Previously used a static name (`codemux-upgrade`) without restrictive permissions, creating a symlink/race risk.

### 3. Orphaned Temp File Cleanup (Fixed)

If `replace_binary()` fails (e.g., permission denied), the temporary `.codemux-upgrade-tmp` file is now cleaned up via `remove_file()` in the error path.

### 4. Shell Injection Prevention

`shell_escape()` wraps all user-controlled strings in single quotes before embedding in shell commands. This prevents injection from session names, paths, or CLI arguments.

## Performance

- **Session listing**: Linear scan of existing sessions for collision detection (`O(n)`). Adequate for typical usage (< 10 sessions).
- **PATH probing**: Manual directory traversal in `which()` and `find_in_path()`. Could be slow on systems with large PATH, but the probe is a one-time startup cost.
- **Zero allocation in hot path**: `shell_escape()` uses `format!()` (allocates), but it's called once per invocation.

## Dependencies

- **Root crate**: Zero runtime dependencies — a deliberate design choice. This simplifies audit, reduces build times, and eliminates supply-chain risk.
- **Extension crate**: Only `zed_extension_api = "0.7"` — a required dependency for Zed extension compatibility.
- **Dev only**: `tempfile = "3.10"` — well-audited, widely used crate.

## Platform Gaps

| Concern | Detail |
|---------|--------|
| Windows upgrade | `upgrade()` returns `WindowsNotSupported` — prebuilt binary replacement not implemented |
| Zellij CWD | In non-auto-attach mode, zellij ignores the `-c` flag — warns to stderr |
| Zellij socket path | Uses `/tmp/z` instead of `$TMPDIR` to avoid 103-byte IPC socket limit on macOS |
| ARM Linux CI | `ubuntu-24.04-arm` runner is relatively new — may have limited availability |
