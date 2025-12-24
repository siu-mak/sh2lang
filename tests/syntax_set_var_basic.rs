mod common;
use common::*;
use sh2c::ast::{Stmt, LValue, Expr};

#[test]
fn parse_set_var_basic() {
    let program = parse_fixture("set_var_basic");
    let func = &program.functions[0];
    // Check for Set statement
    if let Stmt::Set { target, value } = &func.body[1] {
        if let LValue::Var(name) = target {
             assert_eq!(name, "x");
        } else {
             panic!("Expected LValue::Var");
        }
        if let Expr::Literal(s) = value {
             assert_eq!(s, "b");
        } else {
             panic!("Expected Literal b");
        }
    } else {
        panic!("Expected Stmt::Set, got {:?}", func.body[1]);
    }
}

#[test]
fn codegen_set_var_basic() {
    assert_codegen_matches_snapshot("set_var_basic");
}

#[test]
fn exec_set_var_basic() {
    assert_exec_matches_fixture("set_var_basic");
}
