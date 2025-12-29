mod common;
use common::{assert_codegen_matches_snapshot, assert_exec_matches_fixture, parse_fixture};
use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};

#[test]
fn parse_let_args_functions() {
    let program = parse_fixture("let_args");
    let func = &program.functions[0];
    if let Stmt {
        kind: StmtKind::Print(expr),
        ..
    } = &func.body[1]
    {
        assert!(matches!(
            expr,
            Expr {
                kind: ExprKind::Count(_),
                ..
            }
        ));
    } else {
        panic!("Expected Print(Count)")
    }

    if let Stmt {
        kind: StmtKind::Print(expr),
        ..
    } = &func.body[2]
    {
        assert!(matches!(
            expr,
            Expr {
                kind: ExprKind::Index { .. },
                ..
            }
        ));
    } else {
        panic!("Expected Print(Index)")
    }

    if let Stmt {
        kind: StmtKind::Print(expr),
        ..
    } = &func.body[3]
    {
        assert!(matches!(
            expr,
            Expr {
                kind: ExprKind::Join { .. },
                ..
            }
        ));
    } else {
        panic!("Expected Print(Join)")
    }
}

#[test]
fn parse_count_list() {
    let program = parse_fixture("count_list_literal");
    let func = &program.functions[0];
    if let Stmt {
        kind: StmtKind::Print(expr),
        ..
    } = &func.body[0]
    {
        assert!(matches!(
            expr,
            Expr {
                kind: ExprKind::Count(_),
                ..
            }
        ));
    } else {
        panic!("Expected Print(Count)");
    }
}

#[test]
fn codegen_let() {
    assert_codegen_matches_snapshot("let");
}
#[test]
fn codegen_let_args() {
    assert_codegen_matches_snapshot("let_args");
}
#[test]
fn codegen_count_args() {
    assert_codegen_matches_snapshot("count_args");
}
#[test]
fn codegen_count_list_literal() {
    assert_codegen_matches_snapshot("count_list_literal");
}
#[test]
fn codegen_count_list_var() {
    assert_codegen_matches_snapshot("count_list_var");
}
#[test]
fn codegen_index_list_literal() {
    assert_codegen_matches_snapshot("index_list_literal");
}
#[test]
fn codegen_index_list_var() {
    assert_codegen_matches_snapshot("index_list_var");
}
#[test]
fn codegen_join_list_literal() {
    assert_codegen_matches_snapshot("join_list_literal");
}
#[test]
fn codegen_join_list_var() {
    assert_codegen_matches_snapshot("join_list_var");
}
#[test]
fn codegen_len_basic() {
    assert_codegen_matches_snapshot("len_basic");
}
#[test]
fn codegen_number_let_print() {
    assert_codegen_matches_snapshot("number_let_print");
}

#[test]
fn exec_let_args() {
    assert_exec_matches_fixture("let_args");
}

// --- Migrated form compare_tests.rs ---
use sh2c::{ast, lexer, parser};

