use std::process::Command;

fn sh2c_path() -> String {
    env!("CARGO_BIN_EXE_sh2c").to_string()
}

fn assert_cmd_output_contains(args: &[&str], contains: &[&str], not_contains: &[&str]) {
    let output = Command::new(sh2c_path())
        .args(args)
        .output()
        .expect("Failed to run sh2c");

    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("sh2c failed with exit code {}", output.status);
    }

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
    
    for c in contains {
        assert!(stdout.contains(c), "Expected output to contain '{}', but it didn't.\nStdout:\n{}", c, stdout);
    }
    for c in not_contains {
        assert!(!stdout.contains(c), "Expected output to NOT contain '{}', but it did.\nStdout:\n{}", c, stdout);
    }
}

#[test]
fn cli_no_diagnostics_bash() {
    assert_cmd_output_contains(
        &["--no-diagnostics", "tests/fixtures/no_diagnostics_basic.sh2"],
        &[
            "main() {", // sanity check
        ], 
        &[   // not contains
             "__sh2_loc=",
             "__sh2_err_handler", 
             "trap '__sh2_err_handler' ERR" 
        ],
    );
}

#[test]
fn cli_no_diagnostics_posix() {
    assert_cmd_output_contains(
        &["--target", "posix", "--no-diagnostics", "tests/fixtures/no_diagnostics_basic.sh2"],
        &[
             "main() {",
        ],
        &[
             "__sh2_loc=",
             "Error in " 
        ],
    );
}

#[test]
fn cli_diagnostics_default_bash() {
    assert_cmd_output_contains(
        &["tests/fixtures/no_diagnostics_basic.sh2"],
        &[
             "__sh2_loc=",
             "__sh2_err_handler",
             "trap '__sh2_err_handler' ERR"
        ],
        &[],
    );
}

#[test]
fn cli_diagnostics_default_posix() {
    // Using pipeline fixture to trigger "Error in ..." logic more explicitly if basic doesn't cover all cases,
    // but basic run() should trigger it in posix codegen too.
    assert_cmd_output_contains(
        &["--target", "posix", "tests/fixtures/no_diagnostics_basic.sh2"],
        &[
             "if [ $__sh2_status -ne 0 ]; then printf 'Error in %s\\n' \"$__sh2_loc\"",
        ],
        &[],
    );
}

#[test]
fn cli_no_diagnostics_pipeline_bash() {
    assert_cmd_output_contains(
        &["--no-diagnostics", "tests/fixtures/no_diagnostics_pipeline.sh2"],
        &[],
        &[
             "__sh2_loc=",
             "__sh2_err_handler"
        ],
    );
}

#[test]
fn cli_no_diagnostics_pipeline_posix() {
    assert_cmd_output_contains(
        &["--target", "posix", "--no-diagnostics", "tests/fixtures/no_diagnostics_pipeline.sh2"],
        &[],
        &[
             "__sh2_loc=",
             "Error in "
        ],
    );
}
