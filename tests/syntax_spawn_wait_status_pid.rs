use sh2c::ast::{Stmt, Expr, CompareOp};
mod common;
use common::*;

#[test]
fn parse_spawn_wait_status_pid() {
    let program = parse_fixture("spawn_wait_status_pid");
    let func = &program.functions[0];

    // stmt0: spawn run(...)
    assert!(matches!(func.body[0], Stmt::Spawn { .. }));

    // stmt1: if pid() > 0 { ... }
    if let Stmt::If { cond, .. } = &func.body[1] {
        assert!(matches!(cond,
            Expr::Compare { op: CompareOp::Gt, .. }
        ));
    } else { panic!("Expected If for pid() check"); }

    // stmt2: wait(pid())
    if let Stmt::Wait(Some(e)) = &func.body[2] {
        assert!(matches!(e, Expr::Pid));
    } else { panic!("Expected Wait(Some(Pid))"); }

    // stmt3: if status() == 7 { ... }
    if let Stmt::If { cond, .. } = &func.body[3] {
        if let Expr::Compare { left, op, right } = cond {
            assert_eq!(*op, CompareOp::Eq);
            assert!(matches!(**left, Expr::Status));
            assert!(matches!(**right, Expr::Number(7)));
        } else { panic!("Expected Compare for status() == 7"); }
    } else { panic!("Expected If for status()"); }
}

#[test]
fn codegen_spawn_wait_status_pid() {
    assert_codegen_matches_snapshot("spawn_wait_status_pid");
}

#[test]
fn exec_spawn_wait_status_pid() {
    assert_exec_matches_fixture("spawn_wait_status_pid");
}
