use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
use sh2c::ast::{CompareOp, Expr, Stmt};
mod common;
use common::*;

#[test]
fn parse_spawn_wait_status_pid() {
    let program = parse_fixture("spawn_wait_status_pid");
    let func = &program.functions[0];

    // stmt0: spawn run(...)
    assert!(matches!(
        func.body[0],
        Stmt {
            kind: StmtKind::Spawn { .. },
            ..
        }
    ));

    // stmt1: if pid() > 0 { ... }
    if let Stmt {
        kind: StmtKind::If { cond, .. },
        ..
    } = &func.body[1]
    {
        assert!(matches!(
            cond,
            Expr {
                kind: ExprKind::Compare {
                    op: CompareOp::Gt,
                    ..
                },
                ..
            }
        ));
    } else {
        panic!("Expected If for pid() check");
    }

    // stmt2: wait(pid())
    if let Stmt {
        kind: StmtKind::Wait(Some(e)),
        ..
    } = &func.body[2]
    {
        assert!(matches!(
            e,
            Expr {
                kind: ExprKind::Pid,
                ..
            }
        ));
    } else {
        panic!("Expected Wait(Some(Pid))");
    }

    // stmt3: if status() == 7 { ... }
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
                    kind: ExprKind::Status,
                    ..
                }
            ));
            assert!(matches!(
                **right,
                Expr {
                    kind: ExprKind::Number(7),
                    ..
                }
            ));
        } else {
            panic!("Expected Compare for status() == 7");
        }
    } else {
        panic!("Expected If for status()");
    }
}

#[test]
fn codegen_spawn_wait_status_pid() {
    assert_codegen_matches_snapshot("spawn_wait_status_pid");
}

#[test]
fn exec_spawn_wait_status_pid() {
    assert_exec_matches_fixture("spawn_wait_status_pid");
}

/// Test that spawn correctly backgrounds the command - the flag file should
/// NOT exist immediately after spawn (command runs async), but SHOULD exist
/// after wait() completes.
#[test]
fn exec_spawn_async_file() {
    assert_exec_matches_fixture_target("spawn_async_file", TargetShell::Bash);
    assert_exec_matches_fixture_target("spawn_async_file", TargetShell::Posix);
}
