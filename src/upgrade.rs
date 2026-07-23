use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
    /// curl (or PowerShell on Windows) was not found on PATH.
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

fn find_curl() -> Result<String, UpgradeError> {
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
    #[cfg(windows)]
    {
        if let Ok(path) = which_powershell() {
            return Ok(path);
        }
    }
    Err(UpgradeError::CurlNotFound)
}

fn which(name: &str) -> Result<String, ()> {
    let path_env = std::env::var("PATH").unwrap_or_default();
    let path_sep = if cfg!(windows) { ';' } else { ':' };
    for dir in path_env.split(path_sep) {
        let full = std::path::Path::new(dir).join(name);
        if full.is_file() {
            return Ok(full.to_string_lossy().into_owned());
        }
    }
    Err(())
}

#[cfg(windows)]
fn which_powershell() -> Result<String, ()> {
    for name in &["pwsh.exe", "powershell.exe"] {
        if let Ok(path) = which(name) {
            return Ok(path);
        }
    }
    Err(())
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

/// Parses a version string like `"1.2.3"` or `"v1.2.3"` into a `(major, minor, patch)` tuple.
/// Strips any prerelease suffix (e.g. `"v1.2.3-rc1"` parses as `(1, 2, 3)`).
pub fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.strip_prefix('v').unwrap_or(s);
    let mut parts: Vec<&str> = s.splitn(3, '.').collect();
    if parts.len() != 3 {
        return None;
    }
    // Strip any suffix from the patch component (e.g. "3-rc1" → "3")
    let patch_str = parts[2];
    let digit_end = patch_str
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(patch_str.len());
    parts[2] = &patch_str[..digit_end];
    if parts[2].is_empty() {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

/// Compares two version strings (with optional "v" prefix).
/// Falls back to lexical comparison when either version cannot be parsed.
fn version_cmp(latest: &str, current: &str) -> std::cmp::Ordering {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => l.cmp(&c),
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
    std::io::stdin().read_line(&mut input).ok();
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
pub fn upgrade(check_only: bool, yes: bool) -> Result<UpgradeResult, UpgradeError> {
    #[cfg(windows)]
    {
        return Err(UpgradeError::WindowsNotSupported);
    }

    let latest_tag = check_latest()?;
    let latest_ver = latest_tag.strip_prefix('v').unwrap_or(&latest_tag);

    if version_cmp(&latest_tag, &format!("v{}", VERSION)) != std::cmp::Ordering::Greater {
        return Err(UpgradeError::AlreadyLatest {
            current: VERSION.to_string(),
        });
    }

    if check_only {
        println!("Latest version: v{} (current: v{})", latest_ver, VERSION);
        return Ok(UpgradeResult {
            previous: VERSION.to_string(),
            current: latest_ver.to_string(),
            path: std::env::current_exe().unwrap_or_default(),
        });
    }

    let method = detect_install_method();

    match method {
        InstallMethod::Cargo => {
            handle_external_upgrade("cargo install codemux --force", "cargo", latest_ver, yes)
        }
        InstallMethod::Homebrew => {
            handle_external_upgrade("brew upgrade codemux", "Homebrew", latest_ver, yes)
        }
        InstallMethod::Prebuilt => {
            let current_exe = std::env::current_exe().map_err(UpgradeError::IoError)?;
            perform_prebuilt_upgrade(&latest_tag, latest_ver, &current_exe)
        }
    }
}

/// Prompts the user and optionally runs an external package-manager upgrade command.
fn handle_external_upgrade(
    cmd: &str,
    label: &str,
    latest_ver: &str,
    yes: bool,
) -> Result<UpgradeResult, UpgradeError> {
    println!("Detected {} installation.", label);
    println!("Recommended command: {}", cmd);
    if yes || prompt_yes_no("Run this command?") {
        run_command(cmd)?;
    } else {
        println!("Upgrade cancelled.");
    }
    Ok(UpgradeResult {
        previous: VERSION.to_string(),
        current: latest_ver.to_string(),
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

    let tmp_dir = std::env::temp_dir().join(format!("codemux-upgrade-{}", std::process::id()));
    std::fs::create_dir_all(&tmp_dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_dir, std::fs::Permissions::from_mode(0o700))?;
    }

    // Ensure the temp directory is always cleaned up, even on error.
    let result = do_prebuilt_upgrade(latest_tag, latest_ver, current_exe, asset, &tmp_dir);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    result
}

/// Core prebuilt upgrade logic (download, extract, replace, verify).
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

    replace_binary(&extracted_binary, current_exe)?;
    verify_version(current_exe, latest_ver)?;

    println!("codemux: upgraded v{} → v{} ✓", VERSION, latest_ver);

    Ok(UpgradeResult {
        previous: VERSION.to_string(),
        current: latest_ver.to_string(),
        path: current_exe.to_path_buf(),
    })
}

fn run_command(cmd: &str) -> Result<(), UpgradeError> {
    let mut parts = cmd.split_whitespace();
    let program = parts.next().unwrap_or(cmd);
    let args: Vec<&str> = parts.collect();
    let status = Command::new(program).args(&args).status()?;
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
        .map_err(|_| {
            UpgradeError::IoError(io::Error::new(
                io::ErrorKind::Other,
                "failed to run updated binary",
            ))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains(expected) {
        return Err(UpgradeError::NetworkError(format!(
            "version mismatch after upgrade: expected {}, got {}",
            expected,
            stdout.trim()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
