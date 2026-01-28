use common::{assert_exec_matches_fixture_target, TargetShell};

mod common;

#[test]
fn test_capture_allow_fail_status() {
    assert_exec_matches_fixture_target("capture_allow_fail_preserves_status", TargetShell::Bash);
}
