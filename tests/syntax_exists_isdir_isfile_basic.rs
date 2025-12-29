mod common;
use common::*;
use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};

#[test]
fn parse_exists_isdir_isfile_basic() {
    let program = parse_fixture("exists_isdir_isfile_basic");
    let func = &program.functions[0];
    assert_eq!(func.body.len(), 2);

    // stmt0: run(...)
    if let Stmt {
        kind: StmtKind::Run(_),
        ..
    } = &func.body[0]
    {
    } else {
        panic!("Expected Run");
    }

    // stmt1: if exists(...) && is_file(...) && is_dir(...)
    if let Stmt {
        kind: StmtKind::If { cond, .. },
        ..
    } = &func.body[1]
    {
        // Don’t overfit structure; just ensure it’s an && chain containing these nodes.
        fn contains_exists(e: &Expr) -> bool {
            match e {
                Expr {
                    kind: ExprKind::Exists(_),
                    ..
                } => true,
                Expr {
                    kind: ExprKind::And(a, b),
                    ..
                }
                | Expr {
                    kind: ExprKind::Or(a, b),
                    ..
                } => contains_exists(a) || contains_exists(b),
                Expr {
                    kind: ExprKind::Not(x),
                    ..
                } => contains_exists(x),
                Expr {
                    kind: ExprKind::Compare { left, right, .. },
                    ..
                } => contains_exists(left) || contains_exists(right),
                _ => false,
            }
        }
        fn contains_is_file(e: &Expr) -> bool {
            match e {
                Expr {
                    kind: ExprKind::IsFile(_),
                    ..
                } => true,
                Expr {
                    kind: ExprKind::And(a, b),
                    ..
                }
                | Expr {
                    kind: ExprKind::Or(a, b),
                    ..
                } => contains_is_file(a) || contains_is_file(b),
                Expr {
                    kind: ExprKind::Not(x),
                    ..
                } => contains_is_file(x),
                Expr {
                    kind: ExprKind::Compare { left, right, .. },
                    ..
                } => contains_is_file(left) || contains_is_file(right),
                _ => false,
            }
        }
        fn contains_is_dir(e: &Expr) -> bool {
            match e {
                Expr {
                    kind: ExprKind::IsDir(_),
                    ..
                } => true,
                Expr {
                    kind: ExprKind::And(a, b),
                    ..
                }
                | Expr {
                    kind: ExprKind::Or(a, b),
                    ..
                } => contains_is_dir(a) || contains_is_dir(b),
                Expr {
                    kind: ExprKind::Not(x),
                    ..
                } => contains_is_dir(x),
                Expr {
                    kind: ExprKind::Compare { left, right, .. },
                    ..
                } => contains_is_dir(left) || contains_is_dir(right),
                _ => false,
            }
        }

        assert!(contains_exists(cond));
        assert!(contains_is_file(cond));
        assert!(contains_is_dir(cond));
    } else {
        panic!("Expected If");
    }
}

#[test]
fn codegen_exists_isdir_isfile_basic() {
    assert_codegen_matches_snapshot("exists_isdir_isfile_basic");
}

#[test]
fn exec_exists_isdir_isfile_basic() {
    assert_exec_matches_fixture("exists_isdir_isfile_basic");
}
