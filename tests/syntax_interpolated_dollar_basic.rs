mod common;
use common::*;
use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};

#[test]
fn parse_interpolated_dollar_basic() {
    let program = parse_fixture("interpolated_dollar_basic");
    let func = &program.functions[0];
    if let Stmt {
        kind:
            StmtKind::Print(Expr {
                kind: ExprKind::Concat(_, _),
                ..
            }),
        ..
    } = &func.body[1]
    {
        // ok
    } else {
        panic!("Expected Print(Concat..), got {:?}", func.body[1]);
    }
}

#[test]
fn codegen_interpolated_dollar_basic() {
    assert_codegen_matches_snapshot("interpolated_dollar_basic");
}

#[test]
fn exec_interpolated_dollar_basic() {
    assert_exec_matches_fixture("interpolated_dollar_basic");
}
