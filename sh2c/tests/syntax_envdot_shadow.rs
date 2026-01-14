mod common;
use common::*;
use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};

#[test]
fn parse_envdot_shadow() {
    let program = parse_fixture("envdot_shadow");
    // Only one function 'shadow', top level code is separate (or in main if wrapped, but here we just check functions)
    // Find 'shadow' function
    let func_shadow = program.functions.iter().find(|f| f.name == "shadow").expect("Function 'shadow' not found");

    // Check shadow function body contains print(env.FOO)
    if let Stmt {
        node:
            StmtKind::Print(Expr {
                node: ExprKind::EnvDot(name),
                ..
            }),
        ..
    } = &func_shadow.body[0]
    {
        assert_eq!(name, "FOO");
    } else {
        panic!("Expected print(env.FOO) as first statement in shadow");
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
