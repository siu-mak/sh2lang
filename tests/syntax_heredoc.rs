mod common;
use common::*;

#[test]
fn codegen_heredoc_cat_basic() {
    assert_codegen_matches_snapshot("heredoc_cat_basic");
}

#[test]
fn exec_heredoc_cat_basic() {
    assert_exec_matches_fixture("heredoc_cat_basic");
}

#[test]
fn codegen_heredoc_literal_safety() {
    assert_codegen_matches_snapshot("heredoc_literal_safety");
}

#[test]
fn exec_heredoc_literal_safety() {
    assert_exec_matches_fixture("heredoc_literal_safety");
}
