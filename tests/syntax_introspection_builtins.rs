use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
use sh2c::ast::{CompareOp, Expr, Stmt};
mod common;
use common::*;

#[test]
fn parse_introspection_builtins() {
    let program = parse_fixture("introspection_builtins");
    let func = &program.functions[0];

    // stmt0: with env { FOO = "bar" } { if env("FOO") == "bar" ... }
    if let Stmt {
        kind: StmtKind::WithEnv { bindings, body },
        ..
    } = &func.body[0]
    {
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "FOO");
        assert!(matches!(bindings[0].1, Expr { kind: ExprKind::Literal(ref s), .. } if s == "bar"));
        assert!(matches!(
            body[0],
            Stmt {
                kind: StmtKind::If { .. },
                ..
            }
        ));
    } else {
        panic!("Expected WithEnv at stmt0");
    }

    // stmt1: if len(pwd()) > 0 { ... }
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
            assert_eq!(*op, CompareOp::Gt);
            assert!(matches!(
                **right,
                Expr {
                    kind: ExprKind::Number(0),
                    ..
                }
            ));
            assert!(matches!(
                **left,
                Expr {
                    kind: ExprKind::Len(_),
                    ..
                }
            ));
        } else {
            panic!("Expected Compare in stmt1");
        }
    } else {
        panic!("Expected If at stmt1");
    }

    // stmt2: if len(argv0()) > 0 { ... }
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
            assert_eq!(*op, CompareOp::Gt);
            assert!(matches!(
                **right,
                Expr {
                    kind: ExprKind::Number(0),
                    ..
                }
            ));
            assert!(matches!(
                **left,
                Expr {
                    kind: ExprKind::Len(_),
                    ..
                }
            ));
        } else {
            panic!("Expected Compare in stmt2");
        }
    } else {
        panic!("Expected If at stmt2");
    }

    // stmt3: if argc() == 0 { ... }
    if let Stmt {
        kind: StmtKind::If { cond, .. },
        ..
    } = &func.body[3]
    {
        if let Expr {
            kind: ExprKind::Compare { left, op, right },
            ..
        } = cond
        {
            assert_eq!(*op, CompareOp::Eq);
            assert!(matches!(
                **left,
                Expr {
                    kind: ExprKind::Argc,
                    ..
                }
            ));
            assert!(matches!(
                **right,
                Expr {
                    kind: ExprKind::Number(0),
                    ..
                }
            ));
        } else {
            panic!("Expected Compare in stmt3");
        }
    } else {
        panic!("Expected If at stmt3");
    }

    // stmt4: if self_pid() > 0 && ppid() > 0 { ... }
    if let Stmt {
        kind: StmtKind::If { cond, .. },
        ..
    } = &func.body[4]
    {
        assert!(matches!(
            cond,
            Expr {
                kind: ExprKind::And(_, _),
                ..
            }
        ));
    } else {
        panic!("Expected If at stmt4");
    }

    // stmt5: if uid() >= 0 { ... }
    if let Stmt {
        kind: StmtKind::If { cond, .. },
        ..
    } = &func.body[5]
    {
        if let Expr {
            kind: ExprKind::Compare { left, op, right },
            ..
        } = cond
        {
            assert_eq!(*op, CompareOp::Ge);
            assert!(matches!(
                **left,
                Expr {
                    kind: ExprKind::Uid,
                    ..
                }
            ));
            assert!(matches!(
                **right,
                Expr {
                    kind: ExprKind::Number(0),
                    ..
                }
            ));
        } else {
            panic!("Expected Compare in stmt5");
        }
    } else {
        panic!("Expected If at stmt5");
    }
}

#[test]
fn codegen_introspection_builtins() {
    assert_codegen_matches_snapshot("introspection_builtins");
}

#[test]
fn exec_introspection_builtins() {
    assert_exec_matches_fixture("introspection_builtins");
}
