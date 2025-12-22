use sh2c::ast::{Expr, Function, Program, Stmt};
use sh2c::lexer::lex;
use sh2c::parser::parse;

#[test]
fn parse_for_list_var() {
    let src = include_str!("fixtures/for_list_var.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);

    let main_func = &program.functions[0];
    assert_eq!(main_func.name, "main");

    // let xs = ["a", "b", "c"]
    // for x in xs { print(x) }

    assert_eq!(main_func.body.len(), 2);

    match &main_func.body[0] {
        Stmt::Let { name, value } => {
            assert_eq!(name, "xs");
            if let Expr::List(elems) = value {
                assert_eq!(elems.len(), 3);
            } else {
                panic!("Expected List expr");
            }
        }
        _ => panic!("Expected Let stmt"),
    }

    match &main_func.body[1] {
        Stmt::For { var, items, body } => {
            assert_eq!(var, "x");
            assert_eq!(items.len(), 1);
            assert_eq!(items[0], Expr::Var("xs".to_string()));
            
            assert_eq!(body.len(), 1);
            match &body[0] {
                Stmt::Print(Expr::Var(v)) => assert_eq!(v, "x"),
                _ => panic!("Expected Print(Var(x)) in loop body"),
            }
        }
        _ => panic!("Expected For stmt"),
    }
}
