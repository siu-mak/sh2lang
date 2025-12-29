use std::fs;
use std::process::Command;

fn sh2c_path() -> String {
    env!("CARGO_BIN_EXE_sh2c").to_string()
}

fn assert_cmd_stdout(args: &[&str], expected_file: &str) {
    let output = Command::new(sh2c_path())
        .args(args)
        .output()
        .expect("Failed to run sh2c");

    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("sh2c failed with exit code {}", output.status);
    }

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    let expected = fs::read_to_string(expected_file).expect("Failed to read expected file");

    assert_eq!(stdout.trim(), expected.trim());
}

fn assert_cmd_fail(args: &[&str], expected_status: Option<i32>, expected_stderr_part: &str) {
    let output = Command::new(sh2c_path())
        .args(args)
        .output()
        .expect("Failed to run sh2c");

    if output.status.success() {
        panic!("sh2c succeeded unexpectedly");
    }

    if let Some(code) = expected_status {
        assert_eq!(output.status.code(), Some(code), "Exit code mismatch");
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(expected_stderr_part),
        "Stderr did not contain expected text. Got:\n{}",
        stderr
    );
}

#[test]
fn cli_target_default_is_bash() {
    // Default should match cli_target_basic.sh.expected (bash)
    assert_cmd_stdout(
        &["tests/fixtures/cli_target_basic.sh2"],
        "tests/fixtures/cli_target_basic.sh.expected",
    );
}

#[test]
fn cli_target_explicit_bash() {
    assert_cmd_stdout(
        &["--target", "bash", "tests/fixtures/cli_target_basic.sh2"],
        "tests/fixtures/cli_target_basic.sh.expected",
    );
}

#[test]
fn cli_target_explicit_bash_equal() {
    assert_cmd_stdout(
        &["--target=bash", "tests/fixtures/cli_target_basic.sh2"],
        "tests/fixtures/cli_target_basic.sh.expected",
    );
}

#[test]
fn cli_target_posix() {
    assert_cmd_stdout(
        &["--target", "posix", "tests/fixtures/cli_target_basic.sh2"],
        "tests/fixtures/cli_target_basic.posix.sh.expected",
    );
}

#[test]
fn cli_target_invalid_value() {
    assert_cmd_fail(
        &["--target", "fish", "tests/fixtures/cli_target_basic.sh2"],
        Some(1),
        "Invalid target",
    );
}

#[test]
fn cli_target_missing_arg() {
    assert_cmd_fail(&["--target"], Some(1), "--target requires an argument");
}

#[test]
fn cli_target_posix_rejects_array() {
    // This should fail with exit code 2 and a message about list values
    assert_cmd_fail(
        &[
            "--target",
            "posix",
            "tests/fixtures/cli_target_posix_rejects_array.sh2",
        ],
        Some(2),
        "Array assignment is not supported",
    );
}
