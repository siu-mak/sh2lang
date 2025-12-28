mod common;
use common::*;
use sh2c::ast::{Stmt, StmtKind, Expr, ExprKind};

#[test]
fn parse_interpolated_string_escape() {
    let program = parse_fixture("interpolated_string_escape");
    let func = &program.functions[0];
    // First print: "literal: \${name}"
    if let Stmt { kind: StmtKind::Print(Expr { kind: ExprKind::Literal(s), .. }), .. } = &func.body[1] {
        assert!(s.contains("${name}"));
    } else {
        panic!("Expected literal for first print, got {:?}", func.body[1]);
    }
    // Second print: "dollar: \$"
    if let Stmt { kind: StmtKind::Print(Expr { kind: ExprKind::Literal(s), .. }), .. } = &func.body[2] {
         assert!(s.contains("$"));
    } else {
        panic!("Expected literal for second print");
    }
}

#[test]
fn codegen_interpolated_string_escape() {
    assert_codegen_matches_snapshot("interpolated_string_escape");
}

#[test]
fn exec_interpolated_string_escape() {
    assert_exec_matches_fixture("interpolated_string_escape");
}