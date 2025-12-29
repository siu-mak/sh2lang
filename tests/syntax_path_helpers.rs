#[allow(unused_imports)]
use sh2c::ast::{self, Expr, ExprKind, Stmt, StmtKind};
use sh2c::codegen::TargetShell;

mod common;
use common::*;

// HOME

#[test]
fn codegen_path_home_basic_bash() {
    assert_codegen_matches_snapshot_target("path_home_basic", TargetShell::Bash);
}

#[test]
fn codegen_path_home_basic_posix() {
    assert_codegen_matches_snapshot_target("path_home_basic", TargetShell::Posix);
}

#[test]
fn exec_path_home_basic_bash() {
    assert_exec_matches_fixture_target("path_home_basic", TargetShell::Bash);
}

#[test]
fn exec_path_home_basic_posix() {
    assert_exec_matches_fixture_target("path_home_basic", TargetShell::Posix);
}

// PATH_JOIN

#[test]
fn codegen_path_join_basic_bash() {
    assert_codegen_matches_snapshot_target("path_join_basic", TargetShell::Bash);
}

#[test]
fn codegen_path_join_basic_posix() {
    assert_codegen_matches_snapshot_target("path_join_basic", TargetShell::Posix);
}

#[test]
fn exec_path_join_basic_bash() {
    assert_exec_matches_fixture_target("path_join_basic", TargetShell::Bash);
}

#[test]
fn exec_path_join_basic_posix() {
    assert_exec_matches_fixture_target("path_join_basic", TargetShell::Posix);
}

#[test]
fn exec_path_join_slashes_and_empty_bash() {
    assert_exec_matches_fixture_target("path_join_slashes_and_empty", TargetShell::Bash);
}

#[test]
fn exec_path_join_slashes_and_empty_posix() {
    assert_exec_matches_fixture_target("path_join_slashes_and_empty", TargetShell::Posix);
}

#[test]
fn exec_path_join_absolute_reset_bash() {
    assert_exec_matches_fixture_target("path_join_absolute_reset", TargetShell::Bash);
}

#[test]
fn exec_path_join_absolute_reset_posix() {
    assert_exec_matches_fixture_target("path_join_absolute_reset", TargetShell::Posix);
}

// ERRORS

#[test]
fn compile_path_home_arity_error() {
    assert_codegen_panics("path_home_arity_error", "home() takes no arguments");
}

#[test]
fn compile_path_join_arity_error() {
    assert_codegen_panics("path_join_arity_error", "requires at least 1 argument");
}

#[test]
fn compile_path_home_stmt_error() {
    assert_codegen_panics(
        "path_home_stmt_error",
        "returns a value; use it in an expression",
    );
}

#[test]
fn compile_path_join_stmt_error() {
    assert_codegen_panics(
        "path_join_stmt_error",
        "returns a value; use it in an expression",
    );
}
