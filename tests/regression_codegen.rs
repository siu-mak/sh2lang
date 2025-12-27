mod common;
use common::*;

#[test]
fn compile_call_user_func_expr() {
    assert_codegen_matches_snapshot("call_user_func_expr");
}

#[test]
fn exec_call_user_func_expr() {
    assert_exec_matches_fixture("call_user_func_expr");
}
