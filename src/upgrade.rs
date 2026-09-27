use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::Command;

/// Result of a successful upgrade operation.
#[derive(Debug, Clone, PartialEq)]
pub struct UpgradeResult {
    /// The version before the upgrade.
    pub previous: String,
    /// The version after the upgrade.
    pub current: String,
    /// Path to the current binary.
    pub path: PathBuf,
}

/// Errors that can occur during the upgrade process.
#[derive(Debug)]
pub enum UpgradeError {
    /// curl was not found on PATH.
    CurlNotFound,
    /// A network request failed.
    NetworkError(String),
    /// Writing to the target path requires elevated privileges.
    PermissionDenied(PathBuf),
    /// The current OS/arch combination has no prebuilt binary.
    UnsupportedPlatform {
        os: &'static str,
        arch: &'static str,
    },
    /// The running version is already the latest.
    AlreadyLatest { current: String },
    /// The GitHub API response could not be parsed.
    ParseError(String),
    /// The downloaded binary failed validation.
    VerificationError(String),
    /// A filesystem I/O error occurred.
    IoError(io::Error),
    /// Windows upgrade via binary replacement is not yet supported.
    WindowsNotSupported,
}

impl fmt::Display for UpgradeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpgradeError::CurlNotFound => {
                write!(f, "upgrade requires curl. Install curl or update manually: cargo install codemux --force")
            }
            UpgradeError::NetworkError(msg) => {
                write!(f, "could not reach GitHub API: {}", msg)
            }
            UpgradeError::PermissionDenied(path) => {
                write!(
                    f,
                    "permission denied writing to {}. Try: sudo codemux --upgrade",
                    path.display()
                )
            }
            UpgradeError::UnsupportedPlatform { os, arch } => {
                write!(
                    f,
                    "no prebuilt binary for {}/{}. Use: cargo install codemux --force",
                    os, arch
                )
            }
            UpgradeError::AlreadyLatest { current } => {
                write!(f, "already up to date (v{})", current)
            }
            UpgradeError::ParseError(msg) => {
                write!(f, "failed to parse version: {}", msg)
            }
            UpgradeError::VerificationError(msg) => {
                write!(f, "upgrade verification failed: {}", msg)
            }
            UpgradeError::IoError(e) => {
                write!(f, "{}", e)
            }
            UpgradeError::WindowsNotSupported => {
                write!(f, "Windows upgrade is not yet supported. Download the latest release from https://github.com/jellydn/zed-codemux/releases")
            }
        }
    }
}

impl From<io::Error> for UpgradeError {
    fn from(e: io::Error) -> Self {
        UpgradeError::IoError(e)
    }
}

fn debug_enabled() -> bool {
    std::env::var("CODEMUX_DEBUG")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn find_curl() -> Result<PathBuf, UpgradeError> {
    let curl_candidates: &[&str] = if cfg!(windows) {
        &["curl.exe", "curl.cmd"]
    } else {
        &["curl"]
    };
    for name in curl_candidates {
        if let Ok(path) = which(name) {
            return Ok(path);
        }
    }
    Err(UpgradeError::CurlNotFound)
}

fn which(name: &str) -> Result<PathBuf, ()> {
    let path_env = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_env) {
        let full = dir.join(name);
        if is_executable(&full) {
            return Ok(full);
        }
    }
    Err(())
}

fn is_executable(path: &std::path::Path) -> bool {
    if !path.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    #[cfg(not(unix))]
    true
}

/// Extracts the `tag_name` value from a minimal GitHub API JSON response.
/// Handles optional whitespace around the colon (e.g. `"tag_name" : "v1.0"`).
fn parse_tag_name(json: &str) -> Option<String> {
    let key = "\"tag_name\"";
    let pos = json.find(key)?;
    let after_key = &json[pos + key.len()..];
    // skip optional whitespace before and after the colon
    let after_colon = after_key.trim_start().strip_prefix(':')?.trim_start();
    let value_start = after_colon.strip_prefix('"')?;
    let value_end = value_start.find('"')?;
    Some(value_start[..value_end].to_string())
}

