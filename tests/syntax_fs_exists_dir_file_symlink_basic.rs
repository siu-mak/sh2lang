mod common;
use common::*;
use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};

#[test]
fn parse_fs_exists_dir_file_symlink_basic() {
    let program = parse_fixture("fs_exists_dir_file_symlink_basic");
    let func = &program.functions[0];
    assert_eq!(func.body.len(), 2);

    assert!(matches!(
        &func.body[0],
        Stmt {
            node: StmtKind::Run(_),
            ..
        }
    ));

    if let Stmt {
        node: StmtKind::If { cond, .. },
        ..
    } = &func.body[1]
    {
        fn has_pred(e: &Expr) -> (bool, bool, bool, bool) {
            match e {
                Expr {
                    node: ExprKind::Exists(_),
                    ..
                } => (true, false, false, false),
                Expr {
                    node: ExprKind::IsFile(_),
                    ..
                } => (false, true, false, false),
                Expr {
                    node: ExprKind::IsDir(_),
                    ..
                } => (false, false, true, false),
                Expr {
                    node: ExprKind::IsSymlink(_),
                    ..
                } => (false, false, false, true),
                Expr {
                    node: ExprKind::And(a, b),
                    ..
                }
                | Expr {
                    node: ExprKind::Or(a, b),
                    ..
                } => {
                    let (a1, a2, a3, a4) = has_pred(a);
                    let (b1, b2, b3, b4) = has_pred(b);
                    (a1 || b1, a2 || b2, a3 || b3, a4 || b4)
                }
                Expr {
                    node: ExprKind::Not(x),
                    ..
                } => has_pred(x),
                _ => (false, false, false, false),
            }
        }
        let (e, f, d, l) = has_pred(cond);
        assert!(e && f && d && l);
    } else {
        panic!("Expected If");
    }
}

#[test]
fn codegen_fs_exists_dir_file_symlink_basic() {
    assert_codegen_matches_snapshot("fs_exists_dir_file_symlink_basic");
}

#[test]
fn exec_fs_exists_dir_file_symlink_basic() {
    assert_exec_matches_fixture("fs_exists_dir_file_symlink_basic");
}
