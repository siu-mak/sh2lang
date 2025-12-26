mod common;
use common::*;
use sh2c::codegen::TargetShell;

#[test]
fn codegen_quote_literal_spaces_bash() {
    assert_codegen_matches_snapshot_target("quote_literal_spaces", TargetShell::Bash);
}
#[test]
fn exec_quote_literal_spaces_bash() {
    assert_exec_matches_fixture_target("quote_literal_spaces", TargetShell::Bash);
}

#[test]
fn codegen_quote_literal_dollar_backslash_star_bash() {
    assert_codegen_matches_snapshot_target("quote_literal_dollar_backslash_star", TargetShell::Bash);
}
#[test]
fn exec_quote_literal_dollar_backslash_star_bash() {
    assert_exec_matches_fixture_target("quote_literal_dollar_backslash_star", TargetShell::Bash);
}

#[test]
fn codegen_quote_literal_newline_bash() {
    assert_codegen_matches_snapshot_target("quote_literal_newline", TargetShell::Bash);
}
#[test]
fn exec_quote_literal_newline_bash() {
    assert_exec_matches_fixture_target("quote_literal_newline", TargetShell::Bash);
}

#[test]
fn codegen_quote_concat_var_bash() {
    assert_codegen_matches_snapshot_target("quote_concat_var", TargetShell::Bash);
}
#[test]
fn exec_quote_concat_var_bash() {
    assert_exec_matches_fixture_target("quote_concat_var", TargetShell::Bash);
}

#[test]
fn codegen_quote_args_splice_bash() {
    assert_codegen_matches_snapshot_target("quote_args_splice", TargetShell::Bash);
}
#[test]
fn exec_quote_args_splice_bash() {
    assert_exec_matches_fixture_target("quote_args_splice", TargetShell::Bash);
}

#[test]
fn codegen_quote_args_disallowed_concat_fail() {
    assert_codegen_panics_target(
        "quote_args_disallowed_concat", 
        TargetShell::Bash, 
        "Cannot emit boolean/list"
    );
}

#[test]
fn codegen_quote_args_splice_posix() {
    assert_codegen_matches_snapshot_target("quote_args_splice", TargetShell::Posix);
}
#[test]
fn exec_quote_args_splice_posix() {
    assert_exec_matches_fixture_target("quote_args_splice", TargetShell::Posix);
}
