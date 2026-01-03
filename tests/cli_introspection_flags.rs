use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_cli_emit_ast() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
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
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
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
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
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
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    let fixture = "tests/fixtures/cli_check_ok.sh2";
    let expected = fs::read_to_string("tests/fixtures/cli_check_ok.stdout.expected").unwrap();

    cmd.arg("--check")
        .arg(fixture)
        .assert()
        .success()
        .stdout(predicate::eq(expected));
}

#[test]
fn test_cli_check_err() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    let fixture = "tests/fixtures/cli_check_err.sh2";
    let expected_stderr = fs::read_to_string("tests/fixtures/cli_check_err.stderr.expected").unwrap();

    cmd.arg("--check")
        .arg(fixture)
        .assert()
        .failure()
        .stderr(predicate::eq(expected_stderr));
}

#[test]
fn test_cli_emit_multi_flags() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    let fixture = "tests/fixtures/cli_check_ok.sh2";
    let expected_stderr = fs::read_to_string("tests/fixtures/cli_emit_multi_flags.stderr.expected").unwrap();

    cmd.arg("--emit-ast")
        .arg("--check")
        .arg(fixture)
        .assert()
        .failure()
        .stderr(predicate::eq(expected_stderr));
}
