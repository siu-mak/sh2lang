use common::{assert_exec_matches_fixture_target, TargetShell};

mod common;

#[test]
fn test_capture_nested_named_arg_allow_fail() {
    assert_exec_matches_fixture_target("capture_nested_named_arg_allow_fail", TargetShell::Bash);
}
