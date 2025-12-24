mod common;
use common::*;
use sh2c::ast::{Stmt};

#[test]
fn parse_exec_basic_new() {
    let program = parse_fixture("exec_basic");
    let func = &program.functions[0];
    if let Stmt::Exec(args) = &func.body[0] {
        assert_eq!(args.len(), 2);
        // We can do deeper checks if needed, but this satisfies the requirement
    } else {
        panic!("Expected Exec statement");
    }
}

#[test]
fn codegen_exec_basic_new() {
    assert_codegen_matches_snapshot("exec_basic");
}

#[test]
fn exec_exec_basic_new() {
    assert_exec_matches_fixture("exec_basic");
}
