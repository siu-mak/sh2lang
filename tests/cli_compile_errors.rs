use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_compile_error_log_posix() {
    let mut cmd = Command::cargo_bin("sh2c").expect("Failed to get binary");
    cmd.arg("--target").arg("posix")
        .arg("tests/fixtures/unsupported_log.sh2")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("compile error: with log(...) is not supported in POSIX sh target"));
}

#[test]
fn test_compile_error_no_backtrace() {
    let mut cmd = Command::cargo_bin("sh2c").expect("Failed to get binary");
    cmd.env_remove("RUST_BACKTRACE")
        .arg("--target").arg("posix")
        .arg("tests/fixtures/unsupported_log.sh2")
        .assert()
        .failure()
        .stderr(predicate::str::contains("stack backtrace:").not());
}
