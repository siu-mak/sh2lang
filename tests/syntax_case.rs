mod common;
use sh2c::ast::Stmt;
use common::{parse_fixture, assert_codegen_matches_snapshot, assert_exec_matches_fixture};

#[test]
fn parse_case_wildcard() {
    let program = parse_fixture("case_wildcard");
    let func = &program.functions[0];
    assert!(matches!(func.body[1], Stmt::Case { .. }));
}

#[test]
fn codegen_case_basic() {
    assert_codegen_matches_snapshot("case_basic");
}

#[test]
fn codegen_case_wildcard() {
    assert_codegen_matches_snapshot("case_wildcard");
}

#[test]
fn exec_case_wildcard() {
    assert_exec_matches_fixture("case_wildcard");
}
