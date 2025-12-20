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


#[test]
fn if_executes_when_var_is_set() {
    let src = r#"
        func main() {
            if TESTVAR {
                run("echo", "yes")
            }
        }
    "#;

    let bash = {
        let tokens = lexer::lex(src);
        let ast = parser::parse(&tokens);
        let ir = lower::lower(ast);
        codegen::emit(&ir)
    };

    fs::write("/tmp/sh2_if_test.sh", &bash).unwrap();

    let output = Command::new("bash")
        .env("TESTVAR", "1")
        .arg("/tmp/sh2_if_test.sh")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "yes");
}


#[test]
fn if_does_not_execute_when_var_is_empty() {
    let src = r#"
        func main() {
            if TESTVAR {
                run("echo", "yes")
            }
        }
    "#;

    let bash = {
        let tokens = lexer::lex(src);
        let ast = parser::parse(&tokens);
        let ir = lower::lower(ast);
        codegen::emit(&ir)
    };

    fs::write("/tmp/sh2_if_empty_test.sh", &bash).unwrap();

    let output = Command::new("bash")
        .arg("/tmp/sh2_if_empty_test.sh")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "");
}
