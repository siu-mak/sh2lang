use sh2c::ast::{Stmt, Expr, CompareOp};
mod common;
use common::*;

#[test]
fn parse_args_ops() {
    let program = parse_fixture("args_ops");
    assert_eq!(program.functions.len(), 2);

    let show = &program.functions[0];
    let main = &program.functions[1];

    // show: first stmt is if argc() == 3
    if let Stmt::If { cond, .. } = &show.body[0] {
        if let Expr::Compare { left, op, right } = cond {
            assert_eq!(*op, CompareOp::Eq);
            assert!(matches!(**left, Expr::Argc));
            assert!(matches!(**right, Expr::Number(3)));
        } else { panic!("Expected Compare"); }
    } else { panic!("Expected If"); }

    // show: arg(2) == "y"
    if let Stmt::If { cond, .. } = &show.body[1] {
        if let Expr::Compare { left, .. } = cond {
            assert!(matches!(**left, Expr::Arg(2)));
        } else { panic!("Expected Compare"); }
    } else { panic!("Expected If"); }

    // show: index(args,0) == "x"
    if let Stmt::If { cond, .. } = &show.body[2] {
        if let Expr::Compare { left, .. } = cond {
            assert!(matches!(**left, Expr::Index { .. }));
        } else { panic!("Expected Compare"); }
    } else { panic!("Expected If"); }

    // show: print(join(args,","))
    assert!(matches!(show.body[3], Stmt::Print(Expr::Join { .. })));

    // main: show("x","y","z")
    assert!(matches!(main.body[0], Stmt::Call { ref name, .. } if name == "show"));
}

#[test]
fn codegen_args_ops() { assert_codegen_matches_snapshot("args_ops"); }

#[test]
fn exec_args_ops() { assert_exec_matches_fixture("args_ops"); }
