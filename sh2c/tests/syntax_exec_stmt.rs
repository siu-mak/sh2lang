use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};
mod common;
use common::*;

#[test]
fn parse_exec_stmt() {
    let program = parse_fixture("exec_stmt");
    let func = &program.functions[0];

    // stmt0: exec("sh", "-c", "echo exec_ok")
    if let Stmt {
        node: StmtKind::Exec(args),
        ..
    } = &func.body[0]
    {
        assert_eq!(args.len(), 3);
        assert!(matches!(args[0], Expr { node: ExprKind::Literal(ref s), .. } if s == "sh"));
        assert!(matches!(args[1], Expr { node: ExprKind::Literal(ref s), .. } if s == "-c"));
        assert!(
            matches!(args[2], Expr { node: ExprKind::Literal(ref s), .. } if s == "echo exec_ok")
        );
    } else {
        panic!("Expected Exec(...)");
    }
}

#[test]
fn codegen_exec_stmt() {
    assert_codegen_matches_snapshot("exec_stmt");
}

#[test]
fn exec_exec_stmt() {
    assert_exec_matches_fixture("exec_stmt");
}