type VersionCore = (u32, u32, u32);
type ParsedVersion<'a> = (VersionCore, Option<&'a str>);

/// Parses a version string like `"1.2.3"` or `"v1.2.3"` into a `(major, minor, patch)` tuple.
/// Ignores prerelease and build metadata in the returned core version.
#[cfg(test)]
fn parse_version(s: &str) -> Option<VersionCore> {
    parse_version_parts(s).map(|(core, _)| core)
}

fn parse_version_parts(s: &str) -> Option<ParsedVersion<'_>> {
    let s = s.strip_prefix('v').unwrap_or(s);
    let version = s.split_once('+').map_or(s, |(version, _)| version);
    let (core, prerelease) = match version.split_once('-') {
        Some((_core, "")) => return None,
        Some((core, prerelease)) => (core, Some(prerelease)),
        None => (version, None),
    };
    let mut parts = core.split('.');
    let parsed = (
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    );
    if parts.next().is_some() {
        return None;
    }
    Some((parsed, prerelease))
}

fn compare_prerelease(left: Option<&str>, right: Option<&str>) -> std::cmp::Ordering {
    match (left, right) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(left), Some(right)) => {
            let mut left = left.split('.');
            let mut right = right.split('.');
            loop {
                match (left.next(), right.next()) {
                    (Some(left), Some(right)) => {
                        let ordering = compare_prerelease_identifier(left, right);
                        if ordering != std::cmp::Ordering::Equal {
                            return ordering;
                        }
                    }
                    (Some(_), None) => return std::cmp::Ordering::Greater,
                    (None, Some(_)) => return std::cmp::Ordering::Less,
                    (None, None) => return std::cmp::Ordering::Equal,
                }
            }
        }
    }
}

fn compare_prerelease_identifier(left: &str, right: &str) -> std::cmp::Ordering {
    let left_numeric = left.chars().all(|c| c.is_ascii_digit());
    let right_numeric = right.chars().all(|c| c.is_ascii_digit());
    match (left_numeric, right_numeric) {
        (true, true) => {
            let left = left.trim_start_matches('0');
            let right = right.trim_start_matches('0');
            left.len().cmp(&right.len()).then_with(|| left.cmp(right))
        }
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        (false, false) => left.cmp(right),
    }
}

/// Compares two version strings (with optional "v" prefix).
/// Falls back to lexical comparison when either version cannot be parsed.
fn version_cmp(latest: &str, current: &str) -> std::cmp::Ordering {
    match (parse_version_parts(latest), parse_version_parts(current)) {
        (Some((latest_core, latest_pre)), Some((current_core, current_pre))) => latest_core
            .cmp(&current_core)
            .then_with(|| compare_prerelease(latest_pre, current_pre)),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => latest.cmp(current),
    }
}

/// How codemux was installed (determines the upgrade strategy).
#[derive(Debug, PartialEq)]
pub enum InstallMethod {
    Cargo,
    Homebrew,
    Prebuilt,
}

/// Detects the installation method by inspecting the current executable path.
pub fn detect_install_method() -> InstallMethod {
    let exe = std::env::current_exe().ok();
    match exe.as_ref().and_then(|p| p.to_str()) {
        Some(path) if path.contains(".cargo/bin") => InstallMethod::Cargo,
        Some(path) if path.contains("Cellar") || path.contains("homebrew") => {
            InstallMethod::Homebrew
        }
        _ => InstallMethod::Prebuilt,
    }
}

fn platform_asset_name() -> Result<&'static str, UpgradeError> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Ok("codemux-macos-arm64.tar.gz"),
        ("macos", "x86_64") => Ok("codemux-macos-x64.tar.gz"),
        ("linux", "aarch64") => Ok("codemux-linux-arm64.tar.gz"),
        ("linux", "x86_64") => Ok("codemux-linux-x64.tar.gz"),
        ("windows", _) => Err(UpgradeError::WindowsNotSupported),
        (os, arch) => Err(UpgradeError::UnsupportedPlatform { os, arch }),
    }
}

