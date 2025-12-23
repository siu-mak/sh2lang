use sh2c::ast::{self, Stmt, Expr, CompareOp};
mod common;
use common::*;

#[test]
fn parse_system_vars_builtins() {
    let program = parse_fixture("system_vars_builtins");
    let func = &program.functions[0];
    // Check first If: uid() == env("UID")
    if let Stmt::If { cond, .. } = &func.body[0] {
        if let Expr::Compare { left, op, right } = cond {
            assert!(matches!(**left, Expr::Uid));
            assert_eq!(*op, CompareOp::Eq);
            if let Expr::Env(inner) = &**right {
                 if let Expr::Literal(s) = &**inner {
                     assert_eq!(s, "UID");
                 } else { panic!("Expected Env(Literal(UID))"); }
            } else { panic!("Expected Env on RHS"); }
        } else { panic!("Expected Compare"); }
    } else { panic!("Expected If"); }
}

#[test]
fn codegen_system_vars_builtins() {
    assert_codegen_matches_snapshot("system_vars_builtins");
}

#[test]
fn exec_system_vars_builtins() {
    assert_exec_matches_fixture("system_vars_builtins");
}
