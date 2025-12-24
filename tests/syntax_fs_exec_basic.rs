mod common;
use common::*;
use sh2c::ast::{Stmt, Expr};

#[test]
fn parse_fs_exec_basic() {
    let program = parse_fixture("fs_exec_basic");
    let func = &program.functions[0];
    assert_eq!(func.body.len(), 2);

    assert!(matches!(&func.body[0], Stmt::Run(_)));

    if let Stmt::If { cond, .. } = &func.body[1] {
        fn has_is_exec(e: &Expr) -> bool {
            match e {
                Expr::IsExec(_) => true,
                Expr::And(a,b) | Expr::Or(a,b) => has_is_exec(a) || has_is_exec(b),
                Expr::Not(x) => has_is_exec(x),
                _ => false,
            }
        }
        assert!(has_is_exec(cond));
    } else {
        panic!("Expected If");
    }
}

#[test]
fn codegen_fs_exec_basic() {
    assert_codegen_matches_snapshot("fs_exec_basic");
}

#[test]
fn exec_fs_exec_basic() {
    assert_exec_matches_fixture("fs_exec_basic");
}
