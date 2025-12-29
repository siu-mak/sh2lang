use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
use sh2c::ast::{Expr, RunCall, Stmt};
mod common;
use common::*;

#[test]
fn parse_string_quote_escape() {
    let program = parse_fixture("string_quote_escape");
    let func = &program.functions[0];

    // stmt0: print("a\"b") => Expr { node: ExprKind::Literal, .. } containing a quote char
    if let Stmt {
        node: StmtKind::Print(e),
        ..
    } = &func.body[0]
    {
        if let Expr {
            node: ExprKind::Literal(s),
            ..
        } = e
        {
            assert_eq!(s, "a\"b");
        } else {
            panic!("Expected Literal");
        }
    } else {
        panic!("Expected Print");
    }

    // stmt1: run("sh","-c","echo \"hi\"") => third arg contains quote chars
    if let Stmt {
        node: StmtKind::Run(RunCall { args, .. }),
        ..
    } = &func.body[1]
    {
        assert_eq!(args.len(), 3);
        if let Expr {
            node: ExprKind::Literal(s),
            ..
        } = &args[2]
        {
            assert_eq!(s, "echo \"hi\"");
        } else {
            panic!("Expected Literal");
        }
    } else {
        panic!("Expected Run");
    }
}

#[test]
fn codegen_string_quote_escape() {
    assert_codegen_matches_snapshot("string_quote_escape");
}

#[test]
fn exec_string_quote_escape() {
    assert_exec_matches_fixture("string_quote_escape");
}
