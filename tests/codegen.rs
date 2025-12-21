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

#[test]
fn if_basic_codegen_matches_snapshot() {
    let src = fs::read_to_string("tests/fixtures/if_basic.sh2").unwrap();
    let expected = fs::read_to_string("tests/fixtures/if_basic.sh.expected").unwrap();

    let output = compile(&src);

    assert_eq!(output.trim(), expected.trim());
}
#[test]
fn print_err_codegen() {
    let src = r#"
        func main() {
            print_err("oops")
        }
    "#;

    let expected = r#"
main() {
  echo "oops" >&2
}

main "$@"
"#;

    let tokens = sh2c::lexer::lex(src);
    let ast = sh2c::parser::parse(&tokens);
    let ir = sh2c::lower::lower(ast);
    let out = sh2c::codegen::emit(&ir);

    assert_eq!(out.trim(), expected.trim());
}

#[test]
fn let_codegen_matches_snapshot() {
    let src = fs::read_to_string("tests/fixtures/let.sh2").unwrap();
    let expected = fs::read_to_string("tests/fixtures/let.sh.expected").unwrap();

    let output = compile(&src);

    assert_eq!(output.trim(), expected.trim());
}
