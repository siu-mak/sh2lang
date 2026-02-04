mod common;
use common::*;
use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};

#[test]
fn parse_interpolated_dollar_escape() {
    let program = parse_fixture("interpolated_dollar_escape");
    let func = &program.functions[0];
    // First print: "literal: {name}"
    if let Stmt {
        node:
            StmtKind::Print(Expr {
                node: ExprKind::Literal(s),
                ..
            }),
        ..
    } = &func.body[1]
    {
        assert_eq!(s, "literal: {name}");
    } else {
        panic!("Expected literal for first print, got {:?}", func.body[1]);
    }
    // Second print: "dollar: $"
    if let Stmt {
        node:
            StmtKind::Print(Expr {
                node: ExprKind::Literal(s),
                ..
            }),
        ..
    } = &func.body[2]
    {
        assert!(s.contains("$"));
    } else {
        panic!("Expected literal for second print");
    }
    // Third print: "interp: {name}" -> Concat or Var
    if let Stmt {
        node: StmtKind::Print(expr),
        ..
    } = &func.body[3]
    {
        match expr {
            Expr {
                node: ExprKind::Concat(..),
                ..
            }
            | Expr {
                node: ExprKind::Var(_),
                ..
            } => {} // Good
            other => panic!(
                "Expected complex expression for third print, got {:?}",
                other
            ),
        }
    }
}

#[test]
fn codegen_interpolated_dollar_escape() {
    assert_codegen_matches_snapshot("interpolated_dollar_escape");
}

#[test]
fn exec_interpolated_dollar_escape() {
    assert_exec_matches_fixture("interpolated_dollar_escape");
}
