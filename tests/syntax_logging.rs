
#[allow(unused_imports)]
use sh2c::ast::{self, Stmt, Expr};
use sh2c::parser;
use sh2c::lexer;
use sh2c::lower;
use sh2c::codegen::{self, TargetShell};

mod common;
use common::*;

// LOGGING

#[test]
fn codegen_log_info_basic_bash() {
    assert_codegen_matches_snapshot_target("log_info_basic", TargetShell::Bash);
}

#[test]
fn codegen_log_info_basic_posix() {
    assert_codegen_matches_snapshot_target("log_info_basic", TargetShell::Posix);
}

#[test]
fn exec_log_info_basic_bash() {
    assert_exec_matches_fixture_target("log_info_basic", TargetShell::Bash);
}

#[test]
fn exec_log_info_basic_posix() {
    assert_exec_matches_fixture_target("log_info_basic", TargetShell::Posix);
}

#[test]
fn exec_log_warn_error_bash() {
    assert_exec_matches_fixture_target("log_warn_error", TargetShell::Bash);
}

#[test]
fn exec_log_warn_error_posix() {
    assert_exec_matches_fixture_target("log_warn_error", TargetShell::Posix);
}

#[test]
fn exec_log_timestamp_enabled_bash() {
    assert_exec_matches_fixture_target("log_timestamp_enabled", TargetShell::Bash);
}

#[test]
fn exec_log_timestamp_enabled_posix() {
    assert_exec_matches_fixture_target("log_timestamp_enabled", TargetShell::Posix);
}

// ERRORS

#[test]
fn compile_log_bad_arity_0() {
    assert_codegen_panics("log_bad_arity_0", "requires 1 or 2 arguments");
}

#[test]
fn compile_log_bad_arity_3() {
    assert_codegen_panics("log_bad_arity_3", "requires 1 or 2 arguments");
}

#[test]
fn compile_log_bad_timestamp_type() {
    assert_codegen_panics("log_bad_timestamp_type", "second argument must be a boolean literal");
}

#[test]
fn compile_log_used_as_expr() {
    assert_codegen_panics("log_used_as_expr", "is a statement, not an expression");
}
