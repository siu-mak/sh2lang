mod common;
use common::{assert_codegen_matches_snapshot, assert_exec_matches_fixture};

#[test]
fn codegen_quote_space_arg() { assert_codegen_matches_snapshot("quote_space_arg"); }
#[test]
fn exec_quote_space_arg() { assert_exec_matches_fixture("quote_space_arg"); }

#[test]
fn codegen_quote_dollar_literal() { assert_codegen_matches_snapshot("quote_dollar_literal"); }
#[test]
fn exec_quote_dollar_literal() { assert_exec_matches_fixture("quote_dollar_literal"); }

#[test]
fn codegen_quote_glob_star() { assert_codegen_matches_snapshot("quote_glob_star"); }
#[test]
fn exec_quote_glob_star() { assert_exec_matches_fixture("quote_glob_star"); }

#[test]
fn codegen_quote_backslash() { assert_codegen_matches_snapshot("quote_backslash"); }
#[test]
fn exec_quote_backslash() { assert_exec_matches_fixture("quote_backslash"); }

#[test]
fn codegen_quote_newline_literal() { assert_codegen_matches_snapshot("quote_newline_literal"); }
#[test]
fn exec_quote_newline_literal() { assert_exec_matches_fixture("quote_newline_literal"); }

#[test]
fn codegen_quote_concat_no_split() { assert_codegen_matches_snapshot("quote_concat_no_split"); }
#[test]
fn exec_quote_concat_no_split() { assert_exec_matches_fixture("quote_concat_no_split"); }