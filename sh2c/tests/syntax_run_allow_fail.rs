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
fn exec_run_allow_fail_works() {
    assert_exec_matches_fixture_target("run_allow_fail_works", TargetShell::Bash);
    assert_exec_matches_fixture_target("run_allow_fail_works", TargetShell::Posix);
}

#[test]
fn exec_run_allow_fail_pipe_segment() {
    assert_exec_matches_fixture_target("run_allow_fail_pipe_segment", TargetShell::Bash);
    assert_exec_matches_fixture_target("run_allow_fail_pipe_segment", TargetShell::Posix);
}

#[test]
fn exec_run_allow_fail_sets_status() {
    assert_exec_matches_fixture_target("run_allow_fail_sets_status", TargetShell::Bash);
    assert_exec_matches_fixture_target("run_allow_fail_sets_status", TargetShell::Posix);
}

#[test]
fn exec_run_allow_fail_does_not_trigger_try_catch() {
    assert_exec_matches_fixture_target(
        "run_allow_fail_does_not_trigger_try_catch",
        TargetShell::Bash,
    );
    assert_exec_matches_fixture_target(
        "run_allow_fail_does_not_trigger_try_catch",
        TargetShell::Posix,
    );
}

#[test]
fn exec_run_allow_fail_pipe_last() {
    assert_exec_matches_fixture_target("run_allow_fail_pipe_last", TargetShell::Bash);
    assert_exec_matches_fixture_target("run_allow_fail_pipe_last", TargetShell::Posix);
}

#[test]
fn compile_panic_run_allow_fail_unknown_option() {
    assert_codegen_panics_target(
        "run_allow_fail_unknown_option",
        TargetShell::Bash,
        "unknown run option: nope",
    );
    assert_codegen_panics_target(
        "run_allow_fail_unknown_option",
        TargetShell::Posix,
        "unknown run option: nope",
    );
}

#[test]
fn compile_panic_run_allow_fail_non_bool() {
    assert_codegen_panics_target(
        "run_allow_fail_non_bool",
        TargetShell::Bash,
        "allow_fail must be true/false literal",
    );
    assert_codegen_panics_target(
        "run_allow_fail_non_bool",
        TargetShell::Posix,
        "allow_fail must be true/false literal",
    );
}

#[test]
fn compile_panic_run_allow_fail_duplicate() {
    assert_codegen_panics_target(
        "run_allow_fail_duplicate",
        TargetShell::Bash,
        "allow_fail specified more than once",
    );
    assert_codegen_panics_target(
        "run_allow_fail_duplicate",
        TargetShell::Posix,
        "allow_fail specified more than once",
    );
}

#[test]
fn compile_panic_run_allow_fail_duplicate_first_false() {
    assert_codegen_panics_target(
        "run_allow_fail_duplicate_first_false",
        TargetShell::Bash,
        "allow_fail specified more than once",
    );
    assert_codegen_panics_target(
        "run_allow_fail_duplicate_first_false",
        TargetShell::Posix,
        "allow_fail specified more than once",
    );
}

#[test]
fn compile_panic_run_allow_fail_in_capture_rejected() {
    let msg = "Expected RParen, got Equals";
    assert_codegen_panics_target("run_allow_fail_in_capture_rejected", TargetShell::Bash, msg);
    assert_codegen_panics_target(
        "run_allow_fail_in_capture_rejected",
        TargetShell::Posix,
        msg,
    );
}
