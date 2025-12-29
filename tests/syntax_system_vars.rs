use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
use sh2c::ast::{CompareOp, Expr, Stmt};
mod common;
use common::*;

#[test]
fn parse_system_vars_builtins() {
    let program = parse_fixture("system_vars_builtins");
    let func = &program.functions[0];
    // Check first If: uid() == env("UID")
    if let Stmt {
        kind: StmtKind::If { cond, .. },
        ..
    } = &func.body[0]
    {
        if let Expr {
            kind: ExprKind::Compare { left, op, right },
            ..
        } = cond
        {
            assert!(matches!(
                **left,
                Expr {
                    kind: ExprKind::Uid,
                    ..
                }
            ));
            assert_eq!(*op, CompareOp::Eq);
            if let Expr {
                kind: ExprKind::Env(inner),
                ..
            } = &**right
            {
                if let Expr {
                    kind: ExprKind::Literal(s),
                    ..
                } = &**inner
                {
                    assert_eq!(s, "UID");
                } else {
                    panic!("Expected Env(Literal(UID))");
                }
            } else {
                panic!("Expected Env on RHS");
            }
        } else {
            panic!("Expected Compare");
        }
    } else {
        panic!("Expected If");
    }

    // Check second If: ppid() == env("PPID")
    if let Stmt {
        kind: StmtKind::If { cond, .. },
        ..
    } = &func.body[1]
    {
        if let Expr {
            kind: ExprKind::Compare { left, op, right },
            ..
        } = cond
        {
            assert!(matches!(
                **left,
                Expr {
                    kind: ExprKind::Ppid,
                    ..
                }
            ));
            assert_eq!(*op, CompareOp::Eq);
            if let Expr {
                kind: ExprKind::Env(inner),
                ..
            } = &**right
            {
                if let Expr {
                    kind: ExprKind::Literal(s),
                    ..
                } = &**inner
                {
                    assert_eq!(s, "PPID");
                } else {
                    panic!("Expected Env(Literal(PPID))");
                }
            } else {
                panic!("Expected Env on RHS");
            }
        } else {
            panic!("Expected Compare");
        }
    } else {
        panic!("Expected If");
    }

    // Check third If: pwd() == env("PWD")
    if let Stmt {
        kind: StmtKind::If { cond, .. },
        ..
    } = &func.body[2]
    {
        if let Expr {
            kind: ExprKind::Compare { left, op, right },
            ..
        } = cond
        {
            assert!(matches!(
                **left,
                Expr {
                    kind: ExprKind::Pwd,
                    ..
                }
            ));
            assert_eq!(*op, CompareOp::Eq);
            if let Expr {
                kind: ExprKind::Env(inner),
                ..
            } = &**right
            {
                if let Expr {
                    kind: ExprKind::Literal(s),
                    ..
                } = &**inner
                {
                    assert_eq!(s, "PWD");
                } else {
                    panic!("Expected Env(Literal(PWD))");
                }
            } else {
                panic!("Expected Env on RHS");
            }
        } else {
            panic!("Expected Compare");
        }
    } else {
        panic!("Expected If");
    }
}

#[test]
fn codegen_system_vars_builtins() {
    assert_codegen_matches_snapshot("system_vars_builtins");
}

#[test]
fn exec_system_vars_builtins() {
    assert_exec_matches_fixture("system_vars_builtins");
}
