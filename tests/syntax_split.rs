use common::assert_exec_matches_fixture;
use assert_cmd::Command;
use predicates::prelude::*;

mod common;

#[test]
fn compile_split_basic() {
    assert_exec_matches_fixture("split_basic");
}

#[test]
fn compile_split_empty_fields() {
    assert_exec_matches_fixture("split_empty_fields");
}

#[test]
fn compile_split_not_found() {
    assert_exec_matches_fixture("split_not_found");
}

#[test]
fn compile_split_multichar() {
    assert_exec_matches_fixture("split_multichar");
}

#[test]
fn compile_split_usage_error() {
    let mut cmd = Command::cargo_bin("sh2c").expect("Failed to find binary");
    cmd.arg("tests/fixtures/split_usage_error.sh2")
       .assert()
       .failure()
       .stderr(predicate::str::contains("Cannot emit boolean/list value as string"))
       .stderr(predicate::str::contains("Internal error").not());
}


#[test]
fn compile_split_basic_posix() {
    let mut cmd = Command::cargo_bin("sh2c").expect("Failed to find binary");
    cmd.arg("--target").arg("posix");
    cmd.arg("tests/fixtures/split_basic.sh2");
    
    // Assert compilation succeeds and emits valid POSIX loop structure
    cmd.assert()
       .success()
       .stdout(predicate::str::contains("__sh2_split() {"))
       .stdout(predicate::str::contains("awk -v s=\"$1\"")) // POSIX simple awk
       .stdout(predicate::str::contains("xs=\"$( __sh2_split"))
       .stdout(predicate::str::contains("for x in $xs; do")); // Unquoted iteration
}
