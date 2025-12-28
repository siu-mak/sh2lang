mod common;
use common::*;
use sh2c::ast::{Stmt, StmtKind, Expr, ExprKind};

#[test]
fn parse_interpolated_string_dollar_basic() {
    let program = parse_fixture("interpolated_string_dollar_basic");
    let func = &program.functions[0];
    if let Stmt { kind: StmtKind::Print(Expr { kind: ExprKind::Concat(_, _), .. }), .. } = &func.body[1] {
        // ok
    } else {
        panic!("Expected Print(Concat..), got {:?}", func.body[1]);
    }
}

#[test]
fn codegen_interpolated_string_dollar_basic() {
    assert_codegen_matches_snapshot("interpolated_string_dollar_basic");
}

#[test]
fn exec_interpolated_string_dollar_basic() {
    assert_exec_matches_fixture("interpolated_string_dollar_basic");
}