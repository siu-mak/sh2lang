use sh2c::ast::{Stmt, Expr};
mod common;
use common::*;

#[test]
fn parse_list_args_utils() {
    let program = parse_fixture("list_args_utils");
    assert_eq!(program.functions.len(), 2);

    let test_fn = &program.functions[0];
    assert_eq!(test_fn.name, "test");

    // In test(): print(argc())
    assert!(matches!(test_fn.body[0], Stmt::Print(Expr::Argc)));

    // print(arg(1)), print(arg(3))
    assert!(matches!(test_fn.body[1], Stmt::Print(Expr::Arg(1))));
    assert!(matches!(test_fn.body[2], Stmt::Print(Expr::Arg(3))));

    // print(index(args, 0)), print(index(args, 2))
    assert!(matches!(test_fn.body[3],
        Stmt::Print(Expr::Index { .. })
    ));
    assert!(matches!(test_fn.body[4],
        Stmt::Print(Expr::Index { .. })
    ));

    // join(args,"-"), count(args)
    assert!(matches!(test_fn.body[5], Stmt::Print(Expr::Join { .. })));
    assert!(matches!(test_fn.body[6], Stmt::Print(Expr::Count(_))));

    // for a in args { ... }
    assert!(matches!(test_fn.body[7], Stmt::For { .. }));

    let main_fn = &program.functions[1];
    assert_eq!(main_fn.name, "main");

    // let xs = ["a","b","c"]
    if let Stmt::Let { name, value } = &main_fn.body[0] {
        assert_eq!(name, "xs");
        assert!(matches!(value, Expr::List(_)));
    } else { panic!("Expected let xs = [..]"); }

    // count(xs), index(xs,1), join(xs,"-")
    assert!(matches!(main_fn.body[1], Stmt::Print(Expr::Count(_))));
    assert!(matches!(main_fn.body[2], Stmt::Print(Expr::Index { .. })));
    assert!(matches!(main_fn.body[3], Stmt::Print(Expr::Join { .. })));

    // for item in xs
    assert!(matches!(main_fn.body[4], Stmt::For { .. }));

    // call test("x","y","z")
    assert!(matches!(main_fn.body[5], Stmt::Call { .. }));
}

#[test]
fn codegen_list_args_utils() {
    assert_codegen_matches_snapshot("list_args_utils");
}

#[test]
fn exec_list_args_utils() {
    assert_exec_matches_fixture("list_args_utils");
}
