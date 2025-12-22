use std::fs;
use sh2c::{lexer, parser, lower, codegen};

fn compile(src: &str) -> String {
    let tokens = lexer::lex(src);
    let ast = parser::parse(&tokens);
    let ir = lower::lower(ast);
    codegen::emit(&ir)
}

#[test]
fn hello_codegen_matches_snapshot() {
    let src = fs::read_to_string("tests/fixtures/hello.sh2").unwrap();
    let expected = fs::read_to_string("tests/fixtures/hello.sh.expected").unwrap();

    let output = compile(&src);

    assert_eq!(output.trim(), expected.trim());
}
#[test]
fn multiple_run_codegen() {
    let src = std::fs::read_to_string("tests/fixtures/multi_run.sh2").unwrap();
    let expected = std::fs::read_to_string("tests/fixtures/multi_run.sh.expected").unwrap();

    let tokens = sh2c::lexer::lex(&src);
    let ast = sh2c::parser::parse(&tokens);
    let ir = sh2c::lower::lower(ast);
    let output = sh2c::codegen::emit(&ir);

    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn if_basic_codegen_matches_snapshot() {
    let src = fs::read_to_string("tests/fixtures/if_basic.sh2").unwrap();
    let expected = fs::read_to_string("tests/fixtures/if_basic.sh.expected").unwrap();

    let output = compile(&src);

    assert_eq!(output.trim(), expected.trim());
}
#[test]
fn print_err_codegen() {
    let src = r#"
        func main() {
            print_err("oops")
        }
    "#;

    let expected = r#"
main() {
  echo "oops" >&2
}

main "$@"
"#;

    let tokens = sh2c::lexer::lex(src);
    let ast = sh2c::parser::parse(&tokens);
    let ir = sh2c::lower::lower(ast);
    let out = sh2c::codegen::emit(&ir);

    assert_eq!(out.trim(), expected.trim());
}

#[test]
fn let_codegen_matches_snapshot() {
    let src = fs::read_to_string("tests/fixtures/let.sh2").unwrap();
    let expected = fs::read_to_string("tests/fixtures/let.sh.expected").unwrap();

    let output = compile(&src);

    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn cmd_sub_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/cmd_sub.sh2";
    let expected_path = "tests/fixtures/cmd_sub.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn pipe_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/pipe.sh2";
    let expected_path = "tests/fixtures/pipe.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn case_basic_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/case_basic.sh2";
    let expected_path = "tests/fixtures/case_basic.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn while_basic_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/while_basic.sh2";
    let expected_path = "tests/fixtures/while_basic.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn for_basic_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/for_basic.sh2";
    let expected_path = "tests/fixtures/for_basic.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn return_basic_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/return_basic.sh2";
    let expected_path = "tests/fixtures/return_basic.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn exit_basic_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/exit_basic.sh2";
    let expected_path = "tests/fixtures/exit_basic.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn if_elif_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/if_elif.sh2";
    let expected_path = "tests/fixtures/if_elif.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn if_boolean_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/if_bool.sh2";
    let expected_path = "tests/fixtures/if_bool.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn for_list_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/for_list.sh2";
    let expected_path = "tests/fixtures/for_list.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn for_args_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/for_args.sh2";
    let expected_path = "tests/fixtures/for_args.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn with_env_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/with_env.sh2";
    let expected_path = "tests/fixtures/with_env.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn with_cwd_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/with_cwd.sh2";
    let expected_path = "tests/fixtures/with_cwd.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn sh_raw_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/sh_raw.sh2";
    let expected_path = "tests/fixtures/sh_raw.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn sh_block_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/sh_block.sh2";
    let expected_path = "tests/fixtures/sh_block.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn if_else_if_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/if_else_if.sh2";
    let expected_path = "tests/fixtures/if_else_if.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn if_paren_cond_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/if_paren_cond.sh2";
    let expected_path = "tests/fixtures/if_paren_cond.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn comments_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/comments.sh2";
    let expected_path = "tests/fixtures/comments.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn call_func_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/call_func.sh2";
    let expected_path = "tests/fixtures/call_func.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn run_args_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/run_args.sh2";
    let expected_path = "tests/fixtures/run_args.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn call_args_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/call_args.sh2";
    let expected_path = "tests/fixtures/call_args.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn cmd_sub_args_codegen_matches_snapshot() {
    let sh2_path = "tests/fixtures/cmd_sub_args.sh2";
    let expected_path = "tests/fixtures/cmd_sub_args.sh.expected";
    assert_codegen_matches_snapshot(sh2_path, expected_path);
}

#[test]
fn cmd_sub_call_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/cmd_sub_call.sh2",
        "tests/fixtures/cmd_sub_call.sh.expected",
    );
}

#[test]
fn cmd_sub_call_args_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/cmd_sub_call_args.sh2",
        "tests/fixtures/cmd_sub_call_args.sh.expected",
    );
}

#[test]
fn capture_basic_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/capture_basic.sh2",
        "tests/fixtures/capture_basic.sh.expected",
    );
}

#[test]
fn capture_args_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/capture_args.sh2",
        "tests/fixtures/capture_args.sh.expected",
    );
}

