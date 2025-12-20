use std::fs;
use std::process::Command;
use sh2c::{lexer, parser, lower, codegen};

#[test]
fn hello_executes_correctly() {
    let src = r#"
        func main() {
            run("echo", "hello")
        }
    "#;

    let bash = {
        let tokens = lexer::lex(src);
        let ast = parser::parse(&tokens);
        let ir = lower::lower(ast);
        codegen::emit(&ir)
    };

    fs::write("/tmp/sh2_test.sh", &bash).unwrap();

    let output = Command::new("bash")
        .arg("/tmp/sh2_test.sh")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(stdout.trim(), "hello");
}
