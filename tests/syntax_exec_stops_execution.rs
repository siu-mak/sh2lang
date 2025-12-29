use sh2c::ast::StmtKind;
mod common;
use common::*;
use sh2c::ast::Stmt;

#[test]
fn parse_exec_stops_execution() {
    let program = parse_fixture("exec_stops_execution");
    let func = &program.functions[0];
    assert_eq!(func.body.len(), 3);

    // Check stmts
    if let Stmt {
        kind: StmtKind::Print(_),
        ..
    } = &func.body[0]
    {
    } else {
        panic!("Expected Print");
    }
    if let Stmt {
        kind: StmtKind::Exec(_),
        ..
    } = &func.body[1]
    {
    } else {
        panic!("Expected Exec");
    }
    if let Stmt {
        kind: StmtKind::Print(_),
        ..
    } = &func.body[2]
    {
    } else {
        panic!("Expected Print");
    }
}

#[test]
fn codegen_exec_stops_execution() {
    assert_codegen_matches_snapshot("exec_stops_execution");
}

#[test]
fn exec_exec_stops_execution() {
    assert_exec_matches_fixture("exec_stops_execution");
}
