use sh2c::ast::{self, Stmt, Expr};
mod common;
use common::*;

#[test]
fn parse_env_basic() {
    let program = parse_fixture("env_basic");
    let func = &program.functions[0];
    // with env { FOO = "bar" } { print(env("FOO")) }
    // body[0] is WithEnv
    if let Stmt::WithEnv { body, .. } = &func.body[0] {
        // body of WithEnv block
        if let Stmt::Print(Expr::Env(inner)) = &body[0] {
            if let Expr::Literal(s) = &**inner {
                assert_eq!(s, "FOO");
            } else { panic!("Expected Literal inside Env"); }
        } else { panic!("Expected Print(Env)"); }
    } else { panic!("Expected WithEnv"); }
}

#[test]
fn codegen_env_basic() {
    assert_codegen_matches_snapshot("env_basic");
}

#[test]
fn exec_env_basic() {
    assert_exec_matches_fixture("env_basic");
}

#[test]
fn codegen_env_indirect() {
    assert_codegen_matches_snapshot("env_indirect");
}

#[test]
fn exec_env_indirect() {
    assert_exec_matches_fixture("env_indirect");
}
