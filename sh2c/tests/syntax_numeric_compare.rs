use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
use sh2c::ast::{CompareOp, Expr, Stmt};
mod common;
use common::*;

#[test]
fn parse_numeric_compare() {
    let program = parse_fixture("numeric_compare");
    let func = &program.functions[0];

    // if 1 < 2
    if let Stmt {
        node: StmtKind::If { cond, .. },
        ..
    } = &func.body[0]
    {
        if let Expr {
            node: ExprKind::Compare { left, op, right },
            ..
        } = cond
        {
            assert!(matches!(
                **left,
                Expr {
                    node: ExprKind::Number(1),
                    ..
                }
            ));
            assert_eq!(*op, CompareOp::Lt);
            assert!(matches!(
                **right,
                Expr {
                    node: ExprKind::Number(2),
                    ..
                }
            ));
        } else {
            panic!("Expected Compare");
        }
    } else {
        panic!("Expected If");
    }
}

#[test]
fn codegen_numeric_compare() {
    assert_codegen_matches_snapshot("numeric_compare");
}

#[test]
fn exec_numeric_compare() {
    assert_exec_matches_fixture("numeric_compare");
}
