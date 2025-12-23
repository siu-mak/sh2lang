use sh2c::ast::{self, Stmt, Expr, ArithOp, CompareOp};
mod common;
use common::*;

#[test]
fn parse_self_pid_arith() {
    let program = parse_fixture("self_pid_arith");
    let func = &program.functions[0];
    
    // Check Let x = self_pid() + 1
    if let Stmt::Let { name, value } = &func.body[0] {
        assert_eq!(name, "x");
        if let Expr::Arith { left, op, right } = value {
             assert!(matches!(**left, Expr::SelfPid));
             assert_eq!(*op, ArithOp::Add);
             if let Expr::Number(n) = **right {
                 assert_eq!(n, 1);
             } else { panic!("Expected Number(1)"); }
        } else { panic!("Expected Arith"); }
    } else { panic!("Expected Let"); }

    // Check If x > self_pid()
    if let Stmt::If { cond, .. } = &func.body[1] {
        if let Expr::Compare { left, op, right } = cond {
             if let Expr::Var(v) = &**left {
                 assert_eq!(v, "x");
             } else { panic!("Expected Var(x)"); }
             assert_eq!(*op, CompareOp::Gt);
             assert!(matches!(**right, Expr::SelfPid));
        } else { panic!("Expected Compare"); }
    } else { panic!("Expected If"); }
}

#[test]
fn codegen_self_pid_arith() {
    assert_codegen_matches_snapshot("self_pid_arith");
}

#[test]
fn exec_self_pid_arith() {
    assert_exec_matches_fixture("self_pid_arith");
}
