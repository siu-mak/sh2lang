
use sh2c::parser::parse;
use sh2c::lexer::lex;

#[test]
fn test_pipe_parser_error() {
    let code = r#"
        func main() {
            run("sh", "-c", "true") | print("not a run")
        }
    "#;
    let tokens = lex(code);
    let result = std::panic::catch_unwind(move || {
        parse(&tokens);
    });
    assert!(result.is_err(), "Parser should panic on non-run pipe segment");
    
    let err = result.err().unwrap();
    let expected = "expected run(...) after '|' in pipeline";
    
    if let Some(msg) = err.downcast_ref::<&str>() {
        assert_eq!(*msg, expected);
    } else if let Some(msg) = err.downcast_ref::<String>() {
        assert_eq!(msg, expected);
    } else {
        panic!("Unknown panic payload type");
    }
}
