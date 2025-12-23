use sh2c::ast::{self, Stmt, Expr, CompareOp};
mod common;
use common::*;

#[test]
fn parse_proc_params_builtins() {
    let program = parse_fixture("proc_params_builtins");
    let func = &program.functions[0];
    
    // Check first If: argv0() == arg(0)
    if let Stmt::If { cond, .. } = &func.body[0] {
        if let Expr::Compare { left, op, right } = cond {
            assert!(matches!(**left, Expr::Argv0));
            assert_eq!(*op, CompareOp::Eq);
            if let Expr::Arg(n) = **right {
                assert_eq!(n, 0);
            } else { panic!("Expected Arg(0)"); }
        } else { panic!("Expected Compare"); }
    } else { panic!("Expected If 1"); }

    // Check second If: argc() == count(args)
    if let Stmt::If { cond, .. } = &func.body[1] {
        if let Expr::Compare { left, op, right } = cond {
            assert!(matches!(**left, Expr::Argc));
            assert_eq!(*op, CompareOp::Eq);
            if let Expr::Count(inner) = &**right {
                assert!(matches!(**inner, Expr::Args));
            } else { panic!("Expected Count(Args)"); }
        } else { panic!("Expected Compare"); }
    } else { panic!("Expected If 2"); }

    // Check third If: self_pid() == env("BASHPID")
    if let Stmt::If { cond, .. } = &func.body[2] {
        if let Expr::Compare { left, op, right } = cond {
            assert!(matches!(**left, Expr::SelfPid));
            assert_eq!(*op, CompareOp::Eq);
            if let Expr::Env(inner) = &**right {
                if let Expr::Literal(s) = &**inner {
                    assert_eq!(s, "BASHPID");
                } else { panic!("Expected Env(Literal)"); }
            } else { panic!("Expected Env"); }
        } else { panic!("Expected Compare"); }
    } else { panic!("Expected If 3"); }
}

#[test]
fn codegen_proc_params_builtins() {
    assert_codegen_matches_snapshot("proc_params_builtins");
}

#[test]
fn exec_proc_params_builtins() {
    assert_exec_matches_fixture("proc_params_builtins");
}
