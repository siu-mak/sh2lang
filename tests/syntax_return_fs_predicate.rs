mod common;
use common::*;
use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};

#[test]
fn parse_return_fs_predicate() {
    let program = parse_fixture("return_fs_predicate");
    let check_func = &program.functions[0];
    assert_eq!(check_func.name, "check");
    // return(is_non_empty(path))
    if let Stmt {
        kind: StmtKind::Return(Some(expr)),
        ..
    } = &check_func.body[0]
    {
        if let Expr {
            kind: ExprKind::IsNonEmpty(path),
            ..
        } = expr
        {
            match &**path {
                Expr {
                    kind: ExprKind::Var(name),
                    ..
                } => assert_eq!(name, "path"),
                _ => panic!("Expected path var"),
            }
        } else {
            panic!("Expected IsNonEmpty");
        }
    } else {
        panic!("Expected ReturnStmt");
    }
}

#[test]
fn codegen_return_fs_predicate() {
    assert_codegen_matches_snapshot("return_fs_predicate");
}

#[test]
fn exec_return_fs_predicate() {
    assert_exec_matches_fixture("return_fs_predicate");
}
