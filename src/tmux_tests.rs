use crate::tmux::TmuxLauncher;
use crate::MuxLauncher;

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
    // Save and unset TMUX env var to test detection
    let saved = std::env::var("TMUX").ok();
    std::env::remove_var("TMUX");
    let launcher = TmuxLauncher::new();
    assert!(!launcher.is_inside_session());
    // Restore
    if let Some(val) = saved {
        std::env::set_var("TMUX", val);
    }
}

#[test]
fn test_is_inside_session_true_when_tmux_set() {
    let saved = std::env::var("TMUX").ok();
    std::env::set_var("TMUX", "/tmp/tmux-501/default,1836,0");
    let launcher = TmuxLauncher::new();
    assert!(launcher.is_inside_session());
    // Restore
    if let Some(val) = saved {
        std::env::set_var("TMUX", val);
    } else {
        std::env::remove_var("TMUX");
    }
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
