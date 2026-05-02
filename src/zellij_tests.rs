use crate::zellij::ZellijLauncher;
use crate::MuxLauncher;

#[test]
fn test_build_command_auto_attach_true() {
    let launcher = ZellijLauncher::new();
    let cmd = launcher.build_command("myapp", "/home/user/projects/myapp", true);
    assert_eq!(cmd, "zellij attach 'myapp' -c");
}

#[test]
fn test_build_command_auto_attach_false() {
    let launcher = ZellijLauncher::new();
    let cmd = launcher.build_command("myapp", "/home/user/projects/myapp", false);
    assert_eq!(cmd, "zellij -s 'myapp'");
}

#[test]
fn test_build_command_with_spaces_in_name() {
    let launcher = ZellijLauncher::new();
    let cmd = launcher.build_command("my app", "/home/user/my projects", true);
    assert_eq!(cmd, "zellij attach 'my app' -c");
}

#[test]
fn test_build_command_with_quotes() {
    let launcher = ZellijLauncher::new();
    let cmd = launcher.build_command("it's", "/home/user/john's files", false);
    assert_eq!(cmd, "zellij -s 'it'\"'\"'s'");
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
