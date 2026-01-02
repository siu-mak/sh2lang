use common::assert_exec_matches_fixture;
use assert_cmd::Command;
use predicates::prelude::*;

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
fn compile_bool_usage_error() {
    let mut cmd = Command::cargo_bin("sh2c").expect("Failed to find binary");
    cmd.arg("tests/fixtures/bool_usage_error.sh2")
       .assert()
       .failure()
       .stderr(predicate::str::contains("Cannot emit boolean/list value as string"))
       .stderr(predicate::str::contains("Internal error").not());
}
