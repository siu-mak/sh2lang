use sh2c::ast::StmtKind;
mod common;
use common::*;
use sh2c::ast::{ExprKind, Stmt};

#[test]
fn parse_uid_ppid_pwd_basic() {
    let program = parse_fixture("uid_ppid_pwd_basic");
    let func = &program.functions[0];
    assert_eq!(func.body.len(), 1);
    if let Stmt {
        kind: StmtKind::If { .. },
        ..
    } = &func.body[0]
    {
    } else {
        panic!("Expected If");
    }
}

#[test]
fn codegen_uid_ppid_pwd_basic() {
    assert_codegen_matches_snapshot("uid_ppid_pwd_basic");
}

#[test]
fn exec_uid_ppid_pwd_basic() {
    assert_exec_matches_fixture("uid_ppid_pwd_basic");
}
