use sh2c::ast::{Stmt, Expr};
mod common;
use common::*;

#[test]
fn parse_interp_basic() {
  let program = parse_fixture("interp_basic");
  let func = &program.functions[0];
  if let Stmt::Print(Expr::Concat(_, _)) = &func.body[1] {
    // ok
  } else {
    panic!("Expected interpolated string to desugar to Concat");
  }
}
#[test] fn codegen_interp_basic() { assert_codegen_matches_snapshot("interp_basic"); }
#[test] fn exec_interp_basic() { assert_exec_matches_fixture("interp_basic"); }
