mod common;
use common::*;
use sh2c::ast::{Stmt, Expr};

#[test]
fn parse_fs_nonempty_basic() {
    let program = parse_fixture("fs_nonempty_basic");
    let func = &program.functions[0];
    assert_eq!(func.body.len(), 2);

    // stmt0: run(...)
    if let Stmt::Run(_) = &func.body[0] {} else { panic!("Expected Run"); }

    // stmt1: if !is_nonempty(...) && is_nonempty(...)
    if let Stmt::If { cond, .. } = &func.body[1] {
        fn contains_is_nonempty(e: &Expr) -> bool {
            match e {
                Expr::IsNonEmpty(_) => true,
                Expr::And(a,b) | Expr::Or(a,b) => contains_is_nonempty(a) || contains_is_nonempty(b),
                Expr::Not(x) => contains_is_nonempty(x),
                _ => false,
            }
        }
        assert!(contains_is_nonempty(cond));
    } else {
        panic!("Expected If");
    }
}

#[test]
fn codegen_fs_nonempty_basic() {
    assert_codegen_matches_snapshot("fs_nonempty_basic");
}

#[test]
fn exec_fs_nonempty_basic() {
    assert_exec_matches_fixture("fs_nonempty_basic");
}
