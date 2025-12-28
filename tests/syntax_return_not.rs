mod common;
use common::*;
use sh2c::ast::{Stmt, StmtKind, Expr, ExprKind};

#[test]
fn parse_return_not() {
    let program = parse_fixture("return_not");
    let func = &program.functions[0];
    if let Stmt { kind: StmtKind::Return(Some(Expr { kind: ExprKind::Not(_), .. })), .. } = &func.body[0] {
        // ok
    } else {
        panic!("Expected return of a Not(...) expr");
    }
}

#[test]
fn codegen_return_not() { assert_codegen_matches_snapshot("return_not"); }

#[test]
fn exec_return_not() { assert_exec_matches_fixture("return_not"); }