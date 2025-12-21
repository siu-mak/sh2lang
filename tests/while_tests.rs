use sh2c::{lexer, parser, lower, codegen};
use sh2c::ast;

#[test]
fn parses_while_stmt() {
    let src = r#"
        func main() {
            while x == "go" {
                print("looping")
            }
        }
    "#;

    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let func = &program.functions[0];
    
    match &func.body[0] {
        ast::Stmt::While { cond, body } => {
            match cond {
                ast::Expr::Compare { .. } => {},
                _ => panic!("Expected compare expr"),
            }
            assert_eq!(body.len(), 1);
        }
        _ => panic!("Expected While stmt"),
    }
}

#[test]
fn codegen_while() {
    let src = r#"
        func main() {
            while x == "go" {
                print("looping")
            }
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
    
    // Expect: 
    // while [ "$x" = "go" ]; do
    //   echo "looping"
    // done
    
    assert!(out.contains("while [ \"$x\" = \"go\" ]; do"));
    assert!(out.contains("echo \"looping\""));
    assert!(out.contains("done"));
}

#[test]
fn exec_while() {
    // This test ensures the loop runs and terminates
    let src = r#"
        func main() {
            let i = "0"
            while i != "1" {
                print("iter")
                let i = "1"
            }
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);
    
    std::fs::write("/tmp/sh2_while_exec.sh", &bash).unwrap();
    
    let out = std::process::Command::new("bash")
        .arg("/tmp/sh2_while_exec.sh")
        .output()
        .unwrap();
        
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "iter");
}
