mod common;
use common::{assert_codegen_matches_snapshot, assert_exec_matches_fixture, parse_fixture};
use sh2c::ast::{Expr, ExprKind, Stmt, StmtKind};

#[test]
fn parse_simple_function() {
    let program = parse_fixture("hello");
    let func = &program.functions[0];
    assert_eq!(func.name, "main");
    assert!(func.params.is_empty());
}

#[test]
fn parse_call_func() {
    let program = parse_fixture("call_func");
    let func = &program.functions[1]; // main is second
    assert!(matches!(
        func.body[0],
        Stmt {
            kind: StmtKind::Call { .. },
            ..
        }
    ));
}

#[test]
fn parse_return_basic() {
    let program = parse_fixture("return_basic");
    let func = &program.functions[0];
    assert!(matches!(
        func.body[1],
        Stmt {
            kind: StmtKind::Return(_),
            ..
        }
    ));
}

#[test]
fn codegen_call_func() {
    assert_codegen_matches_snapshot("call_func");
}
#[test]
fn codegen_call_args() {
    assert_codegen_matches_snapshot("call_args");
}
#[test]
fn codegen_func_args_basic() {
    assert_codegen_matches_snapshot("func_args_basic");
}
#[test]
fn codegen_func_params_basic() {
    assert_codegen_matches_snapshot("func_params_basic");
}
#[test]
fn codegen_func_params_10() {
    assert_codegen_matches_snapshot("func_params_10");
}
#[test]
fn codegen_return_basic() {
    assert_codegen_matches_snapshot("return_basic");
}
#[test]
fn codegen_return_arg() {
    assert_codegen_matches_snapshot("return_arg");
}
#[test]
fn codegen_return_number() {
    assert_codegen_matches_snapshot("return_number");
}
#[test]
fn codegen_return_len() {
    assert_codegen_matches_snapshot("return_len");
}
#[test]
fn codegen_hello() {
    assert_codegen_matches_snapshot("hello");
}

#[test]
fn exec_hello() {
    assert_exec_matches_fixture("hello_exec");
}

#[test]
fn parse_simple_function_inline() {
    let src = r#"
        func hello() {
            run("echo", "hi")
        }
    "#;
    use sh2c::{lexer, parser};
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = sh2c::lexer::lex(&sm, "test");
    let ast = parser::parse(&tokens, &sm, "test");
    assert_eq!(ast.functions.len(), 1);
    assert_eq!(ast.functions[0].name, "hello");
}

// --- Return Status Backfill ---

#[test]
fn parse_return_status() {
    let program = parse_fixture("return_status");
    let func = &program.functions[0];
    if let Stmt {
        kind: StmtKind::Return(Some(val)),
        ..
    } = &func.body[0]
    {
        if let Expr {
            kind: ExprKind::Number(n),
            ..
        } = val
        {
            assert_eq!(*n, 9);
        } else {
            panic!("Expected Return(Number)");
        }
    } else {
        panic!("Expected Return(Some)");
    }
}

#[test]
fn codegen_return_status() {
    assert_codegen_matches_snapshot("return_status");
}

#[test]
fn exec_return_status() {
    assert_exec_matches_fixture("return_status");
}

// --- Backfill: Multi-Run ---

#[test]
fn codegen_multi_run() {
    assert_codegen_matches_snapshot("multi_run");
}
