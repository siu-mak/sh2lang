use sh2c::{lexer, parser, lower, codegen};
use sh2c::ast;

#[test]
fn parses_comparison() {
    let src = r#"
        func main() {
            if a == "b" {
                print("match")
            }
        }
    "#;

    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let func = &program.functions[0];
    
    match &func.body[0] {
        ast::Stmt::If { cond, .. } => {
            if let ast::Expr::Compare { left, op, right } = cond {
                assert_eq!(op, &ast::CompareOp::Eq);
                // Check operands
                matches!(**left, ast::Expr::Var(_));
                matches!(**right, ast::Expr::Literal(_));
            } else {
                panic!("Expected Compare expr in If cond");
            }
        }
        _ => panic!("Expected If stmt"),
    }
}

#[test]
fn precedence_compare_concat() {
    let src = r#"
        func main() {
            if "a" + b == c {
                print("ok")
            }
        }
    "#;
    
    // Should parse as ((("a" + b) == c))
    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let func = &program.functions[0];
    
    if let ast::Stmt::If { cond, .. } = &func.body[0] {
        if let ast::Expr::Compare { left, .. } = cond {
             // Left side should be Concat
             match &**left {
                 ast::Expr::Concat(..) => {},
                 _ => panic!("Expected Concat on left of Compare"),
             }
        } else {
            panic!("Expected Compare expression");
        }
    }
}

#[test]
fn codegen_comparison() {
    let src = r#"
        func main() {
            let x = "foo"
            if x != "bar" {
                print("ne")
            }
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
    
    // Expect: if [ "$x" != "bar" ]; then
    assert!(out.contains("[ \"$x\" != \"bar\" ]"));
}

#[test]
fn exec_comparison() {
    let src = r#"
        func main() {
            let user = "admin"
            if user == "admin" {
                print("access granted")
            }
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);
    
    std::fs::write("/tmp/sh2_compare_exec.sh", &bash).unwrap();
    
    let out = std::process::Command::new("bash")
        .arg("/tmp/sh2_compare_exec.sh")
        .output()
        .unwrap();
        
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "access granted");
}
