use sh2c::ast::{Stmt, Expr, CompareOp};
mod common;
use common::*;

#[test]
fn parse_pid_wait_spawn() {
    let program = parse_fixture("pid_wait_spawn");
    let func = &program.functions[0];

    // stmt0: spawn run(...)
    if let Stmt::Spawn { stmt } = &func.body[0] {
        assert!(matches!(**stmt, Stmt::Run(_)));
    } else { panic!("Expected Spawn(run(...))"); }

    // stmt1: let p = pid()
    if let Stmt::Let { name, value } = &func.body[1] {
        assert_eq!(name, "p");
        assert!(matches!(value, Expr::Pid));
    } else { panic!("Expected let p = pid()"); }

    // stmt2: wait(p)
    if let Stmt::Wait(Some(e)) = &func.body[2] {
        assert!(matches!(e, Expr::Var(v) if v == "p"));
    } else { panic!("Expected wait(p)"); }

    // stmt3: if status() == 0 { ... }
    if let Stmt::If { cond, .. } = &func.body[3] {
        if let Expr::Compare { left, op, right } = cond {
            assert!(matches!(**left, Expr::Status));
            assert_eq!(*op, CompareOp::Eq);
            assert!(matches!(**right, Expr::Number(0)));
        } else { panic!("Expected Compare in if"); }
    } else { panic!("Expected If"); }
}

#[test]
fn codegen_pid_wait_spawn() {
    assert_codegen_matches_snapshot("pid_wait_spawn");
}

#[test]
fn exec_pid_wait_spawn() {
    assert_exec_matches_fixture("pid_wait_spawn");
}
