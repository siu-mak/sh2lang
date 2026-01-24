use sh2c::ast::ExprKind;
use sh2c::ast::StmtKind;
use sh2c::ast::{CompareOp, Expr, Stmt};
mod common;
use common::*;

#[test]
fn parse_args_ops() {
    let program = parse_fixture("args_ops");
    assert_eq!(program.functions.len(), 2);

    let show = &program.functions[0];
    let main = &program.functions[1];

    // show: first stmt is if argc() == 3
    if let Stmt {
        node: StmtKind::If { cond, .. },
        ..
    } = &show.body[0]
    {
        if let Expr {
            node: ExprKind::Compare { left, op, right },
            ..
        } = cond
        {
            assert_eq!(*op, CompareOp::Eq);
            assert!(matches!(
                **left,
                Expr {
                    node: ExprKind::Argc,
                    ..
                }
            ));
            assert!(matches!(
                **right,
                Expr {
                    node: ExprKind::Number(3),
                    ..
                }
            ));
        } else {
            panic!("Expected Compare");
        }
    } else {
        panic!("Expected If");
    }

    // show: arg(2) == "y"
    if let Stmt {
        node: StmtKind::If { cond, .. },
        ..
    } = &show.body[1]
    {
        if let Expr {
            node: ExprKind::Compare { left, .. },
            ..
        } = cond
        {
            if let Expr {
                node: ExprKind::Arg(arg_expr),
                ..
            } = &**left
            {
                if let Expr {
                    node: ExprKind::Number(2),
                    ..
                } = &**arg_expr
                {
                    // arg(2) found
                } else {
                    panic!("Expected arg(2)");
                }
            } else {
                panic!("Expected Arg");
            }
        } else {
            panic!("Expected Compare");
        }
    } else {
        panic!("Expected If");
    }

    // show: index(args,0) == "x"
    if let Stmt {
        node: StmtKind::If { cond, .. },
        ..
    } = &show.body[2]
    {
        if let Expr {
            node: ExprKind::Compare { left, .. },
            ..
        } = cond
        {
            assert!(matches!(
                **left,
                Expr {
                    node: ExprKind::Index { .. },
                    ..
                }
            ));
        } else {
            panic!("Expected Compare");
        }
    } else {
        panic!("Expected If");
    }

    // show: print(join(args,","))
    assert!(matches!(
        show.body[3],
        Stmt {
            node: StmtKind::Print(Expr {
                node: ExprKind::Join { .. },
                ..
            }),
            ..
        }
    ));

    // main: show("x","y","z")
    assert!(
        matches!(main.body[0], Stmt { node: StmtKind::Call { ref name, .. }, .. } if name == "show")
    );
}

#[test]
fn codegen_args_ops() {
    assert_codegen_matches_snapshot("args_ops");
}

#[test]
fn exec_args_ops() {
    assert_exec_matches_fixture("args_ops");
}
