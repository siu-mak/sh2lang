use sh2c::ast::{Stmt, StmtKind};
mod common;
use common::*;

#[test]
fn parse_subshell_group_scope() {
    let program = parse_fixture("subshell_group_scope");
    let func = &program.functions[0];

    // stmt0: let x = "0"
    if let Stmt {
        node: StmtKind::Let { .. },
        ..
    } = &func.body[0]
    {
        // OK
    } else {
        panic!("Expected Let at index 0");
    }

    // stmt1: subshell { ... }
    if let Stmt {
        node: StmtKind::Subshell { body },
        ..
    } = &func.body[1]
    {
        assert_eq!(body.len(), 1, "Subshell body should have 1 statement");
    } else {
        panic!("Expected Subshell at index 1");
    }

    // stmt2: print(x)
    if let Stmt {
        node: StmtKind::Print(_),
        ..
    } = &func.body[2]
    {
        // OK
    } else {
        panic!("Expected Print at index 2");
    }

    // stmt3: group { ... }
    if let Stmt {
        node: StmtKind::Group { body },
        ..
    } = &func.body[3]
    {
        assert_eq!(body.len(), 1, "Group body should have 1 statement");
    } else {
        panic!("Expected Group at index 3");
    }

    // stmt4: print(x)
    if let Stmt {
        node: StmtKind::Print(_),
        ..
    } = &func.body[4]
    {
        // OK
    } else {
        panic!("Expected Print at index 4");
    }
}

#[test]
fn codegen_subshell_group_scope() {
    assert_codegen_matches_snapshot("subshell_group_scope");
}

#[test]
fn exec_subshell_group_scope() {
    assert_exec_matches_fixture("subshell_group_scope");
}
