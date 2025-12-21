use sh2c::{lexer, parser, lower, codegen};
use sh2c::ast;

#[test]
fn parses_case_stmt() {
    let src = r#"
        func main() {
            case x {
                "a" => { print("A") }
                "b" | _ => { print("Other") }
            }
        }
    "#;

    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let func = &program.functions[0];
    
    match &func.body[0] {
        ast::Stmt::Case { expr, arms } => {
            if let ast::Expr::Var(v) = expr {
                assert_eq!(v, "x");
            } else {
                panic!("Expected var expr");
            }
            assert_eq!(arms.len(), 2);
            
            // First arm: "a"
            assert_eq!(arms[0].patterns.len(), 1);
            if let ast::Pattern::Literal(s) = &arms[0].patterns[0] {
                assert_eq!(s, "a");
            } else {
                panic!("Expected literal pattern");
            }

            // Second arm: "b" | _
            assert_eq!(arms[1].patterns.len(), 2);
            if let ast::Pattern::Literal(s) = &arms[1].patterns[0] {
                assert_eq!(s, "b");
            } else {
                panic!("Expected literal pattern");
            }
            if !matches!(arms[1].patterns[1], ast::Pattern::Wildcard) {
                panic!("Expected wildcard pattern");
            }
        }
        _ => panic!("Expected Case stmt"),
    }
}

#[test]
fn codegen_case() {
    let src = r#"
        func main() {
            case "val" {
                "a" => { print("A") }
                _ => { print("Default") }
            }
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
    
    // Expect: 
    // case "val" in
    //   "a")
    //     echo "A"
    //   ;;
    //   *)
    //     echo "Default"
    //   ;;
    // esac
    
    assert!(out.contains("case \"val\" in"));
    assert!(out.contains("\"a\")"));
    assert!(out.contains("*)"));
    assert!(out.contains("esac"));
}

#[test]
fn exec_case() {
    let src = r#"
        func main() {
            let x = "b"
            case x {
                "a" => { print("A") }
                "b" => { print("B") }
                _ => { print("C") }
            }
        }
    "#;
    
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);
    
    std::fs::write("/tmp/sh2_case_exec.sh", &bash).unwrap();
    
    let out = std::process::Command::new("bash")
        .arg("/tmp/sh2_case_exec.sh")
        .output()
        .unwrap();
        
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim(), "B");
}
