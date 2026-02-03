use std::process::Command;
use assert_cmd::prelude::*;

#[test]
fn test_contains_list_true() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    let assert = cmd
        .arg("--target")
        .arg("bash")
        .arg("tests/fixtures/contains_list_true.sh2")
        .assert();
    let output = assert.success().get_output().stdout.clone();
    let script = String::from_utf8(output).unwrap();
    Command::new("bash")
        .arg("-c")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicates::str::contains("true"));
}

#[test]
fn test_contains_list_false() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    let assert = cmd
        .arg("--target")
        .arg("bash")
        .arg("tests/fixtures/contains_list_false.sh2")
        .assert();
    let output = assert.success().get_output().stdout.clone();
    let script = String::from_utf8(output).unwrap();
    Command::new("bash")
        .arg("-c")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicates::str::contains("false"));
}

#[test]
fn test_contains_lines_true() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    let assert = cmd
        .arg("--target")
        .arg("bash")
        .arg("tests/fixtures/contains_lines_true.sh2")
        .assert();
    let output = assert.success().get_output().stdout.clone();
    let script = String::from_utf8(output).unwrap();
    Command::new("bash")
        .arg("-c")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicates::str::contains("true"));
}

#[test]
fn test_contains_posix_fail() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    cmd.arg("--target")
        .arg("posix")
        .arg("tests/fixtures/contains_list_split_posix_fail.sh2")
        .assert()
        .failure()
        .stderr(predicates::str::contains("contains(list, item) is Bash-only"));
}

#[test]
fn test_contains_substring_posix() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    let assert = cmd
        .arg("--target")
        .arg("posix")
        .arg("tests/fixtures/contains_substring_posix.sh2")
        .assert();
    let output = assert.success().get_output().stdout.clone();
    let script = String::from_utf8(output).unwrap();
    Command::new("/bin/sh")
        .arg("-c")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicates::str::contains("ok"));
}
