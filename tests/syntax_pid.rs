use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
use sh2c::ast::{self, Expr, Stmt};
mod common;
use common::*;

#[test]
fn parse_pid_basic() {
    let program = parse_fixture("pid_basic");
    let func = &program.functions[0];
    // spawn run(...), let p = pid(), wait(p), print(status())

    // Check 2nd stmt: let p = pid()
    if let Stmt {
        kind: StmtKind::Let { name, value },
        ..
    } = &func.body[1]
    {
        assert_eq!(name, "p");
        assert!(matches!(
            value,
            Expr {
                kind: ExprKind::Pid,
                ..
            }
        ));
    } else {
        panic!("Expected Let p = Pid");
    }
}

#[test]
fn codegen_pid_basic() {
    assert_codegen_matches_snapshot("pid_basic");
}

#[test]
fn exec_pid_basic() {
    assert_exec_matches_fixture("pid_basic");
}
