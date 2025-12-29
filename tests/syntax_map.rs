mod common;
use common::*;

#[test]
fn compile_map_literal_index() {
    assert_codegen_matches_snapshot("map_literal_index");
}

#[test]
fn exec_map_literal_index() {
    assert_exec_matches_fixture("map_literal_index");
}

#[test]
fn compile_map_for_iter() {
    assert_codegen_matches_snapshot("map_for_iter");
}

#[test]
fn exec_map_for_iter() {
    assert_exec_matches_fixture("map_for_iter");
}

#[test]
fn exec_map_duplicate_keys() {
    assert_exec_matches_fixture("map_duplicate_keys");
}

#[test]
fn compile_map_posix_is_error() {
    assert_codegen_panics_target(
        "map_posix_is_error",
        TargetShell::Posix,
        "map/dict is only supported in Bash target",
    );
}

#[test]
fn compile_map_posix_index_only_is_error() {
    assert_codegen_panics_target(
        "map_posix_index_only_is_error",
        TargetShell::Posix,
        "map/dict is only supported in Bash target",
    );
}

#[test]
fn compile_map_posix_for_only_is_error() {
    assert_codegen_panics_target(
        "map_posix_for_only_is_error",
        TargetShell::Posix,
        "map/dict is only supported in Bash target",
    );
}

#[test]
fn compile_list_index_numeric() {
    assert_codegen_matches_snapshot("list_index_numeric");
}

#[test]
fn exec_list_index_numeric() {
    assert_exec_matches_fixture("list_index_numeric");
}
