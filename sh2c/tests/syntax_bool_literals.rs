use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};
mod common;
use common::*;

#[test]
fn parse_bool_literals() {
    let program = parse_fixture("bool_literals");
    let func = &program.functions[0];

    // stmt0: if true { ... }
    if let Stmt {
        node: StmtKind::If { cond, .. },
        ..
    } = &func.body[0]
    {
        assert!(matches!(
            cond,
            Expr {
                node: ExprKind::Bool(true),
                ..
            }
        ));
    } else {
        panic!("Expected first If");
    }

    // stmt1: if false { ... }
    if let Stmt {
        node: StmtKind::If { cond, .. },
        ..
    } = &func.body[1]
    {
        assert!(matches!(
            cond,
            Expr {
                node: ExprKind::Bool(false),
                ..
            }
        ));
    } else {
        panic!("Expected second If");
    }
}

#[test]
fn codegen_bool_literals() {
    assert_codegen_matches_snapshot("bool_literals");
}

#[test]
fn exec_bool_literals() {
    assert_exec_matches_fixture("bool_literals");
}
