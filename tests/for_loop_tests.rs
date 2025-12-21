use sh2c::{lexer, parser, lower, codegen};
use sh2c::ast;

#[test]
fn parses_for_stmt() {
    let src = r#"
        func main() {
            for x in ("a", "b") {
                print(x)
            }
        }
    "#;

    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let func = &program.functions[0];
    
    match &func.body[0] {
        ast::Stmt::For { var, items, body } => {
            assert_eq!(var, "x");
            assert_eq!(items.len(), 2);
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected For stmt"),
    }
}

#[test]
fn codegen_for() {
    let src = std::fs::read_to_string("tests/fixtures/for_basic.sh2").unwrap();
    let expected = std::fs::read_to_string("tests/fixtures/for_basic.sh.expected").unwrap();
    
    let tokens = lexer::lex(&src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
    
    assert_eq!(out.trim(), expected.trim());
}

#[test]
fn exec_for() {
    let src = r#"
        func main() {
            for x in ("a", "b") {
                print(x)
            }
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);
    
    std::fs::write("/tmp/sh2_for_exec.sh", &bash).unwrap();
    
    let out = std::process::Command::new("sh") // Explicitly use /bin/sh as per requirements
        .arg("/tmp/sh2_for_exec.sh")
        .output()
        .unwrap();
        
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "a\nb");
}
