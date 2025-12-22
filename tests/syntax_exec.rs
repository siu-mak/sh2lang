use sh2c::ast::{self, Stmt};
mod common;
use common::*;

#[test]
fn parse_exec_basic() {
    let program = parse_fixture("exec_basic");
    let func = &program.functions[0];
    if let Stmt::Exec(args) = &func.body[0] {
        assert_eq!(args.len(), 2);
    } else { panic!("Expected Exec statement"); }
}

#[test]
fn codegen_exec_basic() {
    assert_codegen_matches_snapshot("exec_basic");
}

#[test]
fn exec_exec_basic() {
    assert_exec_matches_fixture("exec_basic");
}

#[test]
fn exec_exec_exit_status() {
    assert_exec_matches_fixture("exec_exit_status");
}
