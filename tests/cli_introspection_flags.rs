use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_cli_emit_ast() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    let fixture = "tests/fixtures/cli_emit_ast.sh2";
    let expected = fs::read_to_string("tests/fixtures/cli_emit_ast.stdout.expected").unwrap();

    cmd.arg("--emit-ast")
        .arg(fixture)
        .assert()
        .success()
        .stdout(predicate::eq(expected));
}

#[test]
fn test_cli_emit_ir() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    let fixture = "tests/fixtures/cli_emit_ir.sh2";
    let expected = fs::read_to_string("tests/fixtures/cli_emit_ir.stdout.expected").unwrap();

    cmd.arg("--emit-ir")
        .arg(fixture)
        .assert()
        .success()
        .stdout(predicate::eq(expected));
}

#[test]
fn test_cli_emit_sh() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    let fixture = "tests/fixtures/cli_emit_sh.sh2";
    let expected = fs::read_to_string("tests/fixtures/cli_emit_sh.stdout.expected").unwrap();

    // --emit-sh should match default output but explicit flag
    cmd.arg("--emit-sh")
        .arg("--no-diagnostics") // consistent with expected
        .arg(fixture)
        .assert()
        .success()
        .stdout(predicate::eq(expected));
}

#[test]
fn test_cli_check_ok() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    let fixture = "tests/fixtures/cli_check_ok.sh2";
    // Expected is hardcoded to avoid file newline issues
    let expected = "OK\n";

    cmd.arg("--check")
        .arg(fixture)
        .assert()
        .success()
        .stdout(predicate::eq(expected));
}

#[test]
fn test_cli_check_err() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    let fixture = "tests/fixtures/cli_check_err.sh2";
    // We expect failure. Reading stderr from file is fine as long as we use contains/trim.
    let expected_stderr = fs::read_to_string("tests/fixtures/cli_check_err.stderr.expected").unwrap();

    cmd.arg("--check")
        .arg(fixture)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unexpected character: ;"));
}

#[test]
fn test_cli_emit_multi_flags() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    let fixture = "tests/fixtures/cli_check_ok.sh2";

    cmd.arg("--emit-ast")
        .arg("--check")
        .arg(fixture)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: multple action flags specified"));
}
