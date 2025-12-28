mod common;
use common::*;
use sh2c::ast::{Stmt, StmtKind, Expr, ExprKind};

#[test]
fn parse_bool_str_basic() {
    let program = parse_fixture("bool_str_basic");
    let func = &program.functions[0];
    // print(bool_str(is_non_empty("nonempty")))
    // func body: [run, print(...)]
    if let Stmt { kind: StmtKind::Print(expr), .. } = &func.body[1] {
        if let Expr { kind: ExprKind::BoolStr(inner), .. } = expr {
             if let Expr { kind: ExprKind::IsNonEmpty(path), .. } = &**inner {
                 match &**path {
                     Expr { kind: ExprKind::Literal(s), .. } => assert_eq!(s, "nonempty"),
                     _ => panic!("Expected literal"),
                 }
             } else { panic!("Expected IsNonEmpty"); }
        } else { panic!("Expected BoolStr"); }
    } else { panic!("Expected Print"); }
}

#[test]
fn codegen_bool_str_basic() {
    assert_codegen_matches_snapshot("bool_str_basic");
}

#[test]
fn exec_bool_str_basic() {
    assert_exec_matches_fixture("bool_str_basic");
}