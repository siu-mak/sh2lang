use sh2c::lexer::lex;
use sh2c::parser::parse;

#[test]
fn test_pipe_parser_error() {
    let code = r#"
        func main() {
            run("sh", "-c", "true") | print("not a run")
        }
    "#;
    use sh2c::span::SourceMap;
    let sm = SourceMap::new(code.to_string());
    let tokens = lex(&sm, code).expect("Lexing failed");
    
    let result = parse(&tokens, &sm, "test_pipe_parser_error.sh2");
    assert!(result.is_err(), "Parser should return error on non-run pipe segment");
    
    let err = result.unwrap_err();
    let expected = "expected run(...) after '|' in pipeline";
    assert!(err.msg.contains(expected), "Error message did not contain expected text: {}", err.msg);
}
