mod common;
use common::*;

#[test]
fn codegen_try_run_captures_bash() {
    assert_codegen_matches_snapshot("try_run_captures");
}

#[test]
fn codegen_try_run_captures_posix() {
    assert_codegen_matches_snapshot_target("try_run_captures", TargetShell::Posix);
}

#[test]
fn exec_try_run_captures_bash() {
    assert_exec_matches_fixture_target("try_run_captures", TargetShell::Bash);
}

#[test]
fn exec_try_run_captures_posix() {
    assert_exec_matches_fixture_target("try_run_captures", TargetShell::Posix);
}

#[test]
fn codegen_try_run_success_bash() {
    assert_codegen_matches_snapshot("try_run_success");
}

#[test]
fn codegen_try_run_success_posix() {
    assert_codegen_matches_snapshot_target("try_run_success", TargetShell::Posix);
}

#[test]
fn exec_try_run_success_bash() {
    assert_exec_matches_fixture_target("try_run_success", TargetShell::Bash);
}

#[test]
fn exec_try_run_success_posix() {
    assert_exec_matches_fixture_target("try_run_success", TargetShell::Posix);
}

#[test]
fn compile_try_run_stmt_invalid() {
    assert_codegen_panics("try_run_stmt_invalid", "try_run() must be bound via let");
}
