mod common;
use common::*;
use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};

#[test]
fn parse_fs_predicates_basic() {
    let program = parse_fixture("fs_predicates_basic");
    let func = &program.functions[0];
    assert_eq!(func.body.len(), 2);

    // stmt0: run(...)
    if let Stmt {
        node: StmtKind::Run(_),
        ..
    } = &func.body[0]
    {
    } else {
        panic!("Expected Run");
    }

    // stmt1: if is_symlink(...) && is_exec(...) && is_readable(...) && is_writable(...)
    if let Stmt {
        node: StmtKind::If { cond, .. },
        ..
    } = &func.body[1]
    {
        fn contains_is_symlink(e: &Expr) -> bool {
            match e {
                Expr {
                    node: ExprKind::IsSymlink(_),
                    ..
                } => true,
                Expr {
                    node: ExprKind::And(a, b),
                    ..
                }
                | Expr {
                    node: ExprKind::Or(a, b),
                    ..
                } => contains_is_symlink(a) || contains_is_symlink(b),
                Expr {
                    node: ExprKind::Not(x),
                    ..
                } => contains_is_symlink(x),
                _ => false,
            }
        }
        fn contains_is_exec(e: &Expr) -> bool {
            match e {
                Expr {
                    node: ExprKind::IsExec(_),
                    ..
                } => true,
                Expr {
                    node: ExprKind::And(a, b),
                    ..
                }
                | Expr {
                    node: ExprKind::Or(a, b),
                    ..
                } => contains_is_exec(a) || contains_is_exec(b),
                Expr {
                    node: ExprKind::Not(x),
                    ..
                } => contains_is_exec(x),
                _ => false,
            }
        }
        fn contains_is_readable(e: &Expr) -> bool {
            match e {
                Expr {
                    node: ExprKind::IsReadable(_),
                    ..
                } => true,
                Expr {
                    node: ExprKind::And(a, b),
                    ..
                }
                | Expr {
                    node: ExprKind::Or(a, b),
                    ..
                } => contains_is_readable(a) || contains_is_readable(b),
                Expr {
                    node: ExprKind::Not(x),
                    ..
                } => contains_is_readable(x),
                _ => false,
            }
        }
        fn contains_is_writable(e: &Expr) -> bool {
            match e {
                Expr {
                    node: ExprKind::IsWritable(_),
                    ..
                } => true,
                Expr {
                    node: ExprKind::And(a, b),
                    ..
                }
                | Expr {
                    node: ExprKind::Or(a, b),
                    ..
                } => contains_is_writable(a) || contains_is_writable(b),
                Expr {
                    node: ExprKind::Not(x),
                    ..
                } => contains_is_writable(x),
                _ => false,
            }
        }

        assert!(contains_is_symlink(cond));
        assert!(contains_is_exec(cond));
        assert!(contains_is_readable(cond));
        assert!(contains_is_writable(cond));
    } else {
        panic!("Expected If");
    }
}

#[test]
fn codegen_fs_predicates_basic() {
    assert_codegen_matches_snapshot("fs_predicates_basic");
}

#[test]
fn exec_fs_predicates_basic() {
    assert_exec_matches_fixture("fs_predicates_basic");
}