#[test]
fn subshell_basic_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/subshell_basic.sh2",
        "tests/fixtures/subshell_basic.sh.expected",
    );
}

#[test]
fn group_basic_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/group_basic.sh2",
        "tests/fixtures/group_basic.sh.expected",
    );
}

#[test]
fn with_redirect_stdout_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/with_redirect_stdout.sh2",
        "tests/fixtures/with_redirect_stdout.sh.expected",
    );
}

#[test]
fn with_redirect_stdout_append_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/with_redirect_stdout_append.sh2",
        "tests/fixtures/with_redirect_stdout_append.sh.expected",
    );
}

#[test]
fn with_redirect_stderr_to_stdout_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/with_redirect_stderr_to_stdout.sh2",
        "tests/fixtures/with_redirect_stderr_to_stdout.sh.expected",
    );
}

#[test]
fn with_redirect_combo_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/with_redirect_combo.sh2",
        "tests/fixtures/with_redirect_combo.sh.expected",
    );
}

#[test]
fn spawn_run_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/spawn_run.sh2",
        "tests/fixtures/spawn_run.sh.expected",
    );
}

#[test]
fn spawn_group_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/spawn_group.sh2",
        "tests/fixtures/spawn_group.sh.expected",
    );
}

#[test]
fn spawn_sh_block_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/spawn_sh_block.sh2",
        "tests/fixtures/spawn_sh_block.sh.expected",
    );
}

#[test]
fn try_catch_basic_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/try_catch_basic.sh2",
        "tests/fixtures/try_catch_basic.sh.expected",
    );
}

#[test]
fn try_catch_success_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/try_catch_success.sh2",
        "tests/fixtures/try_catch_success.sh.expected",
    );
}

#[test]
fn wait_all_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/wait_all.sh2",
        "tests/fixtures/wait_all.sh.expected",
    );
}

#[test]
fn wait_no_arg_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/wait_no_arg.sh2",
        "tests/fixtures/wait_no_arg.sh.expected",
    );
}

#[test]
fn wait_complex_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/wait_complex.sh2",
        "tests/fixtures/wait_complex.sh.expected",
    );
}

#[test]
fn wait_list_literal_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/wait_list_literal.sh2",
        "tests/fixtures/wait_list_literal.sh.expected",
    );
}

#[test]
fn wait_list_var_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/wait_list_var.sh2",
        "tests/fixtures/wait_list_var.sh.expected",
    );
}

#[test]
fn wait_list_complex_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/wait_list_complex.sh2",
        "tests/fixtures/wait_list_complex.sh.expected",
    );
}

#[test]
fn wait_pid_literal_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/wait_pid_literal.sh2",
        "tests/fixtures/wait_pid_literal.sh.expected",
    );
}

#[test]
fn wait_pid_var_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/wait_pid_var.sh2",
        "tests/fixtures/wait_pid_var.sh.expected",
    );
}

#[test]
fn with_redirect_stdout_to_stderr_and_stderr_file_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/with_redirect_stdout_to_stderr_and_stderr_file.sh2",
        "tests/fixtures/with_redirect_stdout_to_stderr_and_stderr_file.sh.expected",
    );
}

#[test]
fn with_redirect_stderr_noop_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/with_redirect_stderr_noop.sh2",
        "tests/fixtures/with_redirect_stderr_noop.sh.expected",
    );
}

#[test]
fn with_redirect_stdout_to_stderr_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/with_redirect_stdout_to_stderr.sh2",
        "tests/fixtures/with_redirect_stdout_to_stderr.sh.expected",
    );
}

#[test]
fn with_redirect_stdin_file_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/with_redirect_stdin_file.sh2",
        "tests/fixtures/with_redirect_stdin_file.sh.expected",
    );
}

#[test]
fn with_redirect_stdin_and_stdout_file_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/with_redirect_stdin_and_stdout_file.sh2",
        "tests/fixtures/with_redirect_stdin_and_stdout_file.sh.expected",
    );
}

#[test]
fn cmd_sub_pipe_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/cmd_sub_pipe.sh2",
        "tests/fixtures/cmd_sub_pipe.sh.expected",
    );
}

#[test]
fn capture_pipe_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/capture_pipe.sh2",
        "tests/fixtures/capture_pipe.sh.expected",
    );
}

#[test]
#[should_panic(expected = "Cyclic redirection")]
fn with_redirect_cyclic_panics() {
    assert_codegen_matches_snapshot(
         "tests/fixtures/with_redirect_cyclic.sh2",
         "tests/fixtures/nothing.sh.expected",
    );
}

#[test]
fn and_or_chain_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/and_or_chain.sh2",
        "tests/fixtures/and_or_chain.sh.expected",
    );
}

#[test]
fn export_unset_codegen_matches_snapshot() {
    assert_codegen_matches_snapshot(
        "tests/fixtures/export_unset.sh2",
        "tests/fixtures/export_unset.sh.expected",
    );
}

fn assert_codegen_matches_snapshot(sh2_path: &str, expected_path: &str) {
    let src = fs::read_to_string(sh2_path).unwrap();
    let expected = fs::read_to_string(expected_path).unwrap();
    let output = compile(&src);
    assert_eq!(output.trim(), expected.trim());
}
