mod common;
use common::*;
use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};

#[test]
fn parse_status_basic() {
    let program = parse_fixture("status_basic");
    let func = &program.functions[0];

    // Check print(status())
    if let Stmt {
        node: StmtKind::Print(Expr {
            node: ExprKind::Status,
            ..
        }),
        ..
    } = &func.body[1]
    {
        // OK
    } else {
        panic!("Expected Print(Status)");
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
