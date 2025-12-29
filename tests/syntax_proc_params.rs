use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
use sh2c::ast::{CompareOp, Expr, Stmt};
mod common;
use common::*;

#[test]
fn parse_proc_params_builtins() {
    let program = parse_fixture("proc_params_builtins");
    let func = &program.functions[0];

    // Check first If: argv0() == arg(0)
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
                    node: ExprKind::Argv0,
                    ..
                }
            ));
            assert_eq!(*op, CompareOp::Eq);
            if let Expr {
                node: ExprKind::Arg(n),
                ..
            } = **right
            {
                assert_eq!(n, 0);
            } else {
                panic!("Expected Arg(0)");
            }
        } else {
            panic!("Expected Compare");
        }
    } else {
        panic!("Expected If 1");
    }

    // Check second If: argc() == count(args)
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
                    node: ExprKind::Argc,
                    ..
                }
            ));
            assert_eq!(*op, CompareOp::Eq);
            if let Expr {
                node: ExprKind::Count(inner),
                ..
            } = &**right
            {
                assert!(matches!(
                    **inner,
                    Expr {
                        node: ExprKind::Args,
                        ..
                    }
                ));
            } else {
                panic!("Expected Count(Args)");
            }
        } else {
            panic!("Expected Compare");
        }
    } else {
        panic!("Expected If 2");
    }

    // Check third If: self_pid() == env("BASHPID")
    if let Stmt {
        node: StmtKind::If { cond, .. },
        ..
    } = &func.body[2]
    {
        if let Expr {
            node: ExprKind::Compare { left, op, right },
            ..
        } = cond
        {
            assert!(matches!(
                **left,
                Expr {
                    node: ExprKind::SelfPid,
                    ..
                }
            ));
            assert_eq!(*op, CompareOp::Eq);
            if let Expr {
                node: ExprKind::Env(inner),
                ..
            } = &**right
            {
                if let Expr {
                    node: ExprKind::Literal(s),
                    ..
                } = &**inner
                {
                    assert_eq!(s, "BASHPID");
                } else {
                    panic!("Expected Env(Literal)");
                }
            } else {
                panic!("Expected Env");
            }
        } else {
            panic!("Expected Compare");
        }
    } else {
        panic!("Expected If 3");
    }
}

#[test]
fn codegen_proc_params_builtins() {
    assert_codegen_matches_snapshot("proc_params_builtins");
}

#[test]
fn exec_proc_params_builtins() {
    assert_exec_matches_fixture("proc_params_builtins");
}
