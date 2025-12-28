mod common;
use common::*;
use sh2c::codegen::TargetShell;

#[test]
fn exec_heredoc_basic() {
    assert_codegen_matches_snapshot_target("heredoc_basic", TargetShell::Bash);
    assert_codegen_matches_snapshot_target("heredoc_basic", TargetShell::Posix);
    assert_exec_matches_fixture_target("heredoc_basic", TargetShell::Bash);
    assert_exec_matches_fixture_target("heredoc_basic", TargetShell::Posix);
}

#[test]
fn exec_heredoc_no_trailing_newline() {
    // Verifies codegen appends newline if missing
    assert_codegen_matches_snapshot_target("heredoc_no_trailing_newline", TargetShell::Bash);
    assert_codegen_matches_snapshot_target("heredoc_no_trailing_newline", TargetShell::Posix);
    assert_exec_matches_fixture_target("heredoc_no_trailing_newline", TargetShell::Bash);
    assert_exec_matches_fixture_target("heredoc_no_trailing_newline", TargetShell::Posix);
}

#[test]
fn exec_heredoc_literal_no_expansion() {
    // Verifies strictly no variable expansion
    assert_exec_matches_fixture_target("heredoc_literal_no_expansion", TargetShell::Bash);
    assert_exec_matches_fixture_target("heredoc_literal_no_expansion", TargetShell::Posix);
}

#[test]
fn exec_heredoc_delim_collision() {
    // Verifies delimiter collision avoidance logic
    assert_codegen_matches_snapshot_target("heredoc_delim_collision", TargetShell::Bash);
    assert_codegen_matches_snapshot_target("heredoc_delim_collision", TargetShell::Posix);
    assert_exec_matches_fixture_target("heredoc_delim_collision", TargetShell::Bash);
    assert_exec_matches_fixture_target("heredoc_delim_collision", TargetShell::Posix);
}

#[test]
fn compile_panic_heredoc_stdout_rejected_bash() {
    assert_codegen_panics_target("heredoc_stdout_rejected", TargetShell::Bash, "heredoc only allowed for stdin");
}

#[test]
fn compile_panic_heredoc_stdout_rejected_posix() {
    assert_codegen_panics_target("heredoc_stdout_rejected", TargetShell::Posix, "heredoc only allowed for stdin");
}

#[test]
fn compile_panic_heredoc_stderr_rejected_bash() {
    assert_codegen_panics_target("heredoc_stderr_rejected", TargetShell::Bash, "heredoc only allowed for stdin");
}

#[test]
fn compile_panic_heredoc_stderr_rejected_posix() {
    assert_codegen_panics_target("heredoc_stderr_rejected", TargetShell::Posix, "heredoc only allowed for stdin");
}