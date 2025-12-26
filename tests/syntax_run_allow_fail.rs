use crate::common::*;

mod common;

#[test]
fn exec_run_allow_fail_try() {
    assert_exec_matches_fixture_target("run_allow_fail_try", TargetShell::Bash);
    assert_exec_matches_fixture_target("run_allow_fail_try", TargetShell::Posix);
}

#[test]
fn exec_run_disallow_fail_try() {
    assert_exec_matches_fixture_target("run_disallow_fail_try", TargetShell::Bash);
    assert_exec_matches_fixture_target("run_disallow_fail_try", TargetShell::Posix);
}

#[test]
fn exec_run_allow_fail_pipe_segment() {
    assert_exec_matches_fixture_target("run_allow_fail_pipe_segment", TargetShell::Bash);
    assert_exec_matches_fixture_target("run_allow_fail_pipe_segment", TargetShell::Posix);
}

#[test]
fn compile_panic_run_allow_fail_unknown_option() {
    assert_codegen_panics_target("run_allow_fail_unknown_option", TargetShell::Bash, "unknown run option: nope");
}

#[test]
fn compile_panic_run_allow_fail_non_bool() {
    assert_codegen_panics_target("run_allow_fail_non_bool", TargetShell::Bash, "allow_fail must be true/false literal");
}

#[test]
fn compile_panic_run_allow_fail_duplicate() {
    assert_codegen_panics_target("run_allow_fail_duplicate", TargetShell::Bash, "allow_fail specified more than once");
}
