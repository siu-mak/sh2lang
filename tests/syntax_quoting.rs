mod common;
use common::*;

#[test]
fn exec_literal_arg_spaces() {
    assert_exec_matches_fixture("literal_arg_spaces");
}

#[test]
fn exec_literal_arg_dollar() {
    assert_exec_matches_fixture("literal_arg_dollar");
}

#[test]
fn exec_literal_arg_glob_star() {
    assert_exec_matches_fixture("literal_arg_glob_star");
}

#[test]
fn exec_literal_arg_backslash() {
    assert_exec_matches_fixture("literal_arg_backslash");
}

#[test]
fn exec_literal_arg_newline() {
    assert_exec_matches_fixture("literal_arg_newline");
}

#[test]
fn exec_concat_var_and_literal_safe() {
    assert_exec_matches_fixture("concat_var_and_literal_safe");
}

#[test]
fn exec_capture_is_single_arg() {
    assert_exec_matches_fixture("capture_is_single_arg");
}

#[test]
fn codegen_literal_arg_spaces() {
    assert_codegen_matches_snapshot("literal_arg_spaces");
}

#[test]
fn codegen_literal_arg_dollar() {
    assert_codegen_matches_snapshot("literal_arg_dollar");
}

#[test]
fn codegen_literal_arg_glob_star() {
    assert_codegen_matches_snapshot("literal_arg_glob_star");
}

#[test]
fn codegen_literal_arg_backslash() {
    assert_codegen_matches_snapshot("literal_arg_backslash");
}

#[test]
fn codegen_literal_arg_newline() {
    assert_codegen_matches_snapshot("literal_arg_newline");
}

#[test]
fn codegen_concat_var_and_literal_safe() {
    assert_codegen_matches_snapshot("concat_var_and_literal_safe");
}

#[test]
fn codegen_capture_is_single_arg() {
    assert_codegen_matches_snapshot("capture_is_single_arg");
}
