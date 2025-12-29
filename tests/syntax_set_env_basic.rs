use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
mod common;
use common::*;
use sh2c::ast::{Expr, LValue, Stmt};

#[test]
fn parse_set_env_basic() {
    let program = parse_fixture("set_env_basic");
    let func = &program.functions[0];
    assert_eq!(func.body.len(), 3);

    match &func.body[0] {
        Stmt {
            node: StmtKind::Set { target, .. },
            ..
        } => {
            assert!(matches!(target, LValue::Env(name) if name == "FOO"));
        }
        _ => panic!("Expected Set env"),
    }

    match &func.body[1] {
        Stmt {
            node: StmtKind::Print(e),
            ..
        } => assert!(matches!(
            e,
            Expr {
                node: ExprKind::Env(_),
                ..
            }
        )),
        _ => panic!("Expected Print(env(\"FOO\"))"),
    }

    match &func.body[2] {
        Stmt {
            node: StmtKind::Run(_),
            ..
        }
        | Stmt {
            node: StmtKind::Pipe(_),
            ..
        } => {}
        _ => panic!("Expected Run or Pipe"),
    }
}

#[test]
fn codegen_set_env_basic() {
    assert_codegen_matches_snapshot("set_env_basic");
}

#[test]
fn exec_set_env_basic() {
    assert_exec_matches_fixture("set_env_basic");
}
