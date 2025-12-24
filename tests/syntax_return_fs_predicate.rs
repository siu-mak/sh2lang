mod common;
use common::*;

#[test]
fn exec_return_fs_predicate() {
    assert_exec_matches_fixture("return_fs_predicate");
}
