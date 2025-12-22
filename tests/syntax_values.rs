mod common;
use sh2c::ast::{Stmt, Expr};
use common::{parse_fixture, assert_codegen_matches_snapshot, assert_exec_matches_fixture};

#[test]
fn parse_let_args_functions() {
    let program = parse_fixture("let_args");
    let func = &program.functions[0];
    if let Stmt::Print(expr) = &func.body[1] {
        assert!(matches!(expr, Expr::Count(_)));
    } else { panic!("Expected Print(Count)") }

    if let Stmt::Print(expr) = &func.body[2] {
        assert!(matches!(expr, Expr::Index { .. }));
    } else { panic!("Expected Print(Index)") }
    
    if let Stmt::Print(expr) = &func.body[3] {
        assert!(matches!(expr, Expr::Join { .. }));
    } else { panic!("Expected Print(Join)") }
}

#[test]
fn parse_count_list() {
    let program = parse_fixture("count_list_literal");
    let func = &program.functions[0];
    if let Stmt::Print(expr) = &func.body[0] {
        assert!(matches!(expr, Expr::Count(_)));
    } else {
        panic!("Expected Print(Count)");
    }
}

#[test]
fn codegen_let() { assert_codegen_matches_snapshot("let"); }
#[test]
fn codegen_let_args() { assert_codegen_matches_snapshot("let_args"); }
#[test]
fn codegen_count_args() { assert_codegen_matches_snapshot("count_args"); }
#[test]
fn codegen_count_list_literal() { assert_codegen_matches_snapshot("count_list_literal"); }
#[test]
fn codegen_count_list_var() { assert_codegen_matches_snapshot("count_list_var"); }
#[test]
fn codegen_index_list_literal() { assert_codegen_matches_snapshot("index_list_literal"); }
#[test]
fn codegen_index_list_var() { assert_codegen_matches_snapshot("index_list_var"); }
#[test]
fn codegen_join_list_literal() { assert_codegen_matches_snapshot("join_list_literal"); }
#[test]
fn codegen_join_list_var() { assert_codegen_matches_snapshot("join_list_var"); }
#[test]
fn codegen_len_basic() { assert_codegen_matches_snapshot("len_basic"); }
#[test]
fn codegen_number_let_print() { assert_codegen_matches_snapshot("number_let_print"); }

#[test]
fn exec_let_args() { assert_exec_matches_fixture("let_args"); }

// --- Migrated form compare_tests.rs ---
use sh2c::{lexer, parser, ast};

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
    
    // (("a" + b) == c)
    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let func = &program.functions[0];
    
    if let ast::Stmt::If { cond, .. } = &func.body[0] {
        if let ast::Expr::Compare { left, .. } = cond {
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
    use sh2c::{lower, codegen};
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
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
    use sh2c::{lower, codegen};
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);
    let (stdout, _, _) = common::run_bash_script(&bash, &[], &[]);
    assert_eq!(stdout.trim(), "access granted");
}

// --- Migrated from concat_tests.rs ---

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
    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let func = &program.functions[0];
    if let ast::Stmt::Print(ast::Expr::Concat(left, right)) = &func.body[0] {
        if let ast::Expr::Literal(s) = &**right {
            assert_eq!(s, "c");
        } else { panic!("Expected 'c' on right"); }
        if let ast::Expr::Concat(ll, lr) = &**left {
             match (&**ll, &**lr) {
                 (ast::Expr::Literal(l), ast::Expr::Var(r)) => {
                     assert_eq!(l, "a");
                     assert_eq!(r, "b");
                 }
                 _ => panic!("Expected 'a' + b"),
             }
        } else { panic!("Expected nested concat on left"); }
    } else { panic!("Expected Print(Concat)"); }
}

#[test]
fn codegen_concatenation() {
    let src = r#"
        func main() {
            let name = "world"
            print("hello " + name)
        }
    "#;
    use sh2c::{lower, codegen};
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
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
    use sh2c::{lower, codegen};
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);
    let (stdout, _, _) = common::run_bash_script(&bash, &[], &[]);
    assert_eq!(stdout.trim(), "running");
}

// --- Migrated from let_tests.rs ---

#[test]
fn parses_let_statement() {
    let src = r#"
        func main() {
            let x = "y"
        }
    "#;
    use sh2c::{lexer, parser, ast};
    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    let func = &program.functions[0];
    match &func.body[0] {
        ast::Stmt::Let { name, value: ast::Expr::Literal(val) } => {
            assert_eq!(name, "x");
            assert_eq!(val, "y");
        }
        _ => panic!("Expected Let stmt with literal"),
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
    use sh2c::{lexer, parser, lower, codegen};
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
    use sh2c::{lexer, parser, lower, codegen};
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let bash = codegen::emit(&ir);
    let (stdout, _, _) = common::run_bash_script(&bash, &[], &[]);
    assert_eq!(stdout.trim(), "works");
}

#[test]
fn let_alias_variable() {
    let src = r#"
        func main() {
            let a = "origin"
            let b = a
            print(b)
        }
    "#;
    use sh2c::{lexer, parser, lower, codegen};
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
    assert!(out.contains("b=\"$a\""));
}
