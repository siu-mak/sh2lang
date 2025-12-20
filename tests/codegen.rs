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
