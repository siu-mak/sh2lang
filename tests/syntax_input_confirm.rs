use crate::common::*;

mod common;

#[test]
fn exec_input_basic_bash() {
    assert_exec_matches_fixture_target("input_basic", TargetShell::Bash);
}

#[test]
fn exec_confirm_yes_bash() {
    assert_exec_matches_fixture_target("confirm_yes", TargetShell::Bash);
}

#[test]
fn exec_confirm_invalid_then_no_bash() {
    assert_exec_matches_fixture_target("confirm_invalid_then_no", TargetShell::Bash);
}

#[test]
fn compile_panic_input_posix_unsupported() {
    assert_codegen_panics_target("input_basic", TargetShell::Posix, "input(...) is not supported in POSIX sh target");
}

#[test]
fn compile_panic_confirm_posix_unsupported() {
    assert_codegen_panics_target("confirm_yes", TargetShell::Posix, "confirm(...) is not supported in POSIX sh target");
}