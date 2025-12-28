mod common;
use common::*;

#[test]
fn codegen_status_after_run_no_wrapper() {
    assert_codegen_matches_snapshot("status_after_run_no_wrapper");
}

#[test]
fn exec_status_after_run_no_wrapper() {
    assert_exec_matches_fixture("status_after_run_no_wrapper");
}