use sh2c::ast::{Expr, Function, Program, Stmt};
use sh2c::lexer::lex;
use sh2c::parser::parse;

#[test]
fn parse_for_list_var() {
    let src = include_str!("fixtures/for_list_var.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);

    let main_func = &program.functions[0];
    assert_eq!(main_func.name, "main");

    // let xs = ["a", "b", "c"]
    // for x in xs { print(x) }

    assert_eq!(main_func.body.len(), 2);

    match &main_func.body[0] {
        Stmt::Let { name, value } => {
            assert_eq!(name, "xs");
            if let Expr::List(elems) = value {
                assert_eq!(elems.len(), 3);
            } else {
                panic!("Expected List expr");
            }
        }
        _ => panic!("Expected Let stmt"),
    }

    match &main_func.body[1] {
        Stmt::For { var, items, body } => {
            assert_eq!(var, "x");
            assert_eq!(items.len(), 1);
            assert_eq!(items[0], Expr::Var("xs".to_string()));
            
            assert_eq!(body.len(), 1);
            match &body[0] {
                Stmt::Print(Expr::Var(v)) => assert_eq!(v, "x"),
                _ => panic!("Expected Print(Var(x)) in loop body"),
            }
        }
        _ => panic!("Expected For stmt"),
    }
}

#[test]
fn parse_pipe_basic() {
    let src = include_str!("fixtures/pipe_basic.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::Pipe(_)));
}

#[test]
fn parse_case_wildcard() {
    let src = include_str!("fixtures/case_wildcard.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[1], Stmt::Case { .. }));
}

#[test]
fn parse_if_bool_and() {
    let src = include_str!("fixtures/if_bool_and.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    if let Stmt::If { cond, .. } = &func.body[0] {
        assert!(matches!(cond, Expr::And(..)));
    } else {
        panic!("Expected If");
    }
}

#[test]
fn parse_exists_check() {
    let src = include_str!("fixtures/exists_check.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    if let Stmt::If { cond, .. } = &func.body[0] {
        assert!(matches!(cond, Expr::Exists(..)));
    } else {
        panic!("Expected If");
    }
}

#[test]
fn parse_while_basic() {
    let src = include_str!("fixtures/while_basic.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[1], Stmt::While { .. }));
}

#[test]
fn parse_return_basic() {
    let src = include_str!("fixtures/return_basic.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[1], Stmt::Return(_)));
}

#[test]
fn parse_exit_basic() {
    let src = include_str!("fixtures/exit_basic.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[1], Stmt::Exit(_)));
}

#[test]
fn parse_with_env() {
    let src = include_str!("fixtures/with_env.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::WithEnv { .. }));
}

#[test]
fn parse_cd_basic() {
    let src = include_str!("fixtures/cd_basic.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::Cd { .. }));
}

#[test]
fn parse_sh_raw() {
    let src = include_str!("fixtures/sh_raw.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::Sh(_)));
}

#[test]
fn parse_call_func() {
    let src = include_str!("fixtures/call_func.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[1]; // main is second
    assert!(matches!(func.body[0], Stmt::Call { .. }));
}

#[test]
fn parse_subshell_basic() {
    let src = include_str!("fixtures/subshell_basic.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::Subshell { .. }));
}

#[test]
fn parse_group_basic() {
    let src = include_str!("fixtures/group_basic.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::Group { .. }));
}

#[test]
fn parse_spawn_run() {
    let src = include_str!("fixtures/spawn_run.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::Spawn { .. }));
}

#[test]
fn parse_wait_all() {
    let src = include_str!("fixtures/wait_all.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[2], Stmt::Wait(_)));
}

#[test]
fn parse_try_catch_basic() {
    let src = include_str!("fixtures/try_catch_basic.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::TryCatch { .. }));
}

#[test]
fn parse_export_unset() {
    let src = include_str!("fixtures/export_unset.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::Export { .. }));
    assert!(matches!(func.body[2], Stmt::Unset { .. }));
}

#[test]
fn parse_source_basic() {
    let src = include_str!("fixtures/source_basic.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[1], Stmt::Source { .. }));
}

#[test]
fn parse_print_args() {
    let src = include_str!("fixtures/print_args.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    if let Stmt::Print(expr) = &func.body[0] {
        assert!(matches!(expr, Expr::Args));
    } else {
        panic!("Expected Print(Args)");
    }
}

#[test]
fn parse_if_bool_literals() {
    let src = include_str!("fixtures/if_true_literal.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    if let Stmt::If { cond, .. } = &func.body[0] {
        assert!(matches!(cond, Expr::Bool(true)));
    } else {
        panic!("Expected If(Bool(true))");
    }
}

#[test]
fn parse_count_list() {
    let src = include_str!("fixtures/count_list_literal.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    if let Stmt::Print(expr) = &func.body[0] {
        assert!(matches!(expr, Expr::Count(_)));
    } else {
        panic!("Expected Print(Count)");
    }
}

#[test]
fn parse_let_args_functions() {
    let src = include_str!("fixtures/let_args.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    // print(count(xs)) -> Stmt::Print(Expr::Count(..))
    // print(index(xs, 0)) -> Stmt::Print(Expr::Index(..))
    // print(join(xs, ",")) -> Stmt::Print(Expr::Join(..))
    
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
fn parse_run_args() {
    let src = include_str!("fixtures/run_args.sh2");
    let tokens = lex(src);
    let program = parse(&tokens);
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::Run(..)));
}
