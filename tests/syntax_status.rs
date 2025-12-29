use sh2c::ast::StmtKind;
use sh2c::ast::{Expr, ExprKind, Stmt};
mod common;
use common::*;

#[test]
fn parse_status_basic() {
    let program = parse_fixture("status_basic");
    let func = &program.functions[0];
    // func body: run(...), print(status())
    // 2nd statement should be Print(Status)
    if let Stmt {
        kind: StmtKind::Print(Expr {
            kind: ExprKind::Status,
            ..
        }),
        ..
    } = &func.body[1]
    {
        // ok
    } else {
        panic!("Expected Print(Status), got {:?}", &func.body[1]);
    }
}

#[test]
fn codegen_status_basic() {
    assert_codegen_matches_snapshot("status_basic");
}

#[test]
fn exec_status_basic() {
    assert_exec_matches_fixture("status_basic");
}

#[test]
fn exec_status_cond() {
    assert_exec_matches_fixture("status_cond");
}
