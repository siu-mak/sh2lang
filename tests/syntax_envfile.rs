mod common;
use common::*;

#[test]
fn compile_envfile_load_basic() {
    assert_codegen_matches_snapshot("envfile_load_basic");
}

#[test]
fn compile_envfile_roundtrip() {
    assert_codegen_matches_snapshot("envfile_roundtrip");
}

#[test]
fn exec_envfile_load_basic() {
    assert_exec_matches_fixture("envfile_load_basic");
}

#[test]
fn exec_envfile_load_basic_posix() {
    assert_exec_matches_fixture_target("envfile_load_basic", TargetShell::Posix);
}

#[test]
fn exec_envfile_roundtrip() {
    assert_exec_matches_fixture("envfile_roundtrip");
}

#[test]
fn exec_envfile_roundtrip_posix() {
    assert_exec_matches_fixture_target("envfile_roundtrip", TargetShell::Posix);
}

#[test]
fn compile_envfile_load_as_statement() {
    assert_codegen_panics("envfile_load_as_statement", "load_envfile() returns a value; use it in an expression");
}

#[test]
fn exec_envfile_missing_file() {
    assert_exec_matches_fixture("envfile_missing_file");
}

#[test]
fn exec_envfile_missing_file_posix() {
    assert_exec_matches_fixture_target("envfile_missing_file", TargetShell::Posix);
}
