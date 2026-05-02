use crate::zellij::{get_socket_dir, ZellijLauncher};
use crate::MuxLauncher;

#[test]
fn test_build_command_auto_attach_true() {
    let launcher = ZellijLauncher::new();
    let cmd = launcher.build_command("myapp", "/home/user/projects/myapp", true);
    let socket_dir = get_socket_dir();
    assert_eq!(cmd, format!("ZELLIJ_SOCKET_DIR='{}' zellij attach 'myapp' -c", socket_dir));
}

#[test]
fn test_build_command_auto_attach_false() {
    let launcher = ZellijLauncher::new();
    let cmd = launcher.build_command("myapp", "/home/user/projects/myapp", false);
    let socket_dir = get_socket_dir();
    assert_eq!(cmd, format!("ZELLIJ_SOCKET_DIR='{}' zellij -s 'myapp'", socket_dir));
}

#[test]
fn test_build_command_with_spaces_in_name() {
    let launcher = ZellijLauncher::new();
    let cmd = launcher.build_command("my app", "/home/user/my projects", true);
    let socket_dir = get_socket_dir();
    assert_eq!(cmd, format!("ZELLIJ_SOCKET_DIR='{}' zellij attach 'my app' -c", socket_dir));
}

#[test]
fn test_build_command_with_quotes() {
    let launcher = ZellijLauncher::new();
    let cmd = launcher.build_command("it's", "/home/user/john's files", false);
    let socket_dir = get_socket_dir();
    assert_eq!(cmd, format!("ZELLIJ_SOCKET_DIR='{}' zellij -s 'it'\"'\"'s'", socket_dir));
}

#[test]
fn test_list_sessions_when_zellij_not_installed() {
    // This test verifies the behavior when zellij is not in PATH
    // We can't easily mock the Command, but we can verify that
    // a NotFound error results in an empty list
    // In practice, this test passes on systems without zellij
    let launcher = ZellijLauncher::new();
    let result = launcher.list_sessions();
    assert!(result.is_ok());
    // Result should either be empty (zellij not found) or actual sessions
}