fn prompt_yes_no(prompt: &str) -> bool {
    eprint!("{} [Y/n]: ", prompt);
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(0) | Err(_) => return false,
        Ok(_) => {}
    }
    let trimmed = input.trim().to_lowercase();
    trimmed.is_empty() || trimmed == "y" || trimmed == "yes"
}

/// Queries the GitHub API for the latest release tag name.
pub fn check_latest() -> Result<String, UpgradeError> {
    let curl = find_curl()?;

    let mut cmd = Command::new(&curl);
    cmd.args([
        "-sL",
        "--fail",
        "--max-time",
        "15",
        "-H",
        "Accept: application/vnd.github+json",
        "https://api.github.com/repos/jellydn/zed-codemux/releases/latest",
    ]);

    if debug_enabled() {
        eprintln!("[codemux] Running: {:?}", cmd);
    }

    let output = cmd
        .output()
        .map_err(|e| UpgradeError::NetworkError(e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(UpgradeError::NetworkError(stderr.to_string()));
    }

    let body = String::from_utf8_lossy(&output.stdout);

    if debug_enabled() {
        eprintln!("[codemux] API response: {}", body);
    }

    let tag = parse_tag_name(&body).ok_or_else(|| {
        UpgradeError::ParseError("could not find tag_name in GitHub API response".into())
    })?;

    Ok(tag)
}

/// Returns the latest version string (without "v" prefix).
pub fn check_version_only() -> Result<String, UpgradeError> {
    let latest = check_latest()?;
    Ok(latest.strip_prefix('v').unwrap_or(&latest).to_string())
}

/// Checks for and performs a self-upgrade.
///
/// When `check_only` is true, reports whether an update is available without
/// performing the upgrade. When `yes` is true, skips the confirmation prompt.
///
/// On Windows this returns `WindowsNotSupported` — use `check_version_only()`
/// to check for new releases on Windows instead.
#[cfg_attr(windows, allow(unreachable_code, unused_variables))]
pub fn upgrade(check_only: bool, yes: bool) -> Result<UpgradeResult, UpgradeError> {
    #[cfg(windows)]
    {
        return Err(UpgradeError::WindowsNotSupported);
    }

    let latest_tag = check_latest()?;
    let latest_ver = latest_tag.strip_prefix('v').unwrap_or(&latest_tag);

    if version_cmp(&latest_tag, &format!("v{}", crate::VERSION)) != std::cmp::Ordering::Greater {
        if check_only {
            println!("Already up to date (current: v{})", crate::VERSION);
            return Ok(UpgradeResult {
                previous: crate::VERSION.to_string(),
                current: crate::VERSION.to_string(),
                path: std::env::current_exe().unwrap_or_default(),
            });
        }
        return Err(UpgradeError::AlreadyLatest {
            current: crate::VERSION.to_string(),
        });
    }

    if check_only {
        println!(
            "Latest version: v{} (current: v{})",
            latest_ver,
            crate::VERSION
        );
        return Ok(UpgradeResult {
            previous: crate::VERSION.to_string(),
            current: latest_ver.to_string(),
            path: std::env::current_exe().unwrap_or_default(),
        });
    }

    let method = detect_install_method();

    match method {
        InstallMethod::Cargo => handle_external_upgrade(
            "cargo",
            &["install", "codemux", "--force"],
            "cargo",
            latest_ver,
            yes,
        ),
        InstallMethod::Homebrew => {
            handle_external_upgrade("brew", &["upgrade", "codemux"], "Homebrew", latest_ver, yes)
        }
        InstallMethod::Prebuilt => {
            let current_exe = std::env::current_exe().map_err(UpgradeError::IoError)?;
            perform_prebuilt_upgrade(&latest_tag, latest_ver, &current_exe)
        }
    }
}

/// Prompts the user and optionally runs an external package-manager upgrade command.
fn handle_external_upgrade(
    program: &str,
    args: &[&str],
    label: &str,
    latest_ver: &str,
    yes: bool,
) -> Result<UpgradeResult, UpgradeError> {
    let command = std::iter::once(program)
        .chain(args.iter().copied())
        .collect::<Vec<_>>()
        .join(" ");
    println!("Detected {} installation.", label);
    println!("Recommended command: {}", command);
    let upgraded = yes || prompt_yes_no("Run this command?");
    if upgraded {
        run_command(program, args)?;
    } else {
        println!("Upgrade cancelled.");
    }
    Ok(UpgradeResult {
        previous: crate::VERSION.to_string(),
        current: if upgraded {
            latest_ver.to_string()
        } else {
            crate::VERSION.to_string()
        },
        path: std::env::current_exe().unwrap_or_default(),
    })
}

/// Downloads and atomically replaces the current binary with the latest prebuilt release.
fn perform_prebuilt_upgrade(
    latest_tag: &str,
    latest_ver: &str,
    current_exe: &std::path::Path,
) -> Result<UpgradeResult, UpgradeError> {
    let asset = platform_asset_name()?;

    if debug_enabled() {
        eprintln!("[codemux] Downloading asset: {}", asset);
    }

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp_dir =
        std::env::temp_dir().join(format!("codemux-upgrade-{}-{}", std::process::id(), unique));
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new().mode(0o700).create(&tmp_dir)?;
    }
    #[cfg(not(unix))]
    std::fs::create_dir(&tmp_dir)?;

    // Ensure the temp directory is always cleaned up, even on error.
    let result = do_prebuilt_upgrade(latest_tag, latest_ver, current_exe, asset, &tmp_dir);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    result
}

