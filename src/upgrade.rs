use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq)]
pub struct UpgradeResult {
    pub previous: String,
    pub current: String,
    pub path: PathBuf,
}

#[derive(Debug)]
pub enum UpgradeError {
    CurlNotFound,
    NetworkError(String),
    PermissionDenied(PathBuf),
    UnsupportedPlatform {
        os: &'static str,
        arch: &'static str,
    },
    AlreadyLatest {
        current: String,
    },
    ParseError(String),
    IoError(io::Error),
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
    let curl_candidates = if cfg!(windows) {
        ["curl.exe", "curl.cmd"]
    } else {
        ["curl"]
    };
    for name in &curl_candidates {
        if let Ok(path) = which(name) {
            return Ok(path);
        }
    }
    if cfg!(windows) {
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

fn parse_tag_name(json: &str) -> Option<String> {
    let needle = "\"tag_name\":\"";
    let start = json.find(needle)?;
    let value_start = start + needle.len();
    let value_end = json[value_start..].find('\"')?;
    Some(json[value_start..value_start + value_end].to_string())
}

pub fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.strip_prefix('v').unwrap_or(s);
    let parts: Vec<&str> = s.splitn(3, '.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

fn version_cmp(latest: &str, current: &str) -> std::cmp::Ordering {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => l.cmp(&c),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => latest.cmp(current),
    }
}

#[derive(Debug, PartialEq)]
pub enum InstallMethod {
    Cargo,
    Homebrew,
    Prebuilt,
}

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

pub fn check_latest() -> Result<String, UpgradeError> {
    let curl = find_curl()?;

    let mut cmd = Command::new(&curl);
    cmd.args([
        "-sL",
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

pub fn check_version_only() -> Result<String, UpgradeError> {
    let latest = check_latest()?;
    Ok(latest.strip_prefix('v').unwrap_or(&latest).to_string())
}

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
            let cmd = "cargo install codemux --force";
            println!("Detected cargo installation.");
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
        InstallMethod::Homebrew => {
            let cmd = "brew upgrade codemux";
            println!("Detected Homebrew installation.");
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
        InstallMethod::Prebuilt => {
            let current_exe = std::env::current_exe().map_err(UpgradeError::IoError)?;

            let asset = platform_asset_name()?;

            if debug_enabled() {
                eprintln!("[codemux] Downloading asset: {}", asset);
            }

            let tmp_dir = std::env::temp_dir().join("codemux-upgrade");
            std::fs::create_dir_all(&tmp_dir)?;

            let archive_path = tmp_dir.join(asset);
            let download_url = format!(
                "https://github.com/jellydn/zed-codemux/releases/download/{}/{}",
                latest_tag, asset
            );

            let curl = find_curl()?;
            let status = Command::new(&curl)
                .args(["-sL", "-o", &archive_path.to_string_lossy(), &download_url])
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

            replace_binary(&extracted_binary, &current_exe)?;

            let _ = std::fs::remove_dir_all(&tmp_dir);

            verify_version(&current_exe, latest_ver)?;

            println!("codemux: upgraded v{} → v{} ✓", VERSION, latest_ver);

            Ok(UpgradeResult {
                previous: VERSION.to_string(),
                current: latest_ver.to_string(),
                path: current_exe,
            })
        }
    }
}

fn run_command(cmd: &str) -> Result<(), UpgradeError> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let status = Command::new(&shell).args(["-c", cmd]).status()?;
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
