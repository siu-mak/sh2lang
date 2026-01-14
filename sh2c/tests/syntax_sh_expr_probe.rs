use std::process::Command;
use assert_cmd::prelude::*;

#[test]
fn test_sh_expr_var() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    let fixture_path = "tests/fixtures/sh_expr_var_probe.sh2";
    
    // Test with Bash target
    let assert = cmd
        .arg("--target")
        .arg("bash")
        .arg(fixture_path)
        .assert();

    let output = assert.success().get_output().stdout.clone();
    let script = String::from_utf8(output).unwrap();

    let shell_assert = Command::new("bash")
        .arg("-c")
        .arg(&script)
        .assert();

    shell_assert
        .success()
        .stdout(predicates::str::contains("hello"));

    // POSIX test
    let mut cmd_posix = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    let assert_posix = cmd_posix
        .arg("--target")
        .arg("posix")
        .arg(fixture_path)
        .assert();

    let output_posix = assert_posix.success().get_output().stdout.clone();
    let script_posix = String::from_utf8(output_posix).unwrap();

    let shell_assert_posix = Command::new("sh")
        .arg("-c")
        .arg(&script_posix)
        .assert();

    shell_assert_posix
        .success()
        .stdout(predicates::str::contains("hello"));
}

#[test]
fn test_sh_expr_concat() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    let fixture_path = "tests/fixtures/sh_expr_concat_probe.sh2";
    
    // Bash
    let assert = cmd
        .arg("--target")
        .arg("bash")
        .arg(fixture_path)
        .assert();

    let output = assert.success().get_output().stdout.clone();
    let script = String::from_utf8(output).unwrap();

    Command::new("bash")
        .arg("-c")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicates::str::contains("dynamic"));

    // POSIX
    let mut cmd_posix = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    let assert_posix = cmd_posix
        .arg("--target")
        .arg("posix")
        .arg(fixture_path)
        .assert();

    let output_posix = assert_posix.success().get_output().stdout.clone();
    let script_posix = String::from_utf8(output_posix).unwrap();

    Command::new("sh")
        .arg("-c")
        .arg(&script_posix)
        .assert()
        .success()
        .stdout(predicates::str::contains("dynamic"));
}

#[test]
fn test_sh_probe_no_fail_fast() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    let fixture_path = "tests/fixtures/sh_probe_no_fail_fast.sh2";
    
    // Bash
    let assert = cmd
        .arg("--target")
        .arg("bash")
        .arg(fixture_path)
        .assert();

    let output = assert.success().get_output().stdout.clone();
    let script = String::from_utf8(output).unwrap();

    Command::new("bash")
        .arg("-c")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicates::str::contains("after"));

    // POSIX
    let mut cmd_posix = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    let assert_posix = cmd_posix
        .arg("--target")
        .arg("posix")
        .arg(fixture_path)
        .assert();

    let output_posix = assert_posix.success().get_output().stdout.clone();
    let script_posix = String::from_utf8(output_posix).unwrap();

    Command::new("sh")
        .arg("-c")
        .arg(&script_posix)
        .assert()
        .success()
        .stdout(predicates::str::contains("after"));
}
