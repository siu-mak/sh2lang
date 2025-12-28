use sh2c::ast::{Stmt, StmtKind, Expr, ExprKind};
mod common;
use common::*;

#[test]
fn parse_dollar_pipe() {
    let program = parse_fixture("dollar_pipe");
    let func = &program.functions[0];

    // stmt0: let out = $( ... | ... ) => Expr { kind: ExprKind::CommandPipe, .. }
    if let Stmt { kind: StmtKind::Let { name, value }, .. } = &func.body[0] {
        assert_eq!(name, "out");
        assert!(matches!(value, Expr { kind: ExprKind::CommandPipe(_), .. }));
    } else {
        panic!("Expected let out = $(...)");
    }

    assert!(matches!(func.body[1], Stmt { kind: StmtKind::Print(Expr { kind: ExprKind::Var(ref v), .. }), .. } if v == "out"));
}

#[test]
fn codegen_dollar_pipe() {
    assert_codegen_matches_snapshot("dollar_pipe");
}

#[test]
fn exec_dollar_pipe() {
    assert_exec_matches_fixture("dollar_pipe");
}