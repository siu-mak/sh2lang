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
    let tokens = lex(&sm, code);
    let result = std::panic::catch_unwind(move || {
        parse(&tokens, &sm, "test_pipe_parser_error.sh2");
    });
    assert!(
        result.is_err(),
        "Parser should panic on non-run pipe segment"
    );

    let err = result.err().unwrap();
    let expected = "expected run(...) after '|' in pipeline";

    if let Some(msg) = err.downcast_ref::<&str>() {
        assert!(msg.contains(expected), "Panic message did not contain expected text: {}", msg);
    } else if let Some(msg) = err.downcast_ref::<String>() {
        assert!(msg.contains(expected), "Panic message did not contain expected text: {}", msg);
    } else {
        panic!("Unknown panic payload type");
    }
}
