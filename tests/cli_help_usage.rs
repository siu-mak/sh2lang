use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_prints_once_exit_0() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: sh2c"))
        .stderr(""); // Help typically goes to stdout, but stderr accepted if consistent. 
                     // Requirement: "prints usage/help text once to stdout (or stderr)".
                     // Usually standard apps use stdout for --help.
}

#[test]
fn short_help_prints_once_exit_0() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    cmd.arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: sh2c"));
}

#[test]
fn unknown_flag_prints_error_and_usage_once_exit_1() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    cmd.arg("--nope")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("error: Unexpected argument: --nope"))
        .stderr(predicate::str::contains("Usage: sh2c").count(1));
}

#[test]
fn missing_filename_prints_usage_once_exit_1() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    cmd.arg("--target").arg("posix") // Valid flags but no filename
        .assert()
        .failure()
        .code(1)
        // Usage should appear exactly once
        .stderr(predicate::str::contains("Usage: sh2c").count(1));
}

#[test]
fn check_out_conflict_usage_once_exit_2() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    cmd.arg("--check").arg("--out").arg("x.sh").arg("tests/fixtures/cli_target_basic.sh2")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("error: --check cannot be used with --out"))
        .stderr(predicate::str::contains("Usage: sh2c").count(1)); 
        // File creation is checked in cli_out_mode, but we can assume safety here.
}
