use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

struct Fixture {
    _temp: TempDir,
    bin_dir: PathBuf,
    config_home: PathBuf,
    workspace: PathBuf,
}

#[cfg(unix)]
impl Fixture {
    fn new(workspace_name: &str) -> Self {
        let temp = tempfile::tempdir().expect("create temporary test directory");
        let root = temp.path().to_path_buf();
        let bin_dir = root.join("bin");
        let config_home = root.join("config");
        let workspace = root.join(workspace_name);
        fs::create_dir_all(&bin_dir).expect("create mock binary directory");
        fs::create_dir_all(&config_home).expect("create config directory");
        fs::create_dir_all(&workspace).expect("create workspace directory");

        let fixture = Self {
            _temp: temp,
            bin_dir,
            config_home,
            workspace,
        };
        fixture.write_executable(
            "mock-shell",
            "#!/bin/sh\nprintf '[mock-shell] pid=%s\\n' \"$$\" >&2\nif [ \"$1\" = \"-l\" ]; then shift; fi\nexec /bin/sh \"$@\"\n",
        );
        fixture
    }

    fn write_executable(&self, name: &str, contents: &str) {
        let path = self.bin_dir.join(name);
        fs::write(&path, contents).expect("write mock executable");
        let mut permissions = fs::metadata(&path)
            .expect("read mock metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("make mock executable");
    }

    /// Creates a mock `curl` that returns a fake GitHub API releases JSON response.
    /// The mock responds with the given `tag_name` when any argument contains
    /// "releases/latest", and fails otherwise.
    fn add_mock_curl(&self, tag_name: &str) {
        let script = format!(
            r#"#!/bin/sh
# Mock curl for codemux upgrade tests
case "$*" in
    *releases/latest*)
        printf '{{"tag_name": "{}"}}\n'
        exit 0
        ;;
    *)
        echo "mock-curl: unexpected args: $*" >&2
        exit 1
        ;;
esac
"#,
            tag_name
        );
        self.write_executable("curl", &script);
    }

    fn add_mux(&self, name: &str) {
        self.write_executable(
            name,
            "#!/bin/sh\nif [ \"$1\" = \"list-sessions\" ]; then\n    if [ -n \"$MOCK_SESSIONS\" ]; then\n        printf '%s\\n' \"$MOCK_SESSIONS\"\n    fi\n    exit 0\nfi\nprintf 'executed:%s:%s\\n' \"${0##*/}\" \"$*\"\n",
        );
    }

    fn write_config(&self, contents: &str) {
        let directory = self.config_home.join("codemux");
        fs::create_dir_all(&directory).expect("create codemux config directory");
        fs::write(directory.join("config.toml"), contents).expect("write config");
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_codemux"));
        command
            .current_dir(&self.workspace)
            .env("PATH", &self.bin_dir)
            .env("SHELL", self.bin_dir.join("mock-shell"))
            .env("XDG_CONFIG_HOME", &self.config_home)
            .env_remove("CODEMUX_MULTIPLEXER")
            .env_remove("CODEMUX_AUTO_ATTACH")
            .env_remove("CODEMUX_DEBUG")
            .env_remove("TMUX")
            .env_remove("ZELLIJ");
        command
    }
}

#[cfg(unix)]
fn run(mut command: Command) -> Output {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let child = command.spawn().expect("spawn codemux");
    let codemux_pid = child.id();
    let output = child.wait_with_output().expect("wait for codemux");
    assert!(
        output.status.success(),
        "codemux failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        text(&output.stderr).contains(&format!("[mock-shell] pid={codemux_pid}")),
        "the shell did not replace the codemux process"
    );
    output
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(unix)]
#[test]
fn env_override_runs_sanitized_uniquely_suffixed_tmux_command() {
    let fixture = Fixture::new("My Project!!");
    fixture.add_mux("tmux");
    fixture.write_config("multiplexer = \"zellij\"\nauto_attach = true\n");

    let mut command = fixture.command();
    command
        .env("CODEMUX_MULTIPLEXER", "tmux")
        .env("CODEMUX_AUTO_ATTACH", "0")
        .env("CODEMUX_DEBUG", "1")
        .env("MOCK_SESSIONS", "My-Project");
    let output = run(command);
    let stderr = text(&output.stderr);
    let stdout = text(&output.stdout);

    assert!(stderr.contains("[codemux] Resolved multiplexer: Some(Tmux)"));
    assert!(stderr.contains("[codemux] Sanitized name: My-Project"));
    assert!(stderr.contains("[codemux] Auto attach: false"));
    assert!(stderr.contains("[codemux] Final session name: My-Project-2"));
    assert!(stdout.contains("executed:tmux:new-session -s My-Project-2 -c"));
    assert!(stdout.contains(fixture.workspace.to_string_lossy().as_ref()));
}

#[cfg(unix)]
#[test]
fn config_override_selects_zellij_and_reports_ignored_cwd() {
    let fixture = Fixture::new("config-priority");
    fixture.add_mux("tmux");
    fixture.add_mux("zellij");
    fixture.write_config("multiplexer = \"zellij\"\nauto_attach = false\n");

    let mut command = fixture.command();
    command.env("CODEMUX_DEBUG", "1");
    let output = run(command);
    let stderr = text(&output.stderr);

    assert!(stderr.contains("[codemux] Resolved multiplexer: Some(Zellij)"));
    assert!(stderr.contains("zellij cannot set CWD in non-auto-attach mode"));
    assert!(text(&output.stdout).contains("executed:zellij:-s config-priority"));
}

