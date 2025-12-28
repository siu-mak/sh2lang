mod common;
use common::{assert_codegen_matches_snapshot, assert_exec_matches_fixture};

#[test]
fn codegen_fs_predicates_full() {
    assert_codegen_matches_snapshot("fs_predicates_full");
}

#[test]
fn exec_fs_predicates_full() {
    assert_exec_matches_fixture("fs_predicates_full");
}