use common::assert_exec_matches_fixture;
use assert_cmd::Command;

mod common;

#[test]
fn compile_bool_compare_false() {
    assert_exec_matches_fixture("bool_compare_false");
}

#[test]
fn compile_bool_compare_variations() {
    assert_exec_matches_fixture("bool_compare_variations");
}

#[test]
fn compile_bool_var_from_predicate() {
    // This now compiles successfully - boolean variables are supported
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    cmd.arg("tests/fixtures/bool_usage_error.sh2")
       .assert()
       .success();
}

#[test]
fn compile_bool_var_from_literal() {
    // This now compiles successfully - boolean variables are supported
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    cmd.arg("tests/fixtures/bool_literal_usage_error.sh2")
       .assert()
       .success();
}
