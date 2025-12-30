use std::process::Command;

fn sh2c_path() -> String {
    env!("CARGO_BIN_EXE_sh2c").to_string()
}



fn assert_cmd_stdout_str(args: &[&str], expected: &str) {
    let output = Command::new(sh2c_path())
        .args(args)
        .output()
        .expect("Failed to run sh2c");

    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("sh2c failed with exit code {}", output.status);
    }

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");

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
    // Note: CLI uses script parent as base, so paths are basename-only here.
   
     let expected = r#"
__sh2_err_handler() { local s=$?; if [[ "${BASH_COMMAND}" == *"(exit "* ]]; then return $s; fi; printf "Error in %s\n" "${__sh2_loc:-unknown}" >&2; return $s; }
set -o errtrace
trap '__sh2_err_handler' ERR
main() {
  local __sh2_loc=""
  __sh2_loc="cli_target_basic.sh2:2:3"
  x='world'
  __sh2_loc="cli_target_basic.sh2:3:3"
  'echo' 'hello' "$x"; __sh2_status=$?; (exit $__sh2_status)
}
__sh2_status=0
main "$@"
"#;
    assert_cmd_stdout_str(
        &["tests/fixtures/cli_target_basic.sh2"],
        expected,
    );
}

#[test]
fn cli_target_explicit_bash() {
    let expected = r#"
__sh2_err_handler() { local s=$?; if [[ "${BASH_COMMAND}" == *"(exit "* ]]; then return $s; fi; printf "Error in %s\n" "${__sh2_loc:-unknown}" >&2; return $s; }
set -o errtrace
trap '__sh2_err_handler' ERR
main() {
  local __sh2_loc=""
  __sh2_loc="cli_target_basic.sh2:2:3"
  x='world'
  __sh2_loc="cli_target_basic.sh2:3:3"
  'echo' 'hello' "$x"; __sh2_status=$?; (exit $__sh2_status)
}
__sh2_status=0
main "$@"
"#;
    assert_cmd_stdout_str(
        &["--target", "bash", "tests/fixtures/cli_target_basic.sh2"],
        expected,
    );
}

#[test]
fn cli_target_explicit_bash_equal() {
    let expected = r#"
__sh2_err_handler() { local s=$?; if [[ "${BASH_COMMAND}" == *"(exit "* ]]; then return $s; fi; printf "Error in %s\n" "${__sh2_loc:-unknown}" >&2; return $s; }
set -o errtrace
trap '__sh2_err_handler' ERR
main() {
  local __sh2_loc=""
  __sh2_loc="cli_target_basic.sh2:2:3"
  x='world'
  __sh2_loc="cli_target_basic.sh2:3:3"
  'echo' 'hello' "$x"; __sh2_status=$?; (exit $__sh2_status)
}
__sh2_status=0
main "$@"
"#;
    assert_cmd_stdout_str(
        &["--target=bash", "tests/fixtures/cli_target_basic.sh2"],
        expected,
    );
}

#[test]
fn cli_target_posix() {
    let expected = r#"
main() {
  __sh2_loc="cli_target_basic.sh2:2:3"
  x='world'
  __sh2_loc="cli_target_basic.sh2:3:3"
  'echo' 'hello' "$x"; __sh2_status=$?; if [ $__sh2_status -ne 0 ]; then printf 'Error in %s\n' "$__sh2_loc" >&2; exit $__sh2_status; fi
}
__sh2_status=0
main "$@"
"#;
    assert_cmd_stdout_str(
        &["--target", "posix", "tests/fixtures/cli_target_basic.sh2"],
        expected,
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
