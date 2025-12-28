use sh2c::ast::StmtKind;
mod common;
use common::*;
use sh2c::ast::{ExprKind, Stmt};

#[test]
fn parse_argc_argv0_basic() {
    let program = parse_fixture("argc_argv0_basic");
    let func_f = &program.functions[0];
    let func_main = &program.functions[1];

    // f has two If statements
    assert_eq!(func_f.body.len(), 2);
    if let Stmt { kind: StmtKind::If { .. }, .. } = &func_f.body[0] {} else { panic!("Expected If"); }
    if let Stmt { kind: StmtKind::If { .. }, .. } = &func_f.body[1] {} else { panic!("Expected If"); }

    // main calls f(...)
    assert_eq!(func_main.body.len(), 1);
    if let Stmt { kind: StmtKind::Call { name, args }, .. } = &func_main.body[0] {
        assert_eq!(name, "f");
        assert_eq!(args.len(), 2);
    } else {
        panic!("Expected Call");
    }
}

#[test]
fn codegen_argc_argv0_basic() {
    assert_codegen_matches_snapshot("argc_argv0_basic");
}

#[test]
fn exec_argc_argv0_basic() {
    assert_exec_matches_fixture("argc_argv0_basic");
}