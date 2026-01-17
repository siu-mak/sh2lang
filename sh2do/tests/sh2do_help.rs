use std::process::Command;

fn sh2do_path() -> String {
    env!("CARGO_BIN_EXE_sh2do").to_string()
}

#[test]
fn test_help_exits_cleanly() {
    let output = Command::new(sh2do_path())
        .arg("--help")
        .output()
        .expect("Failed to run sh2do");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "sh2do --help should exit with code 0. stderr: {}",
        stderr
    );

    assert!(
        stdout.contains("Usage:"),
        "Help text should contain 'Usage:', got: {}",
        stdout
    );

    assert!(
        stdout.contains("--emit-sh"),
        "Help text should contain '--emit-sh', got: {}",
        stdout
    );

    assert!(
        stderr.is_empty(),
        "stderr should be empty, got: {}",
        stderr
    );
}

#[test]
fn test_short_help_flag() {
    let output = Command::new(sh2do_path())
        .arg("-h")
        .output()
        .expect("Failed to run sh2do");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "sh2do -h should exit with code 0. stderr: {}",
        stderr
    );

    assert!(
        stdout.contains("Usage:"),
        "Help text should contain 'Usage:', got: {}",
        stdout
    );

    assert!(
        stdout.contains("--emit-sh"),
        "Help text should contain '--emit-sh', got: {}",
        stdout
    );

    assert!(
        stderr.is_empty(),
        "stderr should be empty, got: {}",
        stderr
    );
}

#[test]
fn test_help_does_not_execute() {
    let output = Command::new(sh2do_path())
        .arg("--help")
        .arg(r#"print("hi")"#)
        .output()
        .expect("Failed to run sh2do");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "sh2do --help should exit with code 0. stderr: {}",
        stderr
    );

    assert!(
        stdout.contains("Usage:"),
        "Help text should contain 'Usage:', got: {}",
        stdout
    );

    // Should NOT execute and print "hi"
    assert!(
        !stdout.contains("hi\n") && !stdout.ends_with("hi"),
        "Should not execute snippet, got: {}",
        stdout
    );
}

#[test]
fn test_help_examples() {
    let output = Command::new(sh2do_path())
        .arg("--help")
        .output()
        .expect("Failed to run sh2do");

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check for "Examples:" header
    assert!(
        stdout.contains("Examples:"),
        "Help text should contain 'Examples:', got: {}",
        stdout
    );

    // Check for the specific example lines
    assert!(
        stdout.contains("sh2do 'print(\"hi\")'"),
        "Help text should contain 'print(\"hi\")' example, got: {}", 
        stdout
    );

    assert!(
        stdout.contains("sh2do 'run(\"ls\")'"),
        "Help text should contain 'run(\"ls\")' example, got: {}", 
        stdout
    );
}
