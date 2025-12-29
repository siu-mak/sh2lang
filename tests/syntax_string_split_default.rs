mod common;
use common::*;

#[test]
fn exec_string_split_basic() {
    assert_exec_matches_fixture_target("string_split_basic", TargetShell::Bash);
    assert_exec_matches_fixture_target("string_split_basic", TargetShell::Posix);
}

#[test]
fn exec_string_split_trailing_and_empty() {
    assert_exec_matches_fixture_target("string_split_trailing_and_empty", TargetShell::Bash);
    assert_exec_matches_fixture_target("string_split_trailing_and_empty", TargetShell::Posix);
}

#[test]
fn exec_string_split_sep_not_found() {
    assert_exec_matches_fixture_target("string_split_sep_not_found", TargetShell::Bash);
    assert_exec_matches_fixture_target("string_split_sep_not_found", TargetShell::Posix);
}

#[test]
fn exec_string_split_empty_sep_noop() {
    assert_exec_matches_fixture_target("string_split_empty_sep_noop", TargetShell::Bash);
    assert_exec_matches_fixture_target("string_split_empty_sep_noop", TargetShell::Posix);
}

#[test]
fn exec_string_default_alias() {
    assert_exec_matches_fixture_target("string_default_alias", TargetShell::Bash);
    assert_exec_matches_fixture_target("string_default_alias", TargetShell::Posix);
}

#[test]
fn exec_call_unknown_arity_ok() {
    assert_exec_matches_fixture_target("call_unknown_arity_ok", TargetShell::Bash);
    assert_exec_matches_fixture_target("call_unknown_arity_ok", TargetShell::Posix);
}

#[test]
fn codegen_string_split_default() {
    assert_codegen_matches_snapshot("string_split_basic");
    assert_codegen_matches_snapshot("string_split_trailing_and_empty");
    assert_codegen_matches_snapshot("string_split_sep_not_found");
    assert_codegen_matches_snapshot("string_split_empty_sep_noop");
    assert_codegen_matches_snapshot("string_default_alias");
    assert_codegen_matches_snapshot("call_unknown_arity_ok");
}
