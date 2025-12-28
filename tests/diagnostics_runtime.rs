mod common;
use common::*;

#[test]
fn exec_runtime_error_loc() {
    assert_exec_matches_fixture("runtime_error_loc");
}
