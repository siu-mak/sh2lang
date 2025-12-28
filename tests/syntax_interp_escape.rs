use sh2c::ast::{Stmt, StmtKind, Expr, ExprKind};
mod common;
use common::*;

#[test]
fn parse_interp_escape() {
  let program = parse_fixture("interp_escape");
  let func = &program.functions[0];
  // should be a plain literal, not concat/var
  if let Stmt { kind: StmtKind::Print(Expr { kind: ExprKind::Literal(s), .. }), .. } = &func.body[1] {
    assert!(s.contains("${name}"));
  } else {
    panic!("Expected escaped interpolation to remain a literal");
  }
}
#[test] fn codegen_interp_escape() { assert_codegen_matches_snapshot("interp_escape"); }
#[test] fn exec_interp_escape() { assert_exec_matches_fixture("interp_escape"); }