/// Core prebuilt upgrade logic (download, extract, verify, replace).
fn do_prebuilt_upgrade(
    latest_tag: &str,
    latest_ver: &str,
    current_exe: &std::path::Path,
    asset: &str,
    tmp_dir: &std::path::Path,
) -> Result<UpgradeResult, UpgradeError> {
    let archive_path = tmp_dir.join(asset);
    let download_url = format!(
        "https://github.com/jellydn/zed-codemux/releases/download/{}/{}",
        latest_tag, asset
    );

    let curl = find_curl()?;
    let status = Command::new(&curl)
        .args([
            "-sL",
            "--fail",
            "--max-time",
            "60",
            "-o",
            &archive_path.to_string_lossy(),
            &download_url,
        ])
        .status()
        .map_err(|e| UpgradeError::NetworkError(e.to_string()))?;

    if !status.success() {
        return Err(UpgradeError::NetworkError("download failed".into()));
    }

    let extract_status = Command::new("tar")
        .args([
            "xzf",
            &archive_path.to_string_lossy(),
            "-C",
            &tmp_dir.to_string_lossy(),
        ])
        .status()?;

    if !extract_status.success() {
        return Err(UpgradeError::NetworkError("extraction failed".into()));
    }

    let extracted_binary = tmp_dir.join("codemux");
    if !extracted_binary.is_file() {
        return Err(UpgradeError::NetworkError(
            "extracted binary not found".into(),
        ));
    }

    verify_version(&extracted_binary, latest_ver)?;
    replace_binary(&extracted_binary, current_exe)?;

    println!("codemux: upgraded v{} → v{} ✓", crate::VERSION, latest_ver);

    Ok(UpgradeResult {
        previous: crate::VERSION.to_string(),
        current: latest_ver.to_string(),
        path: current_exe.to_path_buf(),
    })
}

fn run_command(program: &str, args: &[&str]) -> Result<(), UpgradeError> {
    let status = Command::new(program).args(args).status()?;
    if !status.success() {
        return Err(UpgradeError::NetworkError(format!(
            "command exited with status: {:?}",
            status.code()
        )));
    }
    Ok(())
}

