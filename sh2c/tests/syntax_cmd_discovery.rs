#[allow(unused_imports)]
use sh2c::ast::{self, Expr, ExprKind, Stmt, StmtKind};
use sh2c::codegen::TargetShell;

mod common;
use common::*;

// WHICH

#[test]
fn codegen_which_found_mock_bash() {
    assert_codegen_matches_snapshot_target("which_found_mock", TargetShell::Bash);
}

#[test]
fn codegen_which_found_mock_posix() {
    assert_codegen_matches_snapshot_target("which_found_mock", TargetShell::Posix);
}

#[test]
fn codegen_which_missing_bash() {
    assert_codegen_matches_snapshot_target("which_missing", TargetShell::Bash);
}

#[test]
fn codegen_which_missing_posix() {
    assert_codegen_matches_snapshot_target("which_missing", TargetShell::Posix);
}

#[test]
fn exec_which_found_mock_bash() {
    assert_exec_matches_fixture_target("which_found_mock", TargetShell::Bash);
}

#[test]
fn exec_which_found_mock_posix() {
    assert_exec_matches_fixture_target("which_found_mock", TargetShell::Posix);
}

#[test]
fn exec_which_missing_bash() {
    assert_exec_matches_fixture_target("which_missing", TargetShell::Bash);
}

#[test]
fn exec_which_missing_posix() {
    assert_exec_matches_fixture_target("which_missing", TargetShell::Posix);
}

// REQUIRE

#[test]
fn codegen_require_ok_mock_bash() {
    assert_codegen_matches_snapshot_target("require_ok_mock", TargetShell::Bash);
}

#[test]
fn codegen_require_ok_mock_posix() {
    assert_codegen_matches_snapshot_target("require_ok_mock", TargetShell::Posix);
}

#[test]
fn exec_require_ok_mock_bash() {
    assert_exec_matches_fixture_target("require_ok_mock", TargetShell::Bash);
}

#[test]
fn exec_require_ok_mock_posix() {
    assert_exec_matches_fixture_target("require_ok_mock", TargetShell::Posix);
}

#[test]
fn exec_require_missing_bash() {
    assert_exec_matches_fixture_target("require_missing", TargetShell::Bash);
}

#[test]
fn exec_require_missing_posix() {
    assert_exec_matches_fixture_target("require_missing", TargetShell::Posix);
}

// ERRORS

#[test]
fn compile_which_arity_error() {
    assert_codegen_panics("which_arity_error", "which() requires exactly 1 argument");
}

#[test]
fn compile_which_stmt_error() {
    assert_codegen_panics(
        "which_stmt_error",
        "which() returns a value; use it in an expression",
    );
}

#[test]
fn compile_require_arity_error() {
    assert_codegen_panics(
        "require_arity_error",
        "require() requires exactly one argument",
    );
}

#[test]
fn compile_require_expr_error() {
    assert_codegen_panics(
        "require_expr_error",
        "require() is a statement; use it as a standalone call",
    );
}
