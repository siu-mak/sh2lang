use sh2c::{lexer, parser, lower, codegen};
use sh2c::ast;

#[test]
fn parses_command_sub() {
    let src = r#"
        func main() {
            let x = $(run("echo", "foo"))
        }
    "#;

    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let func = &program.functions[0];
    
    match &func.body[0] {
        ast::Stmt::Let { value, .. } => {
            if let ast::Expr::Command(args) = value {
                assert_eq!(args.len(), 2);
                if let ast::Expr::Literal(s) = &args[0] {
                    assert_eq!(s, "echo");
                }
            } else {
                panic!("Expected Command Expr");
            }
        }
        _ => panic!("Expected Let stmt"),
    }
}

#[test]
fn codegen_command_sub() {
    let src = r#"
        func main() {
            print("now: " + $(run("date")))
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
    
    // Expect: $( "date" )
    assert!(out.contains("$( \"date\" )"));
}

#[test]
fn exec_command_sub() {
    let src = r#"
        func main() {
            let me = $(run("echo", "tester"))
            print("hello " + me)
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);
    
    std::fs::write("/tmp/sh2_cmd_sub.sh", &bash).unwrap();
    
    let out = std::process::Command::new("bash")
        .arg("/tmp/sh2_cmd_sub.sh")
        .output()
        .unwrap();
        
    let stdout = String::from_utf8_lossy(&out.stdout);
    // "hello tester" (with trailing newline from echo potentially stripped by command substitution? No, bash preserves newlines inside variable but echo adds one at end)
    // Actually:
    // me=$( echo "tester" ) -> "tester" (trailing newline usually stripped by substitution in bash)
    // print("hello " + me) -> "echo" "hello ""$me" -> "hello tester"
    assert_eq!(stdout.trim(), "hello tester");
}
