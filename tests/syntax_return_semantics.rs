mod common;
use common::*;

#[test]
fn exec_return_string_literal() {
    assert_exec_matches_fixture("syntax_return_string_literal");
}

#[test]
fn exec_return_concat() {
    assert_exec_matches_fixture("syntax_return_concat");
}

#[test]
fn exec_return_bool_truthiness() {
    assert_exec_matches_fixture("syntax_return_bool_truthiness");
}

#[test]
fn exec_status_after_sh() {
    assert_exec_matches_fixture("syntax_status_after_sh");
}

#[test]
fn exec_status_after_assign() {
    assert_exec_matches_fixture("syntax_status_after_assign");
}
