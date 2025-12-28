use sh2c::ast::StmtKind;
use sh2c::ast::ExprKind;
use sh2c::ast::{Stmt, Expr, ArithOp};
mod common;
use common::*;

#[test]
fn parse_concat_amp_precedence() {
    let program = parse_fixture("concat_amp_precedence");
    let func = &program.functions[0];

    // let s = "n=" & 1 + 2
    if let Stmt { kind: StmtKind::Let { value, .. }, .. } = &func.body[0] {
        if let Expr { kind: ExprKind::Concat(left, right), .. } = value {
            assert!(matches!(**left, Expr { kind: ExprKind::Literal(ref s), .. } if s == "n="));
            // right should be arithmetic (1 + 2)
            assert!(matches!(**right, Expr { kind: ExprKind::Arith { op: ArithOp::Add, .. }, .. }));
        } else {
            panic!("Expected Concat at top-level");
        }
    } else { panic!("Expected Let"); }
}

#[test]
fn codegen_concat_amp_precedence() {
    assert_codegen_matches_snapshot("concat_amp_precedence");
}

#[test]
fn exec_concat_amp_precedence() {
    assert_exec_matches_fixture("concat_amp_precedence");
}