#[test]
fn parses_comparison() {
    let src = r#"
        func main() {
            if a == "b" {
                print("match")
            }
        }
    "#;

    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = sh2c::lexer::lex(&sm, "test");
    let program = parser::parse(&tokens, &sm, "test");
    let func = &program.functions[0];

    match &func.body[0] {
        ast::Stmt {
            kind: StmtKind::If { cond, .. },
            ..
        } => {
            if let ast::Expr {
                kind: ExprKind::Compare { left, op, right },
                ..
            } = cond
            {
                assert_eq!(op, &ast::CompareOp::Eq);
                // Check operands
                matches!(
                    **left,
                    ast::Expr {
                        kind: ExprKind::Var(_),
                        ..
                    }
                );
                matches!(
                    **right,
                    ast::Expr {
                        kind: ExprKind::Literal(_),
                        ..
                    }
                );
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
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = sh2c::lexer::lex(&sm, "test");
    let program = parser::parse(&tokens, &sm, "test");
    let func = &program.functions[0];

    if let ast::Stmt {
        kind: StmtKind::If { cond, .. },
        ..
    } = &func.body[0]
    {
        if let ast::Expr {
            kind: ExprKind::Compare { left, .. },
            ..
        } = cond
        {
            match &**left {
                ast::Expr {
                    kind:
                        ExprKind::Arith {
                            op: ast::ArithOp::Add,
                            ..
                        },
                    ..
                } => {}
                _ => panic!("Expected Arith(Add) on left of Compare"),
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
    use sh2c::{codegen, lower};
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = sh2c::lexer::lex(&sm, "test");
    let ast = parser::parse(&tokens, &sm, "test");
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
    assert!(out.contains("[ \"$x\" != 'bar' ]"));
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
    use sh2c::{codegen, lower};
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = sh2c::lexer::lex(&sm, "test");
    let ast = parser::parse(&tokens, &sm, "test");
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

    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = sh2c::lexer::lex(&sm, "test");
    let program = parser::parse(&tokens, &sm, "test");
    let func = &program.functions[0];

    match &func.body[0] {
        // Concat is now parsed as Arith(Add)
        ast::Stmt {
            kind:
                StmtKind::Let {
                    value:
                        ast::Expr {
                            kind: ExprKind::Arith { left, op, right },
                            ..
                        },
                    ..
                },
            ..
        } => {
            assert_eq!(*op, ast::ArithOp::Add);
            match (&**left, &**right) {
                (
                    ast::Expr {
                        kind: ExprKind::Literal(l),
                        ..
                    },
                    ast::Expr {
                        kind: ExprKind::Literal(r),
                        ..
                    },
                ) => {
                    assert_eq!(l, "a");
                    assert_eq!(r, "b");
                }
                _ => panic!("Expected Literal + Literal"),
            }
        }
        _ => panic!("Expected Arith expression for concat"),
    }
}

#[test]
fn parses_chained_concatenation() {
    let src = r#"
        func main() {
            print("a" + b + "c")
        }
    "#;
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = sh2c::lexer::lex(&sm, "test");
    let program = parser::parse(&tokens, &sm, "test");
    let func = &program.functions[0];
    // "a" + b + "c" -> (("a" + b) + "c")
    if let ast::Stmt {
        kind:
            StmtKind::Print(ast::Expr {
                kind: ExprKind::Arith { left, op, right },
                ..
            }),
        ..
    } = &func.body[0]
    {
        assert_eq!(*op, ast::ArithOp::Add);
        if let ast::Expr {
            kind: ExprKind::Literal(s),
            ..
        } = &**right
        {
            assert_eq!(s, "c");
        } else {
            panic!("Expected 'c' on right");
        }
        if let ast::Expr {
            kind:
                ExprKind::Arith {
                    left: ll,
                    op: lop,
                    right: lr,
                },
            ..
        } = &**left
        {
            assert_eq!(*lop, ast::ArithOp::Add);
            match (&**ll, &**lr) {
                (
                    ast::Expr {
                        kind: ExprKind::Literal(l),
                        ..
                    },
                    ast::Expr {
                        kind: ExprKind::Var(r),
                        ..
                    },
                ) => {
                    assert_eq!(l, "a");
                    assert_eq!(r, "b");
                }
                _ => panic!("Expected 'a' + b"),
            }
        } else {
            panic!("Expected nested arith on left");
        }
    } else {
        panic!("Expected Print(Arith)");
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
    use sh2c::{codegen, lower};
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = sh2c::lexer::lex(&sm, "test");
    let ast = parser::parse(&tokens, &sm, "test");
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
    assert!(out.contains("'hello '\"$name\""));
}

#[test]
fn exec_concatenation() {
    let src = r#"
        func main() {
            let part1 = "run"
            # Use literal to trigger lowering string optimization
            run("echo", part1 + "ning")
        }
    "#;
    use sh2c::{codegen, lower};
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = sh2c::lexer::lex(&sm, "test");
    let ast = parser::parse(&tokens, &sm, "test");
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
    use sh2c::{ast, lexer, parser};
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = sh2c::lexer::lex(&sm, "test");
    let program = parser::parse(&tokens, &sm, "test");
    let func = &program.functions[0];
    match &func.body[0] {
        ast::Stmt {
            kind:
                StmtKind::Let {
                    name,
                    value:
                        ast::Expr {
                            kind: ExprKind::Literal(val),
                            ..
                        },
                },
            ..
        } => {
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
    use sh2c::{codegen, lexer, lower, parser};
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = sh2c::lexer::lex(&sm, "test");
    let ast = parser::parse(&tokens, &sm, "test");
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
    assert!(out.contains("msg='hello'"));
    assert!(out.contains("printf '%s\\n' \"$msg\""));
}

#[test]
fn exec_let_variable() {
    let src = r#"
        func main() {
            let val = "works"
            run("echo", val)
        }
    "#;
    use sh2c::{codegen, lexer, lower, parser};
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = sh2c::lexer::lex(&sm, "test");
    let ast = parser::parse(&tokens, &sm, "test");
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
    use sh2c::{codegen, lexer, lower, parser};
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = sh2c::lexer::lex(&sm, "test");
    let ast = parser::parse(&tokens, &sm, "test");
    let ir = lower::lower(ast);
    let out = codegen::emit(&ir);
    assert!(out.contains("b=\"$a\""));
}

#[test]
fn parse_arith_precedence() {
    let program = parse_fixture("arith_precedence");
    let func = &program.functions[0];
    if let Stmt {
        kind: StmtKind::Let { value, .. },
        ..
    } = &func.body[0]
    {
        // 1 + 2 * 3 -> Add(1, Mul(2, 3))
        if let Expr {
            kind: ExprKind::Arith { left, op, right },
            ..
        } = value
        {
            assert_eq!(*op, sh2c::ast::ArithOp::Add);
            assert!(matches!(
                **left,
                Expr {
                    kind: ExprKind::Number(1),
                    ..
                }
            ));
            if let Expr {
                kind:
                    ExprKind::Arith {
                        left: l2,
                        op: op2,
                        right: r2,
                    },
                ..
            } = &**right
            {
                assert_eq!(*op2, sh2c::ast::ArithOp::Mul);
                assert!(matches!(
                    **l2,
                    Expr {
                        kind: ExprKind::Number(2),
                        ..
                    }
                ));
                assert!(matches!(
                    **r2,
                    Expr {
                        kind: ExprKind::Number(3),
                        ..
                    }
                ));
            } else {
                panic!("Expected Mul on right");
            }
        } else {
            panic!("Expected Arith expression");
        }
    } else {
        panic!("Expected Let stmt");
    }
}
#[test]
fn codegen_arith_precedence() {
    assert_codegen_matches_snapshot("arith_precedence");
}
#[test]
fn exec_arith_precedence() {
    assert_exec_matches_fixture("arith_precedence");
}

#[test]
fn parse_index_var_index() {
    let program = parse_fixture("index_var_index");
    let func = &program.functions[0];
    if let Stmt {
        kind:
            StmtKind::Print(Expr {
                kind: ExprKind::Index { index, .. },
                ..
            }),
        ..
    } = &func.body[2]
    {
        if let Expr {
            kind: ExprKind::Var(s),
            ..
        } = &**index
        {
            assert_eq!(s, "i");
        } else {
            panic!("Expected Var index");
        }
    } else {
        panic!("Expected Print(Index)");
    }
}
#[test]
fn codegen_index_var_index() {
    assert_codegen_matches_snapshot("index_var_index");
}
#[test]
fn exec_index_var_index() {
    assert_exec_matches_fixture("index_var_index");
}

#[test]
fn parse_index_arith_index() {
    let program = parse_fixture("index_arith_index");
    let func = &program.functions[0];
    if let Stmt {
        kind:
            StmtKind::Print(Expr {
                kind: ExprKind::Index { index, .. },
                ..
            }),
        ..
    } = &func.body[2]
    {
        // i + 2
        if let Expr {
            kind: ExprKind::Arith { left, op, right },
            ..
        } = &**index
        {
            assert_eq!(*op, sh2c::ast::ArithOp::Add);
            // check operands if needed
        } else {
            panic!("Expected Arith index");
        }
    } else {
        panic!("Expected Print(Index)");
    }
}
#[test]
fn codegen_index_arith_index() {
    assert_codegen_matches_snapshot("index_arith_index");
}
#[test]
fn exec_index_arith_index() {
    assert_exec_matches_fixture("index_arith_index");
}

#[test]
fn codegen_index_list_literal_expr() {
    assert_codegen_matches_snapshot("index_list_literal_expr");
}
#[test]
fn exec_index_list_literal_expr() {
    assert_exec_matches_fixture("index_list_literal_expr");
}

#[test]
fn parse_arith_unary_minus() {
    let program = parse_fixture("arith_unary_minus");
    let func = &program.functions[0];
    if let Stmt {
        kind: StmtKind::Let { value, .. },
        ..
    } = &func.body[0]
    {
        // -1 + 2 -> Add(Sub(0, 1), 2)
        if let Expr {
            kind: ExprKind::Arith { left, op, .. },
            ..
        } = value
        {
            assert_eq!(*op, sh2c::ast::ArithOp::Add);
            if let Expr {
                kind: ExprKind::Arith { left, op, right },
                ..
            } = &**left
            {
                assert_eq!(*op, sh2c::ast::ArithOp::Sub);
                assert!(matches!(
                    **left,
                    Expr {
                        kind: ExprKind::Number(0),
                        ..
                    }
                ));
                assert!(matches!(
                    **right,
                    Expr {
                        kind: ExprKind::Number(1),
                        ..
                    }
                ));
            } else {
                panic!("Expected Sub(0,1) on left");
            }
        } else {
            panic!("Expected Arith");
        }
    }
}
#[test]
fn codegen_arith_unary_minus() {
    assert_codegen_matches_snapshot("arith_unary_minus");
}
#[test]
fn exec_arith_unary_minus() {
    assert_exec_matches_fixture("arith_unary_minus");
}

#[test]
fn parse_index_args() {
    let program = parse_fixture("index_args");
    let func = &program.functions[0];
    if let Stmt {
        kind:
            StmtKind::Print(Expr {
                kind: ExprKind::Index { list, .. },
                ..
            }),
        ..
    } = &func.body[0]
    {
        assert!(matches!(
            **list,
            Expr {
                kind: ExprKind::Args,
                ..
            }
        ));
    } else {
        panic!("Expected Print(Index(Args))");
    }
}
#[test]
fn codegen_index_args() {
    assert_codegen_matches_snapshot("index_args");
}
#[test]
fn exec_index_args() {
    assert_exec_matches_fixture("index_args");
}

#[test]
fn parse_join_args() {
    let program = parse_fixture("join_args");
    let func = &program.functions[0];
    if let Stmt {
        kind:
            StmtKind::Print(Expr {
                kind: ExprKind::Join { list, .. },
                ..
            }),
        ..
    } = &func.body[0]
    {
        assert!(matches!(
            **list,
            Expr {
                kind: ExprKind::Args,
                ..
            }
        ));
    } else {
        panic!("Expected Print(Join(Args))");
    }
}
#[test]
fn codegen_join_args() {
    assert_codegen_matches_snapshot("join_args");
}
#[test]
fn exec_join_args() {
    assert_exec_matches_fixture("join_args");
}
