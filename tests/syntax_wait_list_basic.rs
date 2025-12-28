mod common;
use common::*;
use sh2c::ast::{Stmt, StmtKind, Expr, ExprKind};

#[test]
fn parse_wait_list_basic() {
    let program = parse_fixture("wait_list_basic");
    let func = &program.functions[0];
    
    // Check wait([p1, p2])
    if let Stmt { kind: StmtKind::Wait(Some(Expr { kind: ExprKind::List(exprs), .. })), .. } = &func.body[4] {
        assert_eq!(exprs.len(), 2);
    } else {
        panic!("Expected Wait(List([..]))");
    }
}

#[test]
fn codegen_wait_list_basic() {
    assert_codegen_matches_snapshot("wait_list_basic");
}

#[test]
fn exec_wait_list_basic() {
    assert_exec_matches_fixture("wait_list_basic");
}