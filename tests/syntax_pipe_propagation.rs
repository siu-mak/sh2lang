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
