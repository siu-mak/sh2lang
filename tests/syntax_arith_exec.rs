mod common;
use common::{assert_codegen_matches_snapshot, assert_exec_matches_fixture};

#[test]
fn codegen_arith_len_plus_one() {
    assert_codegen_matches_snapshot("arith_len_plus_one");
}
#[test]
fn exec_arith_len_plus_one() {
    assert_exec_matches_fixture("arith_len_plus_one");
}

#[test]
fn codegen_arith_count_args_plus_one() {
    assert_codegen_matches_snapshot("arith_count_args_plus_one");
}
#[test]
fn exec_arith_count_args_plus_one() {
    assert_exec_matches_fixture("arith_count_args_plus_one");
}

#[test]
fn codegen_arith_cmdsub_plus_one() {
    assert_codegen_matches_snapshot("arith_cmdsub_plus_one");
}
#[test]
fn exec_arith_cmdsub_plus_one() {
    assert_exec_matches_fixture("arith_cmdsub_plus_one");
}
