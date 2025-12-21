use sh2c::{lexer, parser, lower, codegen};
use sh2c::ast;

#[test]
fn parses_concatenation() {
    let src = r#"
        func main() {
            let x = "a" + "b"
        }
    "#;

    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let func = &program.functions[0];
    
    match &func.body[0] {
        ast::Stmt::Let { value: ast::Expr::Concat(left, right), .. } => {
            match (&**left, &**right) {
                (ast::Expr::Literal(l), ast::Expr::Literal(r)) => {
                    assert_eq!(l, "a");
                    assert_eq!(r, "b");
                }
                _ => panic!("Expected Literal + Literal"),
            }
        }
        _ => panic!("Expected Concat expression"),
    }
}

#[test]
fn parses_chained_concatenation() {
    let src = r#"
        func main() {
            print("a" + b + "c")
        }
    "#;
    
    // (("a" + b) + "c") - left associative
    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let func = &program.functions[0];
    
    if let ast::Stmt::Print(ast::Expr::Concat(left, right)) = &func.body[0] {
        // right should be "c"
        if let ast::Expr::Literal(s) = &**right {
            assert_eq!(s, "c");
        } else {
            panic!("Expected 'c' on right");
        }
        
        // left should be ("a" + b)
        if let ast::Expr::Concat(ll, lr) = &**left {
             match (&**ll, &**lr) {
                 (ast::Expr::Literal(l), ast::Expr::Var(r)) => {
                     assert_eq!(l, "a");
                     assert_eq!(r, "b");
                 }
                 _ => panic!("Expected 'a' + b"),
             }
        } else {
            panic!("Expected nested concat on left");
        }
    } else {
        panic!("Expected Print(Concat)");
    }
}

#[test]
fn codegen_concatenation() {
    let src = r#"
        func main() {
            let name = "world"
            print("hello " + name)
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
    
    // Check for correct shell juxtaposition
    // "echo" "hello " "$name" -> "echo" "hello ""$name" -- wait, implementation is simple string concat of emitted values
    // emit_val("hello ") -> "\"hello \""
    // emit_val(name) -> "\"$name\""
    // Result: "\"hello \"\"$name\""
    assert!(out.contains("\"hello \"\"$name\""));
}

#[test]
fn exec_concatenation() {
    let src = r#"
        func main() {
            let part1 = "run"
            let part2 = "ning"
            run("echo", part1 + part2)
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);
    
    std::fs::write("/tmp/sh2_concat_exec.sh", &bash).unwrap();
    
    let out = std::process::Command::new("bash")
        .arg("/tmp/sh2_concat_exec.sh")
        .output()
        .unwrap();
        
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "running");
}
