use sh2c::{lexer, parser, lower, codegen};
use sh2c::ast;

#[test]
fn parses_let_statement() {
    let src = r#"
        func main() {
            let x = "y"
        }
    "#;

    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let func = &program.functions[0];
    
    match &func.body[0] {
        ast::Stmt::Let { name, value } => {
            assert_eq!(name, "x");
            assert_eq!(value, "y");
        }
        _ => panic!("Expected Let stmt"),
    }
}

#[test]
fn codegen_let_and_usage() {
    let src = r#"
        func main() {
            let msg = "hello"
            print(msg)
        }
    "#;
    
    // compiler pipeline helpers
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
    
    assert!(out.contains("msg=\"hello\""));
    assert!(out.contains("echo \"$msg\""));
}

#[test]
fn exec_let_variable() {
    let src = r#"
        func main() {
            let val = "works"
            run("echo", val)
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);
    
    std::fs::write("/tmp/sh2_let_exec.sh", &bash).unwrap();
    
    let out = std::process::Command::new("bash")
        .arg("/tmp/sh2_let_exec.sh")
        .output()
        .unwrap();
        
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "works");
}
