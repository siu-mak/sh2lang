use std::process::Command;

fn sh2do_path() -> String {
    env!("CARGO_BIN_EXE_sh2do").to_string()
}

#[test]
fn test_basic_execution() {
    let output = Command::new(sh2do_path())
        .arg(r#"print("hi")"#)
        .output()
        .expect("Failed to run sh2do");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "sh2do should exit with code 0. stderr: {}",
        stderr
    );
    assert_eq!(
        stdout.trim(),
        "hi",
        "Expected 'hi' on stdout, got: {}",
        stdout
    );
}

#[test]
fn test_exit_code_propagation() {
    let output = Command::new(sh2do_path())
        .arg(r#"exit(7)"#)
        .output()
        .expect("Failed to run sh2do");

    assert_eq!(
        output.status.code(),
        Some(7),
        "sh2do should exit with code 7"
    );
}

#[test]
fn test_stdin_execution() {
    let mut child = Command::new(sh2do_path())
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn sh2do");

    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().expect("Failed to get stdin");
        stdin
            .write_all(b"print(\"hi\")")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for sh2do");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "sh2do should exit with code 0. stderr: {}",
        stderr
    );
    assert_eq!(
        stdout.trim(),
        "hi",
        "Expected 'hi' on stdout, got: {}",
        stdout
    );
}
