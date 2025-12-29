use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
use sh2c::ast::{ArithOp, CompareOp, Expr, Stmt};
mod common;
use common::*;

#[test]
fn parse_self_pid_arith() {
    let program = parse_fixture("self_pid_arith");
    let func = &program.functions[0];

    // Check Let x = self_pid() + 1
    if let Stmt {
        node: StmtKind::Let { name, value },
        ..
    } = &func.body[0]
    {
        assert_eq!(name, "x");
        if let Expr {
            node: ExprKind::Arith { left, op, right },
            ..
        } = value
        {
            assert!(matches!(
                **left,
                Expr {
                    node: ExprKind::SelfPid,
                    ..
                }
            ));
            assert_eq!(*op, ArithOp::Add);
            if let Expr {
                node: ExprKind::Number(n),
                ..
            } = **right
            {
                assert_eq!(n, 1);
            } else {
                panic!("Expected Number(1)");
            }
        } else {
            panic!("Expected Arith");
        }
    } else {
        panic!("Expected Let");
    }

    // Check If x > self_pid()
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
            if let Expr {
                node: ExprKind::Var(v),
                ..
            } = &**left
            {
                assert_eq!(v, "x");
            } else {
                panic!("Expected Var(x)");
            }
            assert_eq!(*op, CompareOp::Gt);
            assert!(matches!(
                **right,
                Expr {
                    node: ExprKind::SelfPid,
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
fn codegen_self_pid_arith() {
    assert_codegen_matches_snapshot("self_pid_arith");
}

#[test]
fn exec_self_pid_arith() {
    assert_exec_matches_fixture("self_pid_arith");
}
