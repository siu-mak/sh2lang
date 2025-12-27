mod common;
use common::*;

#[test]
fn compile_matches_basic() {
    assert_codegen_matches_snapshot("matches_basic");
}

#[test]
fn exec_matches_basic() {
    assert_exec_matches_fixture("matches_basic");
}

#[test]
fn exec_matches_basic_posix() {
    assert_exec_matches_fixture_target("matches_basic", TargetShell::Posix);
}

#[test]
fn exec_matches_empty_regex() {
    assert_exec_matches_fixture("matches_empty_regex");
    assert_exec_matches_fixture_target("matches_empty_regex", TargetShell::Posix);
}
