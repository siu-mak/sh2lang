mod common;
use common::*;

#[test]
fn exec_json_kv_no_double_call() {
    // This fixture prints "DOUBLE", expecting it ONCE.
    // If the bug exists (double call emission), it might print twice.
    assert_exec_matches_fixture("json_kv_no_double_call");
}

#[test]
fn exec_json_kv_no_double_call_posix() {
    assert_exec_matches_fixture_target("json_kv_no_double_call", TargetShell::Posix);
}