#[cfg(unix)]
#[test]
fn path_detection_prefers_tmux() {
    let fixture = Fixture::new("path-priority");
    fixture.add_mux("tmux");
    fixture.add_mux("zellij");

    let mut command = fixture.command();
    command.env("CODEMUX_DEBUG", "1");
    let output = run(command);

    assert!(text(&output.stderr).contains("[codemux] Resolved multiplexer: Some(Tmux)"));
    assert!(!text(&output.stderr).contains("config file exists but could not be read"));
    assert!(text(&output.stdout).contains("executed:tmux:new-session -A"));
}

#[cfg(unix)]
#[test]
fn unreadable_existing_config_warns_and_uses_defaults() {
    let fixture = Fixture::new("unreadable-config");
    fixture.add_mux("tmux");
    let config_path = fixture.config_home.join("codemux/config.toml");
    fs::create_dir_all(&config_path).expect("create directory in place of config file");

    let output = run(fixture.command());
    let stderr = text(&output.stderr);

    assert!(stderr.contains("[codemux] Warning: config file exists but could not be read:"));
    assert!(text(&output.stdout).contains("executed:tmux:new-session -A"));
}

#[test]
fn version_flag_prints_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_codemux"))
        .args(["--version"])
        .output()
        .expect("spawn codemux");
    assert!(output.status.success());
    let stdout = text(&output.stdout);
    assert!(stdout.starts_with("codemux "));
}

#[cfg(unix)]
#[test]
fn check_version_flag_prints_latest_version() {
    let fixture = Fixture::new("check-version-test");
    fixture.add_mock_curl("v99.0.0");

    let output = fixture
        .command()
        .args(["--check-version"])
        .output()
        .expect("spawn codemux");

    assert!(
        output.status.success(),
        "codemux failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = text(&output.stdout);
    assert!(
        stdout.contains("Latest version: v99.0.0"),
        "expected 'Latest version: v99.0.0' in stdout, got: {}",
        stdout
    );
}

#[cfg(unix)]
#[test]
fn upgrade_check_flag_reports_update_available() {
    let fixture = Fixture::new("upgrade-check-test");
    fixture.add_mock_curl("v99.0.0");

    let output = fixture
        .command()
        .args(["--upgrade", "--check"])
        .output()
        .expect("spawn codemux");

    assert!(
        output.status.success(),
        "codemux failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = text(&output.stdout);
    assert!(
        stdout.contains("Latest version: v99.0.0"),
        "expected 'Latest version: v99.0.0' in stdout, got: {}",
        stdout
    );
}

#[cfg(unix)]
#[test]
fn upgrade_flag_no_curl_reports_error() {
    let fixture = Fixture::new("no-curl-test");
    // Don't add curl — it won't be on PATH

    let output = fixture
        .command()
        .args(["--upgrade", "--check"])
        .output()
        .expect("spawn codemux");

    assert!(!output.status.success());
    let stderr = text(&output.stderr);
    assert!(
        stderr.contains("codemux: upgrade requires curl"),
        "expected error about missing curl, got: {}",
        stderr
    );
}

#[cfg(unix)]
#[test]
fn check_version_flag_no_curl_reports_error() {
    let fixture = Fixture::new("check-ver-no-curl");
    // Don't add curl — it won't be on PATH

    let output = fixture
        .command()
        .args(["--check-version"])
        .output()
        .expect("spawn codemux");

    assert!(!output.status.success());
    let stderr = text(&output.stderr);
    assert!(
        stderr.contains("codemux: upgrade requires curl"),
        "expected error about missing curl, got: {}",
        stderr
    );
}

#[cfg(unix)]
#[test]
fn upgrade_check_flag_already_latest_reports_up_to_date() {
    let fixture = Fixture::new("already-latest-test");
    // Mock curl returns a version lower than current (0.3.0)
    fixture.add_mock_curl("v0.2.0");

    let output = fixture
        .command()
        .args(["--upgrade", "--check"])
        .output()
        .expect("spawn codemux");

    assert!(!output.status.success());
    let stderr = text(&output.stderr);
    assert!(
        stderr.contains("already up to date"),
        "expected 'already up to date' error, got: {}",
        stderr
    );
}

#[cfg(unix)]
#[test]
fn upgrade_check_flag_malformed_json_reports_parse_error() {
    let fixture = Fixture::new("malformed-json-test");
    // Mock curl returns JSON without a tag_name key
    fixture.write_executable(
        "curl",
        "#!/bin/sh\ncase \"$*\" in\n    *releases/latest*)\n        printf '{\"not_tag_name\": \"v1.0.0\"}\\n'\n        exit 0\n        ;;\n    *)\n        echo \"mock-curl: unexpected args: $*\" >&2\n        exit 1\n        ;;\nesac\n",
    );

    let output = fixture
        .command()
        .args(["--upgrade", "--check"])
        .output()
        .expect("spawn codemux");

    assert!(!output.status.success());
    let stderr = text(&output.stderr);
    assert!(
        stderr.contains("failed to parse version"),
        "expected 'failed to parse version' error, got: {}",
        stderr
    );
}

#[cfg(unix)]
#[test]
fn upgrade_check_with_yes_flag_works() {
    let fixture = Fixture::new("upgrade-check-yes-test");
    fixture.add_mock_curl("v99.0.0");

    let output = fixture
        .command()
        .args(["--upgrade", "--check", "--yes"])
        .output()
        .expect("spawn codemux");

    assert!(
        output.status.success(),
        "codemux failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = text(&output.stdout);
    assert!(
        stdout.contains("Latest version: v99.0.0"),
        "expected 'Latest version: v99.0.0' in stdout, got: {}",
        stdout
    );
}
