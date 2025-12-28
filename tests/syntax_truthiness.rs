mod common;
use common::*;
use sh2c::codegen::TargetShell;

#[test]
fn exec_truthy_empty_string() {
    assert_codegen_matches_snapshot("truthy_empty_string");
    assert_exec_matches_fixture_target("truthy_empty_string", TargetShell::Bash);
    assert_exec_matches_fixture_target("truthy_empty_string", TargetShell::Posix);
}

#[test]
fn exec_truthy_nonempty_string() {
    assert_exec_matches_fixture_target("truthy_nonempty_string", TargetShell::Bash);
    assert_exec_matches_fixture_target("truthy_nonempty_string", TargetShell::Posix);
}

#[test]
fn exec_truthy_whitespace_newline() {
    assert_exec_matches_fixture_target("truthy_whitespace_newline", TargetShell::Bash);
    assert_exec_matches_fixture_target("truthy_whitespace_newline", TargetShell::Posix);
}

#[test]
fn exec_truthy_zero() {
    assert_exec_matches_fixture_target("truthy_zero", TargetShell::Bash);
    assert_exec_matches_fixture_target("truthy_zero", TargetShell::Posix);
}

#[test]
fn exec_truthy_capture() {
    assert_exec_matches_fixture_target("truthy_capture", TargetShell::Bash);
    assert_exec_matches_fixture_target("truthy_capture", TargetShell::Posix);
}

#[test]
fn exec_truthy_not_scalar() {
    assert_exec_matches_fixture_target("truthy_not_scalar", TargetShell::Bash);
    assert_exec_matches_fixture_target("truthy_not_scalar", TargetShell::Posix);
}

#[test]
fn codegen_panic_truthy_args_disallowed_bash() {
    assert_codegen_panics_target("truthy_args_disallowed", TargetShell::Bash, "args/list is not a valid condition");
}

#[test]
fn codegen_panic_truthy_args_disallowed_posix() {
    assert_codegen_panics_target("truthy_args_disallowed", TargetShell::Posix, "args/list is not a valid condition");
}

#[test]
fn codegen_panic_truthy_list_disallowed_bash() {
    assert_codegen_panics_target("truthy_list_disallowed", TargetShell::Bash, "args/list is not a valid condition");
}

#[test]
fn codegen_panic_truthy_list_disallowed_posix() {
    assert_codegen_panics_target("truthy_list_disallowed", TargetShell::Posix, "args/list is not a valid condition");
}