mod common;
use common::*;

#[test]
fn compile_json_kv_basic() {
    assert_codegen_matches_snapshot("json_kv_basic");
}

#[test]
fn compile_json_kv_duplicates() {
    assert_codegen_matches_snapshot("json_kv_duplicates");
}

#[test]
fn compile_json_kv_empty() {
    assert_codegen_matches_snapshot("json_kv_empty");
}

#[test]
fn exec_json_kv_basic() {
    assert_exec_matches_fixture("json_kv_basic");
}

#[test]
fn exec_json_kv_basic_posix() {
    assert_exec_matches_fixture_target("json_kv_basic", TargetShell::Posix);
}

#[test]
fn exec_json_kv_duplicates() {
    assert_exec_matches_fixture("json_kv_duplicates");
}

#[test]
fn exec_json_kv_duplicates_posix() {
    assert_exec_matches_fixture_target("json_kv_duplicates", TargetShell::Posix);
}

#[test]
fn exec_json_kv_empty() {
    assert_exec_matches_fixture("json_kv_empty");
}

#[test]
fn exec_json_kv_empty_posix() {
    assert_exec_matches_fixture_target("json_kv_empty", TargetShell::Posix);
}
