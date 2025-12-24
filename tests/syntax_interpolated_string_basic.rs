mod common;
use common::*;
use sh2c::ast::{Stmt, Expr};

#[test]
fn parse_interpolated_string_basic() {
    let program = parse_fixture("interpolated_string_basic");
    let func = &program.functions[0];
    if let Stmt::Print(Expr::Concat(_, _)) = &func.body[1] {
        // ok
    } else {
        panic!("Expected Print(Concat..)");
    }
}

#[test]
fn codegen_interpolated_string_basic() {
    assert_codegen_matches_snapshot("interpolated_string_basic");
}

#[test]
fn exec_interpolated_string_basic() {
    assert_exec_matches_fixture("interpolated_string_basic");
}
