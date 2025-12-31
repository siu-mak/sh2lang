use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[test]
fn writes_file_and_is_executable() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("out.sh");
    let input_path = "tests/fixtures/cli_target_basic.sh2";

    cmd.arg("--target")
        .arg("posix")
        .arg("--out")
        .arg(out_path.to_str().unwrap())
        .arg(input_path)
        .assert()
        .success()
        .stdout(""); // stdout should be empty

    assert!(out_path.exists(), "Output file was not created");
    
    let content = fs::read_to_string(&out_path).unwrap();
    assert!(content.starts_with("#!/bin/sh"), "Should have POSIX shebang");

    #[cfg(unix)]
    {
        let metadata = fs::metadata(&out_path).unwrap();
        let mode = metadata.permissions().mode();
        assert_ne!(mode & 0o111, 0, "File should be executable");
    }
}

#[test]
fn check_and_out_is_error() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("x.sh");
    let input_path = "tests/fixtures/cli_target_basic.sh2";

    cmd.arg("--check")
        .arg("--out")
        .arg(out_path.to_str().unwrap())
        .arg(input_path)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("--check cannot be used with --out"));

    assert!(!out_path.exists(), "Output file SHOULD NOT be created on conflict");
}

#[test]
fn out_io_error_exit_1() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    // Create a directory, which cannot be overwritten by fs::write
    let dir_path = temp_dir.path().join("dirout");
    fs::create_dir(&dir_path).unwrap();
    let input_path = "tests/fixtures/cli_target_basic.sh2";

    cmd.arg("--target")
        .arg("posix")
        .arg("--out")
        .arg(dir_path.to_str().unwrap())
        .arg(input_path)
        .assert()
        .failure()
        .code(1); // I/O errors should be exit code 1
}

#[test]
fn out_no_chmod_x_does_not_set_exec_bit() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("no_exec.sh");
    let input_path = "tests/fixtures/cli_target_basic.sh2";

    cmd.arg("--target")
        .arg("posix")
        .arg("--out")
        .arg(out_path.to_str().unwrap())
        .arg("--no-chmod-x")
        .arg(input_path)
        .assert()
        .success()
        .stdout("");

    assert!(out_path.exists());
    let content = fs::read_to_string(&out_path).unwrap();
    assert!(content.starts_with("#!/bin/sh"));

    #[cfg(unix)]
    {
        let metadata = fs::metadata(&out_path).unwrap();
        let mode = metadata.permissions().mode();
        // Check execute bit is NOT set
        assert_eq!(mode & 0o111, 0, "File should NOT be executable with --no-chmod-x");
    }
}

#[test]
fn no_chmod_x_without_out_is_usage_error() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    let input_path = "tests/fixtures/cli_target_basic.sh2";

    cmd.arg("--no-chmod-x")
        .arg(input_path)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("error: --no-chmod-x/--chmod-x require --out"))
        .stderr(predicate::str::contains("Usage: sh2c").count(1));
}

#[test]
fn chmod_x_explicit_ok() {
    let mut cmd = Command::cargo_bin("sh2c").unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("exec.sh");
    let input_path = "tests/fixtures/cli_target_basic.sh2";

    cmd.arg("--out")
        .arg(out_path.to_str().unwrap())
        .arg("--chmod-x")
        .arg(input_path)
        .assert()
        .success();

    #[cfg(unix)]
    {
        let metadata = fs::metadata(&out_path).unwrap();
        let mode = metadata.permissions().mode();
        assert_ne!(mode & 0o111, 0, "File should be executable");
    }
}

#[test]
fn conflicting_chmod_flags_error() {
     let mut cmd = Command::cargo_bin("sh2c").unwrap();
    let input_path = "tests/fixtures/cli_target_basic.sh2";
    
    cmd.arg("--no-chmod-x")
       .arg("--chmod-x")
       .arg(input_path)
       .assert()
       .failure()
       .code(1)
       .stderr(predicate::str::contains("error: --no-chmod-x cannot be used with --chmod-x"));
}
