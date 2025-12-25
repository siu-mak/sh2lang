mod common;
use common::*;
use sh2c::ast::{Stmt, LValue};

#[test]
fn parse_set_var_basic() {
    let program = parse_fixture("set_var_basic");
    let func = &program.functions[0];
    assert_eq!(func.body.len(), 3);

    match &func.body[0] {
        Stmt::Let { name, .. } => {
             assert_eq!(name, "x");
        }
        _ => panic!("Expected Let"),
    }

    match &func.body[1] {
        Stmt::Set { target, .. } => {
            assert!(matches!(target, LValue::Var(name) if name == "x"));
        }
        _ => panic!("Expected Set"),
    }

    if let Stmt::Print(_) = &func.body[2] {} else { panic!("Expected Print"); }
}

#[test]
fn codegen_set_var_basic() {
    assert_codegen_matches_snapshot("set_var_basic");
}

#[test]
fn exec_set_var_basic() {
    assert_exec_matches_fixture("set_var_basic");
}
