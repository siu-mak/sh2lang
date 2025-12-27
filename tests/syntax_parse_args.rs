mod common;
use common::*;

#[test]
fn compile_parse_args_basic() {
    assert_codegen_matches_snapshot("parse_args_basic");
}

#[test]
fn exec_parse_args_basic() {
    assert_exec_matches_fixture("parse_args_basic");
}

#[test]
fn exec_parse_args_basic_posix() {
    assert_exec_matches_fixture_target("parse_args_basic", TargetShell::Posix);
}

#[test]
fn exec_parse_args_double_dash() {
    assert_exec_matches_fixture("parse_args_double_dash");
    assert_exec_matches_fixture_target("parse_args_double_dash", TargetShell::Posix);
}

#[test]
fn exec_parse_args_repeated_and_missing() {
    assert_exec_matches_fixture("parse_args_repeated_and_missing");
    assert_exec_matches_fixture_target("parse_args_repeated_and_missing", TargetShell::Posix);
}
