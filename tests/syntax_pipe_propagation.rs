use crate::common::*;

mod common;

#[test]
fn exec_pipe_fail_middle() {
    assert_exec_matches_fixture_target("pipe_fail_middle", TargetShell::Bash);
    assert_exec_matches_fixture_target("pipe_fail_middle", TargetShell::Posix);
}

#[test]
fn exec_pipe_fail_first() {
    assert_exec_matches_fixture_target("pipe_fail_first", TargetShell::Bash);
    assert_exec_matches_fixture_target("pipe_fail_first", TargetShell::Posix);
}

#[test]
fn exec_pipe_two_fail_rightmost_wins() {
    assert_exec_matches_fixture_target("pipe_two_fail_rightmost_wins", TargetShell::Bash);
    assert_exec_matches_fixture_target("pipe_two_fail_rightmost_wins", TargetShell::Posix);
}

#[test]
fn exec_pipe_allow_fail_middle_ignored() {
    assert_exec_matches_fixture_target("pipe_allow_fail_middle_ignored", TargetShell::Bash);
    assert_exec_matches_fixture_target("pipe_allow_fail_middle_ignored", TargetShell::Posix);
}

#[test]
fn exec_pipeblocks_fail_middle() {
    assert_exec_matches_fixture_target("pipeblocks_fail_middle", TargetShell::Bash);
    assert_exec_matches_fixture_target("pipeblocks_fail_middle", TargetShell::Posix);
}

/// Test that POSIX pipelines work correctly even when the shell has `set -e` enabled.
/// The pipeline helper must shield errexit during waits/status collection.
/// We only test POSIX with -e flag because Bash doesn't have errexit enabled by default.
#[test]
fn exec_pipe_posix_errexit_safe() {
    // Posix target: run with -e flag to verify errexit-safe behavior
    // The pipeline itself should succeed (wait + collect status), and then
    // (exit $__sh2_status) should trigger errexit abort, preventing UNREACHABLE.
    assert_exec_matches_fixture_target_with_flags("pipe_posix_errexit_safe", TargetShell::Posix, &["-e"]);
}

