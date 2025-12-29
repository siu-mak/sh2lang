use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
use sh2c::ast::{CompareOp, Expr, Stmt};
mod common;
use common::*;

#[test]
fn parse_pid_wait_spawn() {
    let program = parse_fixture("pid_wait_spawn");
    let func = &program.functions[0];

    // stmt0: spawn run(...)
    if let Stmt {
        node: StmtKind::Spawn { stmt },
        ..
    } = &func.body[0]
    {
        assert!(matches!(
            **stmt,
            Stmt {
                node: StmtKind::Run(_),
                ..
            }
        ));
    } else {
        panic!("Expected Spawn(run(...))");
    }

    // stmt1: let p = pid()
    if let Stmt {
        node: StmtKind::Let { name, value },
        ..
    } = &func.body[1]
    {
        assert_eq!(name, "p");
        assert!(matches!(
            value,
            Expr {
                node: ExprKind::Pid,
                ..
            }
        ));
    } else {
        panic!("Expected let p = pid()");
    }

    // stmt2: wait(p)
    if let Stmt {
        node: StmtKind::Wait(Some(e)),
        ..
    } = &func.body[2]
    {
        assert!(matches!(e, Expr { node: ExprKind::Var(v), .. } if v == "p"));
    } else {
        panic!("Expected wait(p)");
    }

    // stmt3: if status() == 0 { ... }
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
            assert_eq!(*op, CompareOp::Eq);
            assert!(matches!(
                **right,
                Expr {
                    node: ExprKind::Number(0),
                    ..
                }
            ));
        } else {
            panic!("Expected Compare in if");
        }
    } else {
        panic!("Expected If");
    }
}

#[test]
fn codegen_pid_wait_spawn() {
    assert_codegen_matches_snapshot("pid_wait_spawn");
}

#[test]
fn exec_pid_wait_spawn() {
    assert_exec_matches_fixture("pid_wait_spawn");
}
