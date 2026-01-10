use std::process::Command;
use assert_cmd::prelude::*;
// use predicates::prelude::*; 

#[test]
fn test_status_probe_behavior() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    let fixture_path = "tests/fixtures/status_probe_die_message.sh2";
    
    // Test with Bash target
    let assert = cmd
        .arg("--target")
        .arg("bash")
        .arg(fixture_path)
        .assert();

    let output = assert.success().get_output().stdout.clone();
    let script = String::from_utf8(output).unwrap();

    // Run the generated script
    let mut shell_cmd = Command::new("bash");
    let shell_assert = shell_cmd
        .arg("-c")
        .arg(&script)
        .assert();

    // Expect failure (exit 1) AND custom error message
    shell_assert
        .failure()
        .code(1)
        .stderr(predicates::str::contains("Custom Error Message"));
}
