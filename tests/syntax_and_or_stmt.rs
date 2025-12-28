use sh2c::ast::{ExprKind, Stmt, StmtKind};
mod common;
use common::*;

#[test]
fn parse_and_or_stmt() {
    let program = parse_fixture("and_or_stmt");
    let func = &program.functions[0];

    // stmt0: run(...) && print(...)
    if let Stmt { kind: StmtKind::AndThen { .. }, .. } = &func.body[0] {
        // OK
    } else {
        panic!("Expected AndThen at index 0, got {:?}", func.body[0]);
    }

    // stmt1: run(...) && print(...)
    if let Stmt { kind: StmtKind::AndThen { .. }, .. } = &func.body[1] {
        // OK
    } else {
        panic!("Expected AndThen at index 1");
    }

    // stmt2: run(...) || print(...)
    if let Stmt { kind: StmtKind::OrElse { .. }, .. } = &func.body[2] {
        // OK
    } else {
        panic!("Expected OrElse at index 2, got {:?}", func.body[2]);
    }

    // stmt3: run(...) || print(...)
    if let Stmt { kind: StmtKind::OrElse { .. }, .. } = &func.body[3] {
        // OK
    } else {
        panic!("Expected OrElse at index 3");
    }
}

#[test]
fn codegen_and_or_stmt() {
    assert_codegen_matches_snapshot("and_or_stmt");
}

#[test]
fn exec_and_or_stmt() {
    assert_exec_matches_fixture("and_or_stmt");
}