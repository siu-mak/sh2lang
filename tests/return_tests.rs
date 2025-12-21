use sh2c::{lexer, parser, lower, codegen};
use sh2c::ast;

#[test]
fn parses_return_stmt() {
    let src = r#"
        func main() {
            return
            return "0"
        }
    "#;

    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let func = &program.functions[0];
    
    match &func.body[0] {
        ast::Stmt::Return(None) => {},
        _ => panic!("Expected Return(None) stmt"),
    }
    
    match &func.body[1] {
        ast::Stmt::Return(Some(ast::Expr::Literal(s))) if s == "0" => {},
        _ => panic!("Expected Return(Some(...)) stmt"),
    }
}

#[test]
fn codegen_return() {
    let src = std::fs::read_to_string("tests/fixtures/return_basic.sh2").unwrap();
    let expected = std::fs::read_to_string("tests/fixtures/return_basic.sh.expected").unwrap();
    
    let tokens = lexer::lex(&src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
    
    assert_eq!(out.trim(), expected.trim());
}

#[test]
fn exec_return() {
    let src = r#"
        func main() {
            print("before")
            return
            print("after")
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);
    
    std::fs::write("/tmp/sh2_return_exec.sh", &bash).unwrap();
    
    let out = std::process::Command::new("sh")
        .arg("/tmp/sh2_return_exec.sh")
        .output()
        .unwrap();
        
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "before");
}
