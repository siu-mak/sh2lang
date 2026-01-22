use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_version_long_flag() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2do"));
    let expected = format!("sh2do {}\n", env!("CARGO_PKG_VERSION"));

    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::eq(expected.as_str()))
        .stderr(predicate::str::is_empty());
}

#[test]
fn test_version_short_flag() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2do"));
    let expected = format!("sh2do {}\n", env!("CARGO_PKG_VERSION"));

    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicate::eq(expected.as_str()))
        .stderr(predicate::str::is_empty());
}
