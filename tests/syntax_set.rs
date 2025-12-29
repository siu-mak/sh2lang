use sh2c::ast::StmtKind;
mod common;
use common::*;
use sh2c::ast::{ExprKind, LValue, Stmt};

#[test]
fn parse_set_var_and_env() {
    let program = parse_fixture("set_env_basic");
    let func = &program.functions[0];
    // set env.FOO = "bar"
    if let Stmt {
        kind: StmtKind::Set { target, value: _ },
        ..
    } = &func.body[0]
    {
        if let LValue::Env(name) = target {
            assert_eq!(name, "FOO");
        } else {
            panic!("Expected Env target");
        }
    } else {
        panic!("Expected Set");
    }

    let program_var = parse_fixture("set_var_basic");
    let func_var = &program_var.functions[0];
    // set x = "b"
    if let Stmt {
        kind: StmtKind::Set { target, value: _ },
        ..
    } = &func_var.body[1]
    {
        if let LValue::Var(name) = target {
            assert_eq!(name, "x");
        } else {
            panic!("Expected Var target");
        }
    } else {
        panic!("Expected Set");
    }
}

#[test]
fn codegen_set_var_basic() {
    assert_codegen_matches_snapshot("set_var_basic");
}

#[test]
fn codegen_set_var_list() {
    assert_codegen_matches_snapshot("set_var_list");
}

#[test]
fn codegen_set_env_basic() {
    assert_codegen_matches_snapshot("set_env_basic");
}

#[test]
fn exec_set_var_basic() {
    assert_exec_matches_fixture("set_var_basic");
}

#[test]
fn exec_set_var_list() {
    // Note: requires bash 4+ for arrays usually, but our codegen handles it.
    assert_exec_matches_fixture("set_var_list");
}

#[test]
fn exec_set_env_basic() {
    assert_exec_matches_fixture("set_env_basic");
}

#[test]
fn codegen_set_env_list_invalid() {
    // This should panic during lowering because we assign a list to strict env var
    assert_codegen_panics(
        "set_env_list_invalid",
        "set env.<NAME> requires a scalar string/number",
    );
}

#[test]
fn codegen_set_env_args_invalid() {
    // This should panic during lowering because we assign args to strict env var
    assert_codegen_panics(
        "set_env_args_invalid",
        "set env.<NAME> requires a scalar string/number",
    );
}

#[test]
fn codegen_set_env_scalar_ok() {
    assert_codegen_matches_snapshot("set_env_scalar_ok");
}

#[test]
fn exec_set_env_scalar_ok() {
    assert_exec_matches_fixture("set_env_scalar_ok");
}
