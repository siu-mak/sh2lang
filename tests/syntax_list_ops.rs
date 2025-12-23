use sh2c::ast::{Stmt, Expr, CompareOp};
mod common;
use common::*;

#[test]
fn parse_list_ops() {
    let program = parse_fixture("list_ops");
    let func = &program.functions[0];

    // stmt0: let xs = ["a","b","c"]
    if let Stmt::Let { name, value } = &func.body[0] {
        assert_eq!(name, "xs");
        assert!(matches!(value, Expr::List(_)));
    } else { panic!("Expected Let(xs=List)"); }

    // stmt1: if count(xs) == 3 { ... }
    if let Stmt::If { cond, .. } = &func.body[1] {
        if let Expr::Compare { left, op, right } = cond {
            assert_eq!(*op, CompareOp::Eq);
            assert!(matches!(**left, Expr::Count(_)));
            assert!(matches!(**right, Expr::Number(3)));
        } else { panic!("Expected Compare"); }
    } else { panic!("Expected If"); }

    // stmt2: if index(xs,1) == "b" { ... }
    if let Stmt::If { cond, .. } = &func.body[2] {
        if let Expr::Compare { left, op, right } = cond {
            assert_eq!(*op, CompareOp::Eq);
            assert!(matches!(**left, Expr::Index { .. }));
            assert!(matches!(**right, Expr::Literal(ref s) if s == "b"));
        } else { panic!("Expected Compare"); }
    } else { panic!("Expected If"); }

    // stmt3: print(join(xs,"-"))
    assert!(matches!(func.body[3], Stmt::Print(Expr::Join { .. })));
}

#[test]
fn codegen_list_ops() { assert_codegen_matches_snapshot("list_ops"); }

#[test]
fn exec_list_ops() { assert_exec_matches_fixture("list_ops"); }
