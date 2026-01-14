mod common;
use common::{assert_codegen_matches_snapshot, assert_exec_matches_fixture, parse_fixture};
use sh2c::ast::{Expr, ExprKind};
use sh2c::ast::{Stmt, StmtKind};

#[test]
fn parse_print_args() {
    let program = parse_fixture("print_args");
    let func = &program.functions[0];
    if let Stmt {
        node: StmtKind::Print(expr),
        ..
    } = &func.body[0]
    {
        assert!(matches!(
            expr,
            Expr {
                node: ExprKind::Args,
                ..
            }
        ));
    } else {
        panic!("Expected Print(Args)");
    }
}

#[test]
fn codegen_print_err() {
    assert_codegen_matches_snapshot("print_err");
}

#[test]
fn codegen_run_args() {
    assert_codegen_matches_snapshot("run_args");
}

#[test]
fn codegen_print_args() {
    assert_codegen_matches_snapshot("print_args");
}
#[test]
fn codegen_print_err_args() {
    assert_codegen_matches_snapshot("print_err_args");
}

#[test]
fn codegen_with_redirect_stdout() {
    assert_codegen_matches_snapshot("with_redirect_stdout");
}
#[test]
fn codegen_with_redirect_stdout_append() {
    assert_codegen_matches_snapshot("with_redirect_stdout_append");
}
#[test]
fn codegen_with_redirect_stderr_to_stdout() {
    assert_codegen_matches_snapshot("with_redirect_stderr_to_stdout");
}
#[test]
fn codegen_with_redirect_stdout_to_stderr() {
    assert_codegen_matches_snapshot("with_redirect_stdout_to_stderr");
}
#[test]
fn codegen_with_redirect_stdin_file() {
    assert_codegen_matches_snapshot("with_redirect_stdin_file");
}
#[test]
fn codegen_with_redirect_combo() {
    assert_codegen_matches_snapshot("with_redirect_combo");
}

#[test]
fn exec_print_err() {
    assert_exec_matches_fixture("print_err_exec");
}
#[test]
fn exec_print_args() {
    assert_exec_matches_fixture("print_args");
}
#[test]
fn exec_run_args() {
    assert_exec_matches_fixture("run_args");
}

#[test]
fn parse_print_err_statement_inline() {
    let src = r#"
        func main() {
            print_err("fail")
        }
    "#;
    use sh2c::parser;
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = sh2c::lexer::lex(&sm, "test").unwrap();
    let program = parser::parse(&tokens, &sm, "test").unwrap();
    match &program.functions[0].body[0] {
        sh2c::ast::Stmt {
            node:
                StmtKind::PrintErr(sh2c::ast::Expr {
                    node: ExprKind::Literal(s),
                    ..
                }),
            ..
        } => {
            assert_eq!(s, "fail");
        }
        _ => panic!("Expected PrintErr"),
    }
}

// --- Backfill: Redirects ---

#[test]
fn codegen_with_redirect_stderr_noop() {
    assert_codegen_matches_snapshot("with_redirect_stderr_noop");
}
#[test]
fn codegen_with_redirect_stdin_and_stdout_file() {
    assert_codegen_matches_snapshot("with_redirect_stdin_and_stdout_file");
}
#[test]
fn codegen_with_redirect_stdout_to_stderr_and_stderr_file() {
    assert_codegen_matches_snapshot("with_redirect_stdout_to_stderr_and_stderr_file");
}

#[test]
#[should_panic(expected = "Cyclic redirection")]
fn codegen_with_redirect_cyclic() {
    let src = include_str!("fixtures/with_redirect_cyclic.sh2");
    use sh2c::{codegen, lower, parser};
    let sm = sh2c::span::SourceMap::new(src.to_string());
    let tokens = sh2c::lexer::lex(&sm, "test").unwrap();
    let mut ast = parser::parse(&tokens, &sm, "test").unwrap();
    ast.source_maps.insert("test".to_string(), sm);
    let ir = lower::lower(ast).unwrap();
    codegen::emit_with_options(&ir, codegen::CodegenOptions { target: codegen::TargetShell::Bash, include_diagnostics: true }).unwrap();
}
