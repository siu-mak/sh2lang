use sh2c::lexer;
use sh2c::parser;

#[test]
fn parses_simple_function() {
    let src = r#"
        func hello() {
            run("echo", "hi")
        }
    "#;

    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);

    assert_eq!(ast.functions.len(), 1);
    assert_eq!(ast.functions[0].name, "hello");
}

#[test]
fn parses_if_statement() {
    let src = r#"
        func main() {
            if registry {
                print("configured")
            }
        }
    "#;

    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);

    assert_eq!(ast.functions.len(), 1);

    let func = &ast.functions[0];
    assert_eq!(func.name, "main");
    assert_eq!(func.body.len(), 1);

    match &func.body[0] {
        sh2c::ast::Stmt::If {
            var,
            then_body,
            else_body,
        } => {
            assert_eq!(var, "registry");
            assert_eq!(then_body.len(), 1);
            assert!(else_body.is_none());
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn parses_nested_if() {
    let src = r#"
        func main() {
            if a {
                if b {
                    print("x")
                }
            }
        }
    "#;

    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);

    match &ast.functions[0].body[0] {
        sh2c::ast::Stmt::If { then_body, .. } => {
            assert_eq!(then_body.len(), 1);
            matches!(then_body[0], sh2c::ast::Stmt::If { .. });
        }
        _ => panic!("Expected outer if"),
    }
}