fn replace_binary(
    new_binary: &std::path::Path,
    current: &std::path::Path,
) -> Result<(), UpgradeError> {
    let dir = current.parent().ok_or_else(|| {
        UpgradeError::IoError(io::Error::new(
            io::ErrorKind::NotFound,
            "cannot determine binary directory",
        ))
    })?;
    let tmp = dir.join(".codemux-upgrade-tmp");

    std::fs::copy(new_binary, &tmp)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
    }

    std::fs::rename(&tmp, current).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        if e.kind() == io::ErrorKind::PermissionDenied {
            UpgradeError::PermissionDenied(current.to_path_buf())
        } else {
            UpgradeError::IoError(e)
        }
    })?;

    Ok(())
}

fn verify_version(binary: &std::path::Path, expected: &str) -> Result<(), UpgradeError> {
    let output = Command::new(binary)
        .arg("--version")
        .output()
        .map_err(|e| {
            UpgradeError::IoError(io::Error::new(
                e.kind(),
                format!("failed to run downloaded binary: {}", e),
            ))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected_output = format!("codemux {}", expected);
    if !output.status.success() || stdout.trim() != expected_output {
        return Err(UpgradeError::VerificationError(format!(
            "expected '{}', got '{}' (status: {})",
            expected_output,
            stdout.trim(),
            output.status
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    fn write_test_executable(path: &std::path::Path, contents: &str) {
        use std::os::unix::fs::PermissionsExt;

        std::fs::write(path, contents).expect("write test executable");
        let mut permissions = std::fs::metadata(path)
            .expect("read test executable metadata")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).expect("make test executable executable");
    }

    #[test]
    fn test_parse_version_strips_prerelease() {
        assert_eq!(parse_version("v1.2.3-rc1"), Some((1, 2, 3)));
        assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("1.2.3-beta.4"), Some((1, 2, 3)));
        assert_eq!(parse_version("v0.10.0-alpha+001"), Some((0, 10, 0)));
    }

    #[test]
    fn test_parse_version_rejects_malformed() {
        assert_eq!(parse_version("v1.2"), None);
        assert_eq!(parse_version("not-a-version"), None);
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn test_version_comparison_follows_prerelease_precedence() {
        use std::cmp::Ordering;

        assert_eq!(version_cmp("v1.2.3", "v1.2.3-rc1"), Ordering::Greater);
        assert_eq!(version_cmp("v1.2.3-rc2", "v1.2.3-rc1"), Ordering::Greater);
        assert_eq!(version_cmp("v1.2.3-1", "v1.2.3-alpha"), Ordering::Less);
        assert_eq!(
            version_cmp("v1.2.3+build.2", "v1.2.3+build.1"),
            Ordering::Equal
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_is_executable_checks_permission_bits() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().expect("create temporary directory");
        let path = temp.path().join("curl");
        std::fs::write(&path, "not executable").expect("write test file");
        assert!(!is_executable(&path));

        let mut permissions = std::fs::metadata(&path)
            .expect("read test file metadata")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&path, permissions).expect("make test file executable");
        assert!(is_executable(&path));
    }

    #[cfg(unix)]
    #[test]
    fn test_verify_version_requires_exact_successful_output() {
        let temp = tempfile::tempdir().expect("create temporary directory");
        let binary = temp.path().join("codemux");

        write_test_executable(&binary, "#!/bin/sh\necho 'codemux 1.2.30'\n");
        assert!(matches!(
            verify_version(&binary, "1.2.3"),
            Err(UpgradeError::VerificationError(_))
        ));

        write_test_executable(&binary, "#!/bin/sh\necho 'codemux 1.2.3'\nexit 1\n");
        assert!(matches!(
            verify_version(&binary, "1.2.3"),
            Err(UpgradeError::VerificationError(_))
        ));

        write_test_executable(&binary, "#!/bin/sh\necho 'codemux 1.2.3'\n");
        assert!(verify_version(&binary, "1.2.3").is_ok());
    }
}
