mod common;
use sh2c::ast::Expr;
use sh2c::ast::Stmt;
use common::{parse_fixture, assert_codegen_matches_snapshot, assert_exec_matches_fixture};

#[test]
fn parse_print_args() {
    let program = parse_fixture("print_args");
    let func = &program.functions[0];
    if let Stmt::Print(expr) = &func.body[0] {
        assert!(matches!(expr, Expr::Args));
    } else {
        panic!("Expected Print(Args)");
    }
}

#[test]
fn codegen_print_err() { assert_codegen_matches_snapshot("print_err"); }

#[test]
fn codegen_run_args() { assert_codegen_matches_snapshot("run_args"); }

#[test]
fn codegen_print_args() { assert_codegen_matches_snapshot("print_args"); }
#[test]
fn codegen_print_err_args() { assert_codegen_matches_snapshot("print_err_args"); }

#[test]
fn codegen_with_redirect_stdout() { assert_codegen_matches_snapshot("with_redirect_stdout"); }
#[test]
fn codegen_with_redirect_stdout_append() { assert_codegen_matches_snapshot("with_redirect_stdout_append"); }
#[test]
fn codegen_with_redirect_stderr_to_stdout() { assert_codegen_matches_snapshot("with_redirect_stderr_to_stdout"); }
#[test]
fn codegen_with_redirect_stdout_to_stderr() { assert_codegen_matches_snapshot("with_redirect_stdout_to_stderr"); }
#[test]
fn codegen_with_redirect_stdin_file() { assert_codegen_matches_snapshot("with_redirect_stdin_file"); }
#[test]
fn codegen_with_redirect_combo() { assert_codegen_matches_snapshot("with_redirect_combo"); }

#[test]
fn exec_print_err() { assert_exec_matches_fixture("print_err_exec"); }
#[test]
fn exec_print_args() { assert_exec_matches_fixture("print_args"); }
#[test]
fn exec_run_args() { assert_exec_matches_fixture("run_args"); }

#[test]
fn parse_print_err_statement_inline() {
    let src = r#"
        func main() {
            print_err("fail")
        }
    "#;
    use sh2c::{lexer, parser};
    let tokens = lexer::lex(src);
    let program = parser::parse(&tokens);
    match &program.functions[0].body[0] {
        sh2c::ast::Stmt::PrintErr(sh2c::ast::Expr::Literal(s)) => {
            assert_eq!(s, "fail");
        }
        _ => panic!("Expected PrintErr"),
    }
}

// --- Backfill: Redirects ---

#[test]
fn codegen_with_redirect_stderr_noop() { assert_codegen_matches_snapshot("with_redirect_stderr_noop"); }
#[test]
fn codegen_with_redirect_stdin_and_stdout_file() { assert_codegen_matches_snapshot("with_redirect_stdin_and_stdout_file"); }
#[test]
fn codegen_with_redirect_stdout_to_stderr_and_stderr_file() { assert_codegen_matches_snapshot("with_redirect_stdout_to_stderr_and_stderr_file"); }

#[test]
#[should_panic(expected = "Cyclic redirection")]
fn codegen_with_redirect_cyclic() {
    let src = include_str!("fixtures/with_redirect_cyclic.sh2");
    use sh2c::{lexer, parser, lower, codegen};
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    codegen::emit(&ir);
}


