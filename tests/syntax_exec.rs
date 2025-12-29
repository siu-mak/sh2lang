use sh2c::ast::StmtKind;
use sh2c::ast::Stmt;
mod common;
use common::*;

#[test]
fn parse_exec_basic() {
    let program = parse_fixture("exec_basic");
    let func = &program.functions[0];
    assert_eq!(func.body.len(), 3);

    if let Stmt {
        node: StmtKind::Print(_),
        ..
    } = &func.body[0]
    {
    } else {
        panic!("Expected Print");
    }
    if let Stmt {
        node: StmtKind::Exec(_),
        ..
    } = &func.body[1]
    {
    } else {
        panic!("Expected Exec");
    }
    if let Stmt {
        node: StmtKind::Print(_),
        ..
    } = &func.body[2]
    {
    } else {
        panic!("Expected Print");
    }
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
