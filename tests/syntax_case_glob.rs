mod common;
use common::*;

#[test]
fn codegen_case_glob_star() {
    assert_codegen_matches_snapshot("case_glob_star");
}

#[test]
fn exec_case_glob_star() {
    assert_exec_matches_fixture("case_glob_star");
}

#[test]
fn codegen_case_glob_question() {
    assert_codegen_matches_snapshot("case_glob_question");
}

#[test]
fn exec_case_glob_question() {
    assert_exec_matches_fixture("case_glob_question");
}

#[test]
fn codegen_case_glob_literal_safety() {
    assert_codegen_matches_snapshot("case_glob_literal_safety");
}

#[test]
fn exec_case_glob_literal_safety() {
    assert_exec_matches_fixture("case_glob_literal_safety");
}

#[test]
fn codegen_case_glob_dollar_and_backslash() {
    assert_codegen_matches_snapshot("case_glob_dollar_and_backslash");
}

#[test]
fn exec_case_glob_dollar_and_backslash() {
    assert_exec_matches_fixture("case_glob_dollar_and_backslash");
}