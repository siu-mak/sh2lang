use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
use sh2c::ast::{Expr, RunCall, Stmt};
mod common;
use common::*;

#[test]
fn parse_concat_amp_basic() {
    let program = parse_fixture("concat_amp_basic");
    let func = &program.functions[0];

    // let x = "a" & "b"
    if let Stmt {
        node: StmtKind::Let { name, value },
        ..
    } = &func.body[0]
    {
        assert_eq!(name, "x");
        assert!(matches!(
            value,
            Expr {
                node: ExprKind::Concat(_, _),
                ..
            }
        ));
    } else {
        panic!("Expected Let");
    }

    // run("echo", "c" & "d") => second arg is Concat
    if let Stmt {
        node: StmtKind::Run(RunCall { args, .. }),
        ..
    } = &func.body[2]
    {
        assert!(matches!(
            &args[1],
            Expr {
                node: ExprKind::Concat(_, _),
                ..
            }
        ));
    } else {
        panic!("Expected Run");
    }
}

#[test]
fn codegen_concat_amp_basic() {
    assert_codegen_matches_snapshot("concat_amp_basic");
}

#[test]
fn exec_concat_amp_basic() {
    assert_exec_matches_fixture("concat_amp_basic");
}
