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
    // Ensure we use absolute path for input if needed, or relative to CWD.
    // assert_cmd runs from CWD? Usually yes.
    let input_path = "tests/fixtures/cli_target_basic.sh2";

    cmd.arg("--target")
        .arg("posix")
        .arg("--out")
        .arg(&out_path)
        .arg(input_path)
        .assert()
        .success()
        .stdout("");

    assert!(out_path.exists(), "Output file was not created");
    
    let content = fs::read_to_string(&out_path).unwrap();
    // Verify content briefly
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
        .arg(&out_path)
        .arg(input_path)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("--check cannot be used with --out"));

    assert!(!out_path.exists(), "Output file SHOULD NOT be created on conflict");
}
