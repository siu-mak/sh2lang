use std::process::Command;

fn sh2do_path() -> String {
    env!("CARGO_BIN_EXE_sh2do").to_string()
}

#[test]
fn test_emit_sh_from_arg() {
    let output = Command::new(sh2do_path())
        .arg(r#"print("hi")"#)
        .arg("--emit-sh")
        .output()
        .expect("Failed to run sh2do");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "sh2do --emit-sh should exit with code 0. stderr: {}",
        stderr
    );

    // Should contain shell script markers
    assert!(
        stdout.contains("main()"),
        "Expected shell script with 'main()' function, got: {}",
        stdout
    );

    // Should NOT contain execution output
    assert!(
        !stdout.contains("hi\n") && !stdout.ends_with("hi"),
        "Should not execute and print 'hi', got: {}",
        stdout
    );
}

#[test]
fn test_no_exec_alias() {
    let output = Command::new(sh2do_path())
        .arg(r#"print("hi")"#)
        .arg("--no-exec")
        .output()
        .expect("Failed to run sh2do");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "sh2do --no-exec should exit with code 0. stderr: {}",
        stderr
    );

    // Should contain shell script markers
    assert!(
        stdout.contains("main()"),
        "Expected shell script with 'main()' function, got: {}",
        stdout
    );

    // Should NOT contain execution output
    assert!(
        !stdout.contains("hi\n") && !stdout.ends_with("hi"),
        "Should not execute and print 'hi', got: {}",
        stdout
    );
}

#[test]
fn test_emit_sh_from_stdin() {
    let mut child = Command::new(sh2do_path())
        .arg("-")
        .arg("--emit-sh")
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
        "sh2do - --emit-sh should exit with code 0. stderr: {}",
        stderr
    );

    // Should contain shell script
    assert!(
        stdout.contains("main()"),
        "Expected shell script, got: {}",
        stdout
    );

    // Should NOT execute
    assert!(
        !stdout.contains("hi\n") && !stdout.ends_with("hi"),
        "Should not execute, got: {}",
        stdout
    );
}

#[test]
fn test_emit_sh_compile_error() {
    let output = Command::new(sh2do_path())
        .arg("invalid syntax here")
        .arg("--emit-sh")
        .output()
        .expect("Failed to run sh2do");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should exit with non-zero code
    assert!(
        !output.status.success(),
        "sh2do should fail on compile error"
    );

    // Should have error in stderr
    assert!(
        stderr.contains("Expected") || stderr.contains("error"),
        "Expected compile error in stderr, got: {}",
        stderr
    );
}
