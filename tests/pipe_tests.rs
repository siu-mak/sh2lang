use sh2c::{lexer, parser, lower, codegen};
use sh2c::ast;

#[test]
fn parses_pipe() {
    let src = r#"
        func main() {
            run("ls") | run("wc")
        }
    "#;

    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let func = &program.functions[0];
    
    match &func.body[0] {
        ast::Stmt::Pipe(segments) => {
            assert_eq!(segments.len(), 2);
            // Check first segment args ("ls")
            if let ast::Expr::Literal(s) = &segments[0][0] {
                 assert_eq!(s, "ls");
            } else {
                panic!("Expected ls literal");
            }
            // Check second segment args ("wc")
            if let ast::Expr::Literal(s) = &segments[1][0] {
                 assert_eq!(s, "wc");
            } else {
                panic!("Expected wc literal");
            }
        }
        _ => panic!("Expected Pipe stmt"),
    }
}

#[test]
fn codegen_pipe() {
    let src = r#"
        func main() {
            run("echo", "hello") | run("rev")
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
    
    // Expect: "echo" "hello" | "rev"
    assert!(out.contains(r#""echo" "hello" | "rev""#));
}

#[test]
fn exec_pipe() {
    let src = r#"
        func main() {
            run("echo", "hello") | run("rev")
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);
    
    std::fs::write("/tmp/sh2_pipe_exec.sh", &bash).unwrap();
    
    let out = std::process::Command::new("bash")
        .arg("/tmp/sh2_pipe_exec.sh")
        .output()
        .unwrap();
        
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "olleh");
}
