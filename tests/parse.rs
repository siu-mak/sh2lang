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
