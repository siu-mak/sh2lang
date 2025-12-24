mod common;
use common::*;
use sh2c::ast::{Stmt, Expr};

#[test]
fn parse_with_env_colon_basic() {
    let program = parse_fixture("with_env_colon_basic");
    let func = &program.functions[0];
    
    // Check WithEnv bindings
    if let Stmt::WithEnv { bindings, .. } = &func.body[0] {
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].0, "FOO");
        if let Expr::Literal(s) = &bindings[0].1 { assert_eq!(s, "bar"); } else { panic!("Expected literal"); }
        assert_eq!(bindings[1].0, "BAZ");
        if let Expr::Literal(s) = &bindings[1].1 { assert_eq!(s, "qux"); } else { panic!("Expected literal"); }
    } else {
        panic!("Expected Stmt::WithEnv");
    }
}

#[test]
fn codegen_with_env_colon_basic() {
    assert_codegen_matches_snapshot("with_env_colon_basic");
}

#[test]
fn exec_with_env_colon_basic() {
    assert_exec_matches_fixture("with_env_colon_basic");
}
