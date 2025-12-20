use std::fs;
use sh2c::{lexer, parser, lower, codegen};

fn compile(src: &str) -> String {
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    codegen::emit(&ir)
}

#[test]
fn hello_codegen_matches_snapshot() {
    let src = fs::read_to_string("tests/fixtures/hello.sh2").unwrap();
    let expected = fs::read_to_string("tests/fixtures/hello.sh.expected").unwrap();

    let output = compile(&src);

    assert_eq!(output.trim(), expected.trim());
}
#[test]
fn multiple_run_codegen() {
    let src = std::fs::read_to_string("tests/fixtures/multi_run.sh2").unwrap();
    let expected = std::fs::read_to_string("tests/fixtures/multi_run.sh.expected").unwrap();

    let tokens = sh2c::lexer::lex(&src);
    let ast = sh2c::parser::parse(&tokens);
    let ir = sh2c::lower::lower(ast);
    let output = sh2c::codegen::emit(&ir);

    assert_eq!(output.trim(), expected.trim());
}
