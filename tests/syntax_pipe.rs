mod common;
use sh2c::ast::{ExprKind, Stmt, StmtKind};
use common::{parse_fixture, assert_codegen_matches_snapshot, assert_exec_matches_fixture};

#[test]
fn parse_pipe_basic() {
    let program = parse_fixture("pipe_basic");
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt { kind: StmtKind::Pipe(_), .. }));
}

#[test]
fn codegen_pipe_basic() {
    assert_codegen_matches_snapshot("pipe_basic");
}

#[test]
fn codegen_pipe() {
    assert_codegen_matches_snapshot("pipe");
}

#[test]
fn codegen_capture_pipe() {
    assert_codegen_matches_snapshot("capture_pipe");
}

#[test]
fn exec_pipe_basic() {
    assert_exec_matches_fixture("pipe_basic");
}