use std::process::Command;

fn sh2do_path() -> String {
    env!("CARGO_BIN_EXE_sh2do").to_string()
}

#[test]
fn test_single_arg_passthrough() {
    let output = Command::new(sh2do_path())
        .arg(r#"print(arg(1))"#)
        .arg("--")
        .arg("hello")
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
        "hello",
        "Expected 'hello' on stdout, got: {}",
        stdout
    );
}

#[test]
fn test_multiple_args_passthrough() {
    let output = Command::new(sh2do_path())
        .arg(r#"print(argc())"#)
        .arg("--")
        .arg("a")
        .arg("b")
        .arg("c")
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
        "3",
        "Expected '3' on stdout, got: {}",
        stdout
    );
}

#[test]
fn test_passthrough_stdin() {
    let mut child = Command::new(sh2do_path())
        .arg("-")
        .arg("--")
        .arg("world")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn sh2do");

    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().expect("Failed to get stdin");
        stdin
            .write_all(b"print(arg(1))")
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
        "world",
        "Expected 'world' on stdout, got: {}",
        stdout
    );
}

#[test]
fn test_emit_ignores_passthrough() {
    let output = Command::new(sh2do_path())
        .arg(r#"print(arg(1))"#)
        .arg("--emit-sh")
        .arg("--")
        .arg("hello")
        .output()
        .expect("Failed to run sh2do");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "sh2do --emit-sh should exit with code 0. stderr: {}",
        stderr
    );

    // Should contain shell script
    assert!(
        stdout.contains("main()"),
        "Expected shell script, got: {}",
        stdout
    );

    // Should NOT contain "hello" (not executed)
    assert!(
        !stdout.contains("hello\n") && !stdout.ends_with("hello"),
        "Should not execute and print 'hello', got: {}",
        stdout
    );
}
