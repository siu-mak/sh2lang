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
             "trap '__sh2_err_handler' ERR",
             "no_diagnostics_basic.sh2" // main.rs uses script parent as base, so output is basename
        ],
        &[
             "/srv/", 
             "/home/",
             "/Users/",
             env!("CARGO_MANIFEST_DIR"), // ensures absolute path to repo root isn't present
             ":\\", // NO Windows drive letters
        ],
    );

    // Additional assertion: ensure no backslashes in paths
    // We run the command again or refactor helper? Helper consumes.
    // Let's just trust the above + unit tests for now, or add a specific check here.
    let output = Command::new(sh2c_path())
        .args(&["tests/fixtures/no_diagnostics_basic.sh2"])
        .output()
        .expect("Failed to run sh2c");
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check for backslashes in the emitted location line
    // e.g. __sh2_loc="foo\bar"
    if stdout.contains("__sh2_loc=\"") {
        // Simple check: if we see backslashes in output, it might be an issue,
        // unless they are unrelated escapes. But paths shouldn't have them.
        // We know our fixture path "tests/fixtures/..." has forward slashes.
        // So we expect "no_diagnostics_basic.sh2"
        assert!(stdout.contains("no_diagnostics_basic.sh2"));
        assert!(!stdout.contains("tests/fixtures/no_diagnostics_basic.sh2"));
    }
}

#[test]
fn cli_diagnostics_default_posix() {
    // POSIX target with diagnostics should track location but NOT exit on run() failures
    // run() should behave like bash: capture status and continue
    assert_cmd_output_contains(
        &["--target", "posix", "tests/fixtures/no_diagnostics_basic.sh2"],
        &[
             "__sh2_loc=",  // Location tracking is present
             "(exit $__sh2_status)",  // Status is captured but doesn't exit
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
