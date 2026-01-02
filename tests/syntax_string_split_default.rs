mod common;
use common::*;


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
    assert_codegen_matches_snapshot("string_default_alias");
    assert_codegen_matches_snapshot("call_unknown_arity_ok");
}
