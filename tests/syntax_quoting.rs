mod common;
use common::*;
use sh2c::codegen::TargetShell;

#[test]
fn codegen_quoting_glob_run() { assert_codegen_matches_snapshot("quoting_glob_run"); }
#[test]
fn exec_quoting_glob_run() { assert_exec_matches_fixture("quoting_glob_run"); }

#[test]
fn codegen_quoting_space_argv_count() { assert_codegen_matches_snapshot("quoting_space_argv_count"); }
#[test]
fn exec_quoting_space_argv_count() { assert_exec_matches_fixture("quoting_space_argv_count"); }

#[test]
fn codegen_quoting_dollar_literal() { assert_codegen_matches_snapshot("quoting_dollar_literal"); }
#[test]
fn exec_quoting_dollar_literal() { assert_exec_matches_fixture("quoting_dollar_literal"); }

#[test]
fn codegen_quoting_backslash_literal() { assert_codegen_matches_snapshot("quoting_backslash_literal"); }
#[test]
fn exec_quoting_backslash_literal() { assert_exec_matches_fixture("quoting_backslash_literal"); }

#[test]
fn codegen_quoting_single_quote() { assert_codegen_matches_snapshot("quoting_single_quote"); }
#[test]
fn exec_quoting_single_quote() { assert_exec_matches_fixture("quoting_single_quote"); }

#[test]
fn codegen_quoting_newline_literal() { assert_codegen_matches_snapshot("quoting_newline_literal"); }
#[test]
fn exec_quoting_newline_literal() { assert_exec_matches_fixture("quoting_newline_literal"); }
