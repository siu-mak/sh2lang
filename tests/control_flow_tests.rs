use sh2c::{lexer, parser, lower, codegen};
use sh2c::ast;

#[test]
fn parses_break_continue() {
    let src = r#"
        func main() {
            while "x" == "x" {
                break
                continue
            }
        }
    "#;

    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let func = &program.functions[0];
    
    // Check main body
    match &func.body[0] {
        ast::Stmt::While { body, .. } => {
            assert_eq!(body.len(), 2);
            assert!(matches!(body[0], ast::Stmt::Break));
            assert!(matches!(body[1], ast::Stmt::Continue));
        },
        _ => panic!("Expected while loop"),
    }
}

#[test]
fn codegen_break_continue() {
    let src = r#"
        func main() {
            while "x" == "x" {
                break
                continue
            }
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
    
    assert!(out.contains("break"));
    assert!(out.contains("continue"));
}

#[test]
fn exec_break() {
    let src = r#"
        func main() {
            let x = "0"
            while x == "0" {
                print("once")
                break
                print("never")
            }
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);
    
    std::fs::write("/tmp/sh2_break_exec.sh", &bash).unwrap();
    
    let out = std::process::Command::new("bash")
        .arg("/tmp/sh2_break_exec.sh")
        .output()
        .unwrap();
        
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "once");
}

#[test]
fn exec_continue() {
    // Should print "start" then "end" is skipped, then loop finishes
    // Emulating loop count with logic is tricky without arithmetic
    // So we'll trigger continue once then break
    let src = r#"
        func main() {
            let i = "0"
            while i == "0" {
                print("start")
                let i = "1" 
                continue
                print("skipped")
            }
            print("done")
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);
    
    std::fs::write("/tmp/sh2_continue_exec.sh", &bash).unwrap();
    
    let out = std::process::Command::new("bash")
        .arg("/tmp/sh2_continue_exec.sh")
        .output()
        .unwrap();
        
    let stdout = String::from_utf8_lossy(&out.stdout);
    
    // Expect "start\ndone"
    let lines: Vec<&str> = stdout.trim().split_whitespace().collect();
    assert_eq!(lines, vec!["start", "done"]);
}
