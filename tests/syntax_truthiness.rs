mod common;
use sh2c::ast::{Stmt, Expr};
use common::{parse_fixture, assert_codegen_matches_snapshot, assert_exec_matches_fixture};

#[test]
fn parse_truthiness_empty_literal() {
    let program = parse_fixture("truthiness_empty_literal");
    let func = &program.functions[0];
    if let Stmt::If { cond, .. } = &func.body[0] {
        if let Expr::Literal(s) = cond {
            assert_eq!(s, "");
        } else {
            panic!("Expected Literal condition");
        }
    } else {
        panic!("Expected If statement");
    }
}

#[test]
fn codegen_truthiness_empty_literal() { assert_codegen_matches_snapshot("truthiness_empty_literal"); }
#[test]
fn exec_truthiness_empty_literal() { assert_exec_matches_fixture("truthiness_empty_literal"); }

#[test]
fn codegen_truthiness_string_zero_true() { assert_codegen_matches_snapshot("truthiness_string_zero_true"); }
#[test]
fn exec_truthiness_string_zero_true() { assert_exec_matches_fixture("truthiness_string_zero_true"); }

#[test]
fn codegen_truthiness_unset_var() { assert_codegen_matches_snapshot("truthiness_unset_var"); }
#[test]
fn exec_truthiness_unset_var() { assert_exec_matches_fixture("truthiness_unset_var"); }

#[test]
fn codegen_truthiness_empty_var() { assert_codegen_matches_snapshot("truthiness_empty_var"); }
#[test]
fn exec_truthiness_empty_var() { assert_exec_matches_fixture("truthiness_empty_var"); }

#[test]
fn codegen_truthiness_cmdsub_empty() { assert_codegen_matches_snapshot("truthiness_cmdsub_empty"); }
#[test]
fn exec_truthiness_cmdsub_empty() { assert_exec_matches_fixture("truthiness_cmdsub_empty"); }

#[test]
fn codegen_truthiness_cmdsub_nonempty() { assert_codegen_matches_snapshot("truthiness_cmdsub_nonempty"); }
#[test]
fn exec_truthiness_cmdsub_nonempty() { assert_exec_matches_fixture("truthiness_cmdsub_nonempty"); }

#[test]
fn codegen_truthiness_while_scalar() { assert_codegen_matches_snapshot("truthiness_while_scalar"); }
#[test]
fn exec_truthiness_while_scalar() { assert_exec_matches_fixture("truthiness_while_scalar"); }
