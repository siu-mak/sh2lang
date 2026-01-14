use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
use sh2c::ast::{CompareOp, Expr, Stmt};
mod common;
use common::*;

#[test]
fn parse_status_builtin() {
    let program = parse_fixture("status_builtin");
    let func = &program.functions[0];

    // stmt0: run("true")
    assert!(matches!(
        func.body[0],
        Stmt {
            node: StmtKind::Run(_),
            ..
        }
    ));

    // stmt1: if status() == 0 { ... }
    if let Stmt {
        node: StmtKind::If { cond, .. },
        ..
    } = &func.body[1]
    {
        if let Expr {
            node: ExprKind::Compare { left, op, right },
            ..
        } = cond
        {
            assert!(matches!(
                **left,
                Expr {
                    node: ExprKind::Status,
                    ..
                }
            ));
            assert_eq!(*op, CompareOp::Eq);
            assert!(matches!(
                **right,
                Expr {
                    node: ExprKind::Number(0),
                    ..
                }
            ));
        } else {
            panic!("Expected Compare in first if");
        }
    } else {
        panic!("Expected If after first run");
    }

    // stmt2: run("false")
    assert!(matches!(
        func.body[2],
        Stmt {
            node: StmtKind::Run(_),
            ..
        }
    ));

    // stmt3: if status() != 0 { ... }
    if let Stmt {
        node: StmtKind::If { cond, .. },
        ..
    } = &func.body[3]
    {
        if let Expr {
            node: ExprKind::Compare { left, op, right },
            ..
        } = cond
        {
            assert!(matches!(
                **left,
                Expr {
                    node: ExprKind::Status,
                    ..
                }
            ));
            assert_eq!(*op, CompareOp::NotEq);
            assert!(matches!(
                **right,
                Expr {
                    node: ExprKind::Number(0),
                    ..
                }
            ));
        } else {
            panic!("Expected Compare in second if");
        }
    } else {
        panic!("Expected If after second run");
    }
}

#[test]
fn codegen_status_builtin() {
    assert_codegen_matches_snapshot("status_builtin");
}

#[test]
fn exec_status_builtin() {
    assert_exec_matches_fixture("status_builtin");
}
