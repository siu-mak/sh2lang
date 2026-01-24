use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};
mod common;
use common::*;

#[test]
fn parse_list_args_utils() {
    let program = parse_fixture("list_args_utils");
    assert_eq!(program.functions.len(), 2);

    let test_fn = &program.functions[0];
    assert_eq!(test_fn.name, "test");

    // In test(): print(argc())
    assert!(matches!(
        test_fn.body[0],
        Stmt {
            node: StmtKind::Print(Expr {
                node: ExprKind::Argc,
                ..
            }),
            ..
        }
    ));

    // print(arg(1)), print(arg(3))
    if let Stmt {
        node: StmtKind::Print(Expr {
            node: ExprKind::Arg(arg_expr),
            ..
        }),
        ..
    } = &test_fn.body[1]
    {
        if let Expr {
            node: ExprKind::Number(1),
            ..
        } = &**arg_expr
        {
            // arg(1) found
        } else {
            panic!("Expected arg(1)");
        }
    } else {
        panic!("Expected print(arg(1))");
    }
    
    if let Stmt {
        node: StmtKind::Print(Expr {
            node: ExprKind::Arg(arg_expr),
            ..
        }),
        ..
    } = &test_fn.body[2]
    {
        if let Expr {
            node: ExprKind::Number(3),
            ..
        } = &**arg_expr
        {
            // arg(3) found
        } else {
            panic!("Expected arg(3)");
        }
    } else {
        panic!("Expected print(arg(3))");
    }

    // print(index(args, 0)), print(index(args, 2))
    assert!(matches!(
        test_fn.body[3],
        Stmt {
            node: StmtKind::Print(Expr {
                node: ExprKind::Index { .. },
                ..
            }),
            ..
        }
    ));
    assert!(matches!(
        test_fn.body[4],
        Stmt {
            node: StmtKind::Print(Expr {
                node: ExprKind::Index { .. },
                ..
            }),
            ..
        }
    ));

    // join(args,"-"), count(args)
    assert!(matches!(
        test_fn.body[5],
        Stmt {
            node: StmtKind::Print(Expr {
                node: ExprKind::Join { .. },
                ..
            }),
            ..
        }
    ));
    assert!(matches!(
        test_fn.body[6],
        Stmt {
            node: StmtKind::Print(Expr {
                node: ExprKind::Count(_),
                ..
            }),
            ..
        }
    ));

    // for a in args { ... }
    assert!(matches!(
        test_fn.body[7],
        Stmt {
            node: StmtKind::For { .. },
            ..
        }
    ));

    let main_fn = &program.functions[1];
    assert_eq!(main_fn.name, "main");

    // let xs = ["a","b","c"]
    if let Stmt {
        node: StmtKind::Let { name, value },
        ..
    } = &main_fn.body[0]
    {
        assert_eq!(name, "xs");
        assert!(matches!(
            value,
            Expr {
                node: ExprKind::List(_),
                ..
            }
        ));
    } else {
        panic!("Expected let xs = [..]");
    }

    // count(xs), index(xs,1), join(xs,"-")
    assert!(matches!(
        main_fn.body[1],
        Stmt {
            node: StmtKind::Print(Expr {
                node: ExprKind::Count(_),
                ..
            }),
            ..
        }
    ));
    assert!(matches!(
        main_fn.body[2],
        Stmt {
            node: StmtKind::Print(Expr {
                node: ExprKind::Index { .. },
                ..
            }),
            ..
        }
    ));
    assert!(matches!(
        main_fn.body[3],
        Stmt {
            node: StmtKind::Print(Expr {
                node: ExprKind::Join { .. },
                ..
            }),
            ..
        }
    ));

    // for item in xs
    assert!(matches!(
        main_fn.body[4],
        Stmt {
            node: StmtKind::For { .. },
            ..
        }
    ));

    // call test("x","y","z")
    assert!(matches!(
        main_fn.body[5],
        Stmt {
            node: StmtKind::Call { .. },
            ..
        }
    ));
}

#[test]
fn codegen_list_args_utils() {
    assert_codegen_matches_snapshot("list_args_utils");
}

#[test]
fn exec_list_args_utils() {
    assert_exec_matches_fixture("list_args_utils");
}
