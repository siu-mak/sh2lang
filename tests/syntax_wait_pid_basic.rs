mod common;
use common::*;
use sh2c::ast::{Stmt, Expr};

#[test]
fn parse_wait_pid_basic() {
    let program = parse_fixture("wait_pid_basic");
    let func = &program.functions[0];
    
    // Check let p = pid()
    if let Stmt::Let { name, value: Expr::Pid } = &func.body[1] {
        assert_eq!(name, "p");
    } else {
        panic!("Expected Let p = Pid");
    }
    
    // Check wait(p)
    if let Stmt::Wait(Some(Expr::Var(name))) = &func.body[2] {
        assert_eq!(name, "p");
    } else {
        panic!("Expected Wait(Var(p))");
    }
}

#[test]
fn codegen_wait_pid_basic() {
    assert_codegen_matches_snapshot("wait_pid_basic");
}

#[test]
fn exec_wait_pid_basic() {
    assert_exec_matches_fixture("wait_pid_basic");
}
