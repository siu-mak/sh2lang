use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_syntax_range_literal_ok() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("tests/fixtures/syntax_range_literal_ok.sh2");
    
    let output = Command::new(env!("CARGO_BIN_EXE_sh2c"))
        .arg("--check")
        .arg(fixture)
        .output()
        .expect("Failed to run sh2c");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("sh2c --check failed:\n{}", stderr);
    }
}

#[test]
fn test_syntax_for_list_paren_ok() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("tests/fixtures/syntax_for_list_paren_ok.sh2");
    
    // 1. Compile with sh2c
    let compile_output = Command::new(env!("CARGO_BIN_EXE_sh2c"))
        .arg(fixture)
        .output()
        .expect("Failed to run sh2c");

    assert!(compile_output.status.success(), "sh2c compilation failed: {}", String::from_utf8_lossy(&compile_output.stderr));
    
    // 2. Run the generated script with bash
    let mut child = Command::new("bash")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn bash");

    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(&compile_output.stdout).expect("Failed to write to bash stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on bash");

    assert!(output.status.success(), 
        "Runtime check failed! Bash exited with code: {}. Stderr: {}", 
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = "1\n2\n3\n";
    assert_eq!(stdout, expected, "Unexpected stdout");
}

#[test]
fn test_syntax_range_expr_rhs_ok() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("tests/fixtures/syntax_range_expr_rhs_ok.sh2");
    
    let output = Command::new(env!("CARGO_BIN_EXE_sh2c"))
        .arg("--check")
        .arg(fixture)
        .output()
        .expect("Failed to run sh2c");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("sh2c --check failed:\n{}", stderr);
    }
}

#[test]
fn test_syntax_range_paren_ok() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("tests/fixtures/syntax_range_paren_ok.sh2");
    
    let output = Command::new(env!("CARGO_BIN_EXE_sh2c"))
        .arg("--check")
        .arg(fixture)
        .output()
        .expect("Failed to run sh2c");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("sh2c --check failed:\n{}", stderr);
    }
}

#[test]
fn test_syntax_range_spaced_ok() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("tests/fixtures/syntax_range_spaced_ok.sh2");
    
    let output = Command::new(env!("CARGO_BIN_EXE_sh2c"))
        .arg("--check")
        .arg(fixture)
        .output()
        .expect("Failed to run sh2c");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("sh2c --check failed:\n{}", stderr);
    }
}

#[test]
fn test_syntax_for_list_empty_paren_ok() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("tests/fixtures/syntax_for_list_empty_paren_ok.sh2");
    
    // 1. Compile with sh2c
    let compile_output = Command::new(env!("CARGO_BIN_EXE_sh2c"))
        .arg(fixture)
        .output()
        .expect("Failed to run sh2c");

    assert!(compile_output.status.success(), "sh2c compilation failed: {}", String::from_utf8_lossy(&compile_output.stderr));
    
    let generated_bash = String::from_utf8_lossy(&compile_output.stdout);
    println!("DEBUG GENERATED BASH:\n{}", generated_bash);
    // Structural check: ensure we don't see patterns that might iterate over $@
    // These patterns would be bugs for "for x in ()"
    // Note: The script ends with `main "$@"` so we must exclude that.
    // We check that meaningful distinct lines do not contain `in "$@"`.
    for line in generated_bash.lines() {
        if line.trim() == "main \"$@\"" { continue; }
        assert!(!line.contains("in \"$@\""), "Codegen error: explicitly iterating over \"$@\" in line: {}", line);
        assert!(!line.contains("in $@"), "Codegen error: iterating over $@ in line: {}", line);
        assert!(!line.contains("in; do"), "Codegen error: found 'in; do' in line: {}", line);
    }
    // Note: 'in $@' and 'in; do' are also dangerous in some contexts/shells
    assert!(!generated_bash.contains("in $@"), "Codegen error: iterating over $@"); 
    // "for x in; do" iterates over positional params in standard sh
    // We want to ensure we don't accidentally emit this if the list is empty
    // But be careful not to match "for x in ...; do"
    assert!(!generated_bash.contains("in; do"), "Codegen error: found 'in; do' which iterates positional params");
    
    // 2. Run the generated script with bash
    // The fixture exits with 1 if the loop body is entered.
    // Harden: pass dummy args to ensure we don't accidentally iterate over $@
    let mut child = Command::new("bash")
        .arg("-s")
        .arg("--")
        .arg("DUMMY_SHOULD_NOT_ITERATE")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn bash");

    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(&compile_output.stdout).expect("Failed to write to bash stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on bash");

    assert!(output.status.success(), 
        "Runtime check failed! The loop body was likely executed (exit code {}). Stderr: {}", 
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stderr)
    );
    
    // Ensure no output (double chcek)
    assert!(output.stdout.is_empty(), "Runtime check failed! Output was not empty, loop body likely executed.");
}

#[test]
fn test_range_codegen_uses_seq() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir.join("tests/fixtures/syntax_range_literal_ok.sh2");
    
    let output = Command::new(env!("CARGO_BIN_EXE_sh2c"))
        .arg(&fixture)
        .output()
        .expect("Failed to compile");
    
    assert!(output.status.success(), "Compilation failed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Verify essential structure (tolerant of quote style)
    assert!(stdout.contains("for "), "Expected for loop");
    assert!(stdout.contains("$(seq"), "Expected $(seq ...) in output");
    assert!(stdout.contains("); do"), "Expected ); do pattern");
    
    // Harden arg check: find "$(seq" and ensure it has arguments before ")"
    // Robust check: extract the seq command and check it contains the start/end numbers
    // avoiding split_whitespace() issues with potential extra spaces/quotes.
    let start_idx = stdout.find("$(seq").expect("Could not find $(seq");
    let seq_call = &stdout[start_idx..];
    let end_idx = seq_call.find(')').expect("Could not find closing parenthesis for seq");
    let seq_inner = &seq_call[6..end_idx]; // skip "$(seq "
    
    // Check that we have the expected literals '1' and '10' in the arguments, regardless of exact quoting
    // Use containment checking for robust detection of quoted or unquoted variants
    // Allow: 1 10, '1' '10', "1" "10"
    let has_args = 
        (seq_inner.contains(" 1 ") && seq_inner.contains(" 10")) ||
        (seq_inner.contains("'1'") && seq_inner.contains("'10'")) ||
        (seq_inner.contains("\"1\"") && seq_inner.contains("\"10\""));
        
    assert!(has_args, "Expected seq arguments '1' and '10' (quoted or unquoted), found: {}", seq_inner);
    
    // Ensure no commas (common regression if list syntax leaks)
    assert!(!seq_inner.contains(','), "Found comma in seq args, implies list syntax leakage: {}", seq_inner);
}
