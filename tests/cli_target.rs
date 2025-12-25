use std::process::Command;
use std::fs;

fn run_cli(args: &[&str], input_file: &str) -> std::process::Output {
    let bin_path = env!("CARGO_BIN_EXE_sh2c");
    let mut cmd = Command::new(bin_path);
    cmd.args(args);
    cmd.arg(input_file);
    cmd.output().expect("Failed to execute sh2c binary")
}

fn assert_output_matches(output: std::process::Output, expected_path: &str) {
    assert!(output.status.success(), "CLI failed with status: {:?}\nStderr: {}", output.status, String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8(output.stdout).expect("Stdout not utf8");
    let expected = fs::read_to_string(expected_path).expect("Failed to read expected output");
    
    // Normalize newlines and trim for robust comparison
    let stdout_norm = stdout.replace("\r\n", "\n");
    let expected_norm = expected.replace("\r\n", "\n");
    
    assert_eq!(stdout_norm.trim(), expected_norm.trim(), "Output mismatch");
}

#[test]
fn cli_default_is_bash() {
    let output = run_cli(&[], "tests/fixtures/posix_params.sh2");
    assert_output_matches(output, "tests/fixtures/posix_params.sh.expected");
}

#[test]
fn cli_explicit_bash() {
    let output = run_cli(&["--target", "bash"], "tests/fixtures/posix_params.sh2");
    assert_output_matches(output, "tests/fixtures/posix_params.sh.expected");
}

#[test]
fn cli_explicit_posix() {
    let output = run_cli(&["--target", "posix"], "tests/fixtures/posix_params.sh2");
    assert_output_matches(output, "tests/fixtures/posix_params.posix.sh.expected");
}

#[test]
fn cli_equals_bash() {
    let output = run_cli(&["--target=bash"], "tests/fixtures/posix_params.sh2");
    assert_output_matches(output, "tests/fixtures/posix_params.sh.expected");
}

#[test]
fn cli_equals_posix() {
    let output = run_cli(&["--target=posix"], "tests/fixtures/posix_params.sh2");
    assert_output_matches(output, "tests/fixtures/posix_params.posix.sh.expected");
}

#[test]
fn cli_invalid_target() {
    let bin_path = env!("CARGO_BIN_EXE_sh2c");
    let output = Command::new(bin_path)
        .arg("--target")
        .arg("invalid")
        .arg("tests/fixtures/posix_params.sh2")
        .output()
        .expect("Failed to start");
        
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid target: invalid"));
}

#[test]
fn cli_missing_arg() {
    let bin_path = env!("CARGO_BIN_EXE_sh2c");
    let output = Command::new(bin_path)
        .arg("--target")
        .output()
        .expect("Failed to start");
        
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--target requires an argument") || stderr.contains("Usage:"));
}
