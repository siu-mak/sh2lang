use sh2c::ast::{Stmt, StmtKind};
mod common;
use common::*;

#[test]
fn parse_exec_replace() {
    let program = parse_fixture("exec_replace");
    let func = &program.functions[0];

    assert!(matches!(
        func.body[0],
        Stmt {
            kind: StmtKind::Print(_),
            ..
        }
    ));
    assert!(matches!(
        func.body[1],
        Stmt {
            kind: StmtKind::Exec(_),
            ..
        }
    ));
    assert!(matches!(
        func.body[2],
        Stmt {
            kind: StmtKind::Print(_),
            ..
        }
    ));
}

#[test]
fn codegen_exec_replace() {
    assert_codegen_matches_snapshot("exec_replace");
}

#[test]
fn exec_exec_replace() {
    assert_exec_matches_fixture("exec_replace");
}
