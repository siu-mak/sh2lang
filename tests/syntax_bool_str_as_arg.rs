mod common;
use common::*;
use sh2c::ast::{Stmt, Expr, RunCall, StmtKind, ExprKind};

#[test]
fn parse_bool_str_as_arg() {
    let program = parse_fixture("bool_str_as_arg");
    let func = &program.functions[0];
    // run("printf", "%s\n", bool_str(is_non_empty("empty")))
    // func body: [run, run(...)]
    if let Stmt { kind: StmtKind::Run(RunCall { args, .. }), .. } = &func.body[1] {
         // args: [printf, %s\n, bool_str(...)]
         if let Expr { kind: ExprKind::BoolStr(inner), .. } = &args[2] {
             if let Expr { kind: ExprKind::IsNonEmpty(_), .. } = inner.as_ref() {
                 // ok
             } else { panic!("Expected IsNonEmpty"); }
         } else { panic!("Expected BoolStr third arg"); }
    } else { panic!("Expected Run"); }
}

#[test]
fn codegen_bool_str_as_arg() {
    assert_codegen_matches_snapshot("bool_str_as_arg");
}

#[test]
fn exec_bool_str_as_arg() {
    assert_exec_matches_fixture("bool_str_as_arg");
}