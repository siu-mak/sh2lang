mod common;
use common::*;
use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};

#[test]
fn parse_fs_non_empty_basic() {
    let program = parse_fixture("fs_non_empty_basic");
    let func = &program.functions[0];

    // Check first if condition: is_non_empty("nonempty")
    if let Stmt {
        node: StmtKind::If { cond, .. },
        ..
    } = &func.body[1]
    {
        if let Expr {
            node: ExprKind::IsNonEmpty(path),
            ..
        } = cond
        {
            if let Expr {
                node: ExprKind::Literal(s),
                ..
            } = &**path
            {
                assert_eq!(s, "nonempty");
            } else {
                panic!("Expected string literal");
            }
        } else {
            panic!("Expected IsNonEmpty");
        }
    } else {
        panic!("Expected If");
    }

    // Check later if condition: !is_non_empty("empty")
    // The 2nd if stmt is index 2 (run, if, if)
    if let Stmt {
        node: StmtKind::If { cond, .. },
        ..
    } = &func.body[2]
    {
        if let Expr {
            node: ExprKind::Not(inner),
            ..
        } = cond
        {
            if let Expr {
                node: ExprKind::IsNonEmpty(path),
                ..
            } = &**inner
            {
                if let Expr {
                    node: ExprKind::Literal(s),
                    ..
                } = &**path
                {
                    assert_eq!(s, "empty");
                } else {
                    panic!("Expected 'empty' literal");
                }
            } else {
                panic!("Expected IsNonEmpty inside Not");
            }
        } else {
            panic!("Expected Not");
        }
    }
}

#[test]
fn codegen_fs_non_empty_basic() {
    assert_codegen_matches_snapshot("fs_non_empty_basic");
}

#[test]
fn exec_fs_non_empty_basic() {
    assert_exec_matches_fixture("fs_non_empty_basic");
}
