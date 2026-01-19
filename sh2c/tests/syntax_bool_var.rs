//! Tests for boolean variable support

mod common;
use common::{assert_exec_matches_fixture, assert_exec_matches_fixture_target, TargetShell};
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn bool_var_basic_bash() {
    assert_exec_matches_fixture("bool_var_basic");
}

#[test]
fn bool_var_basic_posix() {
    assert_exec_matches_fixture_target("bool_var_basic", TargetShell::Posix);
}

#[test]
fn bool_var_string_error() {
    // print(bool_var) must fail with a user-facing compile error, not a panic
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    cmd.arg("tests/fixtures/bool_var_string_error.sh2")
       .assert()
       .failure()
       .stderr(predicate::str::contains("bool is not a string"))
       .stderr(predicate::str::contains("Internal error").not());
}
