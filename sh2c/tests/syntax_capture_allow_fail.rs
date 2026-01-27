use common::{assert_exec_matches_fixture_target, TargetShell};

mod common;

#[test]
fn test_capture_allow_fail_bash() {
    assert_exec_matches_fixture_target("capture_allow_fail_basic", TargetShell::Bash);
}

#[test]
fn test_capture_allow_fail_posix() {
    assert_exec_matches_fixture_target("capture_allow_fail_basic", TargetShell::Posix);
}
