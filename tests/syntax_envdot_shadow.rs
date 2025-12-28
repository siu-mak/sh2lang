mod common;
use common::*;
use sh2c::ast::{Stmt, StmtKind, Expr, ExprKind};

#[test]
fn parse_envdot_shadow() {
    let program = parse_fixture("envdot_shadow");
    let func_shadow = &program.functions[0];
    let func_main = &program.functions[1];
    
    // Check shadow function
    assert_eq!(func_shadow.name, "shadow");
    if let Stmt { kind: StmtKind::Print(Expr { kind: ExprKind::EnvDot(name), .. }), .. } = &func_shadow.body[0] {
         assert_eq!(name, "FOO");
    } else {
         panic!("Expected print(env.FOO)");
    }
    
    // Check main function has WithEnv
    if let Stmt { kind: StmtKind::WithEnv { .. }, .. } = &func_main.body[0] {
    } else {
         panic!("Expected WithEnv in main");
    }
}

#[test]
fn codegen_envdot_shadow() {
    assert_codegen_matches_snapshot("envdot_shadow");
}

#[test]
fn exec_envdot_shadow() {
    assert_exec_matches_fixture("envdot_shadow");
}