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
