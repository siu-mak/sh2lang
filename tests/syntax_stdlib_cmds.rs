
mod common;
use common::*;

#[test]
fn codegen_which_found_bash() {
    assert_codegen_matches_snapshot("which_found");
}

#[test]
fn codegen_which_found_posix() {
    assert_codegen_matches_snapshot_target("which_found", TargetShell::Posix);
}

#[test]
fn exec_which_found_bash() {
    assert_exec_matches_fixture_target("which_found", TargetShell::Bash);
}

#[test]
fn exec_which_found_posix() {
    assert_exec_matches_fixture_target("which_found", TargetShell::Posix);
}

#[test]
fn codegen_which_missing_bash() {
    assert_codegen_matches_snapshot("which_missing");
}

#[test]
fn codegen_which_missing_posix() {
    assert_codegen_matches_snapshot_target("which_missing", TargetShell::Posix);
}

#[test]
fn exec_which_missing_bash() {
    assert_exec_matches_fixture_target("which_missing", TargetShell::Bash);
}

#[test]
fn exec_which_missing_posix() {
    assert_exec_matches_fixture_target("which_missing", TargetShell::Posix);
}

#[test]
fn codegen_require_ok_bash() {
    assert_codegen_matches_snapshot("require_ok");
}

#[test]
fn codegen_require_ok_posix() {
    assert_codegen_matches_snapshot_target("require_ok", TargetShell::Posix);
}

#[test]
fn exec_require_ok_bash() {
    assert_exec_matches_fixture_target("require_ok", TargetShell::Bash);
}

#[test]
fn exec_require_ok_posix() {
    assert_exec_matches_fixture_target("require_ok", TargetShell::Posix);
}

#[test]
fn exec_require_missing_bash() {
    assert_exec_matches_fixture_target("require_missing", TargetShell::Bash);
}

#[test]
fn exec_require_missing_posix() {
    assert_exec_matches_fixture_target("require_missing", TargetShell::Posix);
}

#[test]
fn compile_require_non_literal_is_error() {
    assert_codegen_panics("require_non_literal_is_error", "require() expects a list literal of string literals");
}

#[test]
fn compile_require_non_string_element_is_error() {
    assert_codegen_panics("require_non_string_element_is_error", "require() expects a list literal of string literals");
}

#[test]
fn compile_require_as_expression_is_error() {
    assert_codegen_panics("require_as_expression_is_error", "require() is a statement, not an expression");
}

#[test]
fn compile_which_arity_error() {
    assert_codegen_panics("which_arity_error", "which() returns a value; use it in an expression");
}
