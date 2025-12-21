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

#[test]
fn print_err_writes_to_stderr() {
    let src = r#"
        func main() {
            print_err("fail")
        }
    "#;

    let bash = {
        let t = sh2c::lexer::lex(src);
        let a = sh2c::parser::parse(&t);
        let i = sh2c::lower::lower(a);
        sh2c::codegen::emit(&i)
    };

    std::fs::write("/tmp/sh2_err.sh", &bash).unwrap();

    let out = std::process::Command::new("bash")
        .arg("/tmp/sh2_err.sh")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("fail"));
}
#[test]
fn else_executes_when_var_is_empty() {
    let src = r#"
        func main() {
            if TESTVAR {
                print("yes")
            } else {
                print("no")
            }
        }
    "#;

    let bash = {
        let t = sh2c::lexer::lex(src);
        let a = sh2c::parser::parse(&t);
        let i = sh2c::lower::lower(a);
        sh2c::codegen::emit(&i)
    };

    std::fs::write("/tmp/sh2_else_test.sh", &bash).unwrap();

    let out = std::process::Command::new("bash")
        .arg("/tmp/sh2_else_test.sh")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "no");
}
#[test]
fn parses_empty_function_body() {
    let src = r#"
        func main() {
        }
    "#;

    let tokens = sh2c::lexer::lex(src);
    let ast = sh2c::parser::parse(&tokens);

    assert_eq!(ast.functions.len(), 1);
    assert_eq!(ast.functions[0].body.len(), 0);
}
#[test]
fn parses_multiple_functions() {
    let src = r#"
        func a() { print("x") }
        func b() { print("y") }
    "#;

    let tokens = sh2c::lexer::lex(src);
    let ast = sh2c::parser::parse(&tokens);

    assert_eq!(ast.functions.len(), 2);
    assert_eq!(ast.functions[0].name, "a");
    assert_eq!(ast.functions[1].name, "b");
}
