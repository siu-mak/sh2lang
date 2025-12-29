use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
mod common;
use common::*;
use sh2c::ast::{Expr, LValue, Stmt};

#[test]
fn parse_set_env_and_read() {
    let program = parse_fixture("set_env_and_read");
    let func = &program.functions[0];

    // Check Set env.FOO
    if let Stmt {
        kind: StmtKind::Set { target, .. },
        ..
    } = &func.body[0]
    {
        if let LValue::Env(name) = target {
            assert_eq!(name, "FOO");
        } else {
            panic!("Expected LValue::Env");
        }
    } else {
        panic!("Expected Stmt::Set");
    }

    // Check Print(env.FOO) -> Expr::EnvDot("FOO")
    if let Stmt {
        kind: StmtKind::Print(expr),
        ..
    } = &func.body[1]
    {
        if let Expr {
            kind: ExprKind::EnvDot(name),
            ..
        } = expr
        {
            assert_eq!(name, "FOO");
        } else {
            panic!("Expected Expr::EnvDot");
        }
    } else {
        panic!("Expected Stmt::Print");
    }
}

#[test]
fn codegen_set_env_and_read() {
    assert_codegen_matches_snapshot("set_env_and_read");
}

#[test]
fn exec_set_env_and_read() {
    assert_exec_matches_fixture("set_env_and_read");
}
