mod common;
use common::*;

#[test]
fn exec_runtime_error_loc_bash() {
    assert_exec_matches_fixture("runtime_error_loc");
}

#[test]
fn exec_runtime_error_loc_posix() {
    assert_exec_matches_fixture_target("runtime_error_loc", TargetShell::Posix);
}
#[test]
fn exec_posix_runtime_error() {
    assert_exec_matches_fixture_target("posix_runtime_error", TargetShell::Posix);
}

#[test]
fn exec_cmd_sub_pipe() {
    assert_exec_matches_fixture("cmd_sub_pipe");
}

#[test]
fn exec_runtime_fail_fast_bash() {
    // Test that failing commands stop execution and print diagnostics
    assert_exec_matches_fixture("runtime_fail_fast");
}

#[test]
fn exec_runtime_fail_fast_posix() {
    // Test that failing commands stop execution in POSIX mode too
    assert_exec_matches_fixture_target("runtime_fail_fast", TargetShell::Posix);
}
