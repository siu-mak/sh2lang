
#[allow(unused_imports)]
use sh2c::ast::{self, Stmt, Expr};
use sh2c::parser;
use sh2c::lexer;
use sh2c::lower;
use sh2c::codegen::{self, TargetShell};

mod common;
use common::*;

// READ_FILE

#[test]
fn codegen_read_file_basic_bash() {
    assert_codegen_matches_snapshot_target("read_file_basic", TargetShell::Bash);
}

#[test]
fn codegen_read_file_basic_posix() {
    assert_codegen_matches_snapshot_target("read_file_basic", TargetShell::Posix);
}

#[test]
fn exec_read_file_basic_bash() {
    assert_exec_matches_fixture_target("read_file_basic", TargetShell::Bash);
}

#[test]
fn exec_read_file_basic_posix() {
    assert_exec_matches_fixture_target("read_file_basic", TargetShell::Posix);
}

#[test]
fn exec_read_file_missing_bash() {
    assert_exec_matches_fixture_target("read_file_missing", TargetShell::Bash);
}

#[test]
fn exec_read_file_missing_posix() {
    assert_exec_matches_fixture_target("read_file_missing", TargetShell::Posix);
}

// WRITE_FILE

#[test]
fn codegen_write_file_overwrite_bash() {
    assert_codegen_matches_snapshot_target("write_file_overwrite", TargetShell::Bash);
}

#[test]
fn codegen_write_file_overwrite_posix() {
    assert_codegen_matches_snapshot_target("write_file_overwrite", TargetShell::Posix);
}

#[test]
fn exec_write_file_overwrite_bash() {
    assert_exec_matches_fixture_target("write_file_overwrite", TargetShell::Bash);
}

#[test]
fn exec_write_file_overwrite_posix() {
    assert_exec_matches_fixture_target("write_file_overwrite", TargetShell::Posix);
}


#[test]
fn exec_write_file_append_bash() {
    assert_exec_matches_fixture_target("write_file_append", TargetShell::Bash);
}

#[test]
fn exec_write_file_append_posix() {
    assert_exec_matches_fixture_target("write_file_append", TargetShell::Posix);
}

#[test]
fn exec_write_file_spaces_bash() {
    assert_exec_matches_fixture_target("write_file_spaces", TargetShell::Bash);
}

#[test]
fn exec_write_file_spaces_posix() {
    assert_exec_matches_fixture_target("write_file_spaces", TargetShell::Posix);
}


// ERRORS

#[test]
fn compile_read_file_stmt_error() {
    assert_codegen_panics("read_file_stmt_error", "read_file() returns a value");
}

#[test]
fn compile_write_file_expr_error() {
    assert_codegen_panics("write_file_expr_error", "write_file() is a statement");
}

#[test]
fn compile_write_file_bad_arg() {
    assert_codegen_panics("write_file_bad_arg", "third argument must be a boolean literal");
}
