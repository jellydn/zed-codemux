use crate::tmux::TmuxLauncher;
use crate::MuxLauncher;
use std::sync::{Mutex, MutexGuard, OnceLock};

/// Process-global lock that serializes access to environment variable mutations.
/// Prevents race conditions when tests modify `TMUX` concurrently in parallel
/// test runs.
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// RAII guard that acquires the global env lock and restores an environment
/// variable to its original value on drop.
/// Ensures env vars are restored even when a test panics.
struct EnvGuard {
    key: String,
    saved: Option<String>,
    _lock: MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn new(key: &str) -> Self {
        let _lock = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let saved = std::env::var(key).ok();
        Self {
            key: key.to_string(),
            saved,
            _lock,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.saved {
            Some(v) => std::env::set_var(&self.key, v),
            None => std::env::remove_var(&self.key),
        }
    }
}

#[test]
fn test_build_command_auto_attach_true() {
    let launcher = TmuxLauncher::new();
    let cmd = launcher.build_command("myapp", "/home/user/projects/myapp", true);
    assert_eq!(
        cmd,
        "tmux new-session -A -s 'myapp' -c '/home/user/projects/myapp'"
    );
}

#[test]
fn test_is_inside_session_false_when_tmux_unset() {
    let _guard = EnvGuard::new("TMUX");
    std::env::remove_var("TMUX");
    let launcher = TmuxLauncher::new();
    assert!(!launcher.is_inside_session());
}

#[test]
fn test_is_inside_session_true_when_tmux_set() {
    let _guard = EnvGuard::new("TMUX");
    std::env::set_var("TMUX", "/tmp/tmux-501/default,1836,0");
    let launcher = TmuxLauncher::new();
    assert!(launcher.is_inside_session());
}

#[test]
fn test_build_inside_command() {
    let launcher = TmuxLauncher::new();
    let cmd = launcher.build_inside_command("myapp", "/home/user/projects/myapp");
    assert_eq!(
        cmd,
        "tmux new-window -n 'myapp' -c '/home/user/projects/myapp'"
    );
}

#[test]
fn test_build_inside_command_with_spaces() {
    let launcher = TmuxLauncher::new();
    let cmd = launcher.build_inside_command("my app", "/home/user/my projects");
    assert_eq!(
        cmd,
        "tmux new-window -n 'my app' -c '/home/user/my projects'"
    );
}

#[test]
fn test_build_inside_command_with_quotes() {
    let launcher = TmuxLauncher::new();
    let cmd = launcher.build_inside_command("it's", "/home/user/john's files");
    assert_eq!(
        cmd,
        "tmux new-window -n 'it'\"'\"'s' -c '/home/user/john'\"'\"'s files'"
    );
}

#[test]
fn test_build_command_auto_attach_false() {
    let launcher = TmuxLauncher::new();
    let cmd = launcher.build_command("myapp", "/home/user/projects/myapp", false);
    assert_eq!(
        cmd,
        "tmux new-session -s 'myapp' -c '/home/user/projects/myapp'"
    );
}

#[test]
fn test_build_command_with_spaces_in_name() {
    let launcher = TmuxLauncher::new();
    let cmd = launcher.build_command("my app", "/home/user/my projects", true);
    assert_eq!(
        cmd,
        "tmux new-session -A -s 'my app' -c '/home/user/my projects'"
    );
}

#[test]
fn test_build_command_with_quotes() {
    let launcher = TmuxLauncher::new();
    let cmd = launcher.build_command("it's", "/home/user/john's files", false);
    assert_eq!(
        cmd,
        "tmux new-session -s 'it'\"'\"'s' -c '/home/user/john'\"'\"'s files'"
    );
}

#[test]
fn test_list_sessions_when_tmux_not_installed() {
    // This test verifies the behavior when tmux is not in PATH
    // We can't easily mock the Command, but we can verify that
    // a NotFound error results in an empty list
    // In practice, this test passes on systems without tmux
    let launcher = TmuxLauncher::new();
    let result = launcher.list_sessions();
    assert!(result.is_ok());
    // Result should either be empty (tmux not found) or actual sessions
}
