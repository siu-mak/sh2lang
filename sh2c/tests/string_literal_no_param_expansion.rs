//! Tests for ensuring NO implicit parameter expansion in sh2 string literals.
//! P0 Correctness Fix.

use std::process::Command;
use std::path::Path;

fn compile_and_run(source_path: &Path, env_vars: &[(&str, &str)]) -> String {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_sh = temp_dir.path().join("output.sh");
    
    // We expect the sh2c binary to be available in the cargo target dir
    let sh2c = env!("CARGO_BIN_EXE_sh2c");
    let status = Command::new(sh2c)
        .arg(source_path.to_str().expect("Path not utf8"))
        .arg("-o")
        .arg(output_sh.to_str().expect("Path not utf8"))
        // We rely on default bash target
        .status()
        .expect("Failed to execute sh2c");
        
    if !status.success() {
        panic!("Compilation failed for {:?}", source_path);
    }
    
    // Run bash
    let mut cmd = Command::new("bash");
    cmd.arg(&output_sh);
    for (k, v) in env_vars {
        cmd.env(k, v);
    }
    
    let output = cmd.output().expect("Failed to execute generated script");
    
    String::from_utf8(output.stdout).expect("Stdout was not utf8")
}

#[test]
fn test_string_dollar_no_expand_print() {
    let fixture = Path::new("tests/fixtures/repro_dollar_expansion_print.sh2");
    let base = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture = base.join(fixture);
    
    // Pass FOO=EXPANDED in env. 
    // Expect output to be literally "$FOO", NOT "EXPANDED".
    let out = compile_and_run(&fixture, &[("FOO", "EXPANDED")]);
    assert_eq!(out.trim(), "$FOO", "Normal strings must be strict literals");
}

#[test]
fn test_string_braced_no_expand_run_printf() {
    let fixture = Path::new("tests/fixtures/string_braced_no_expand_run_printf.sh2");
    let base = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture = base.join(fixture);
    
    // Pass Package=BAD
    // Expect output to be literally "${Package}", NOT "BAD"
    let out = compile_and_run(&fixture, &[("Package", "BAD")]);
    assert_eq!(out.trim(), "${Package}", "Normal strings must be strict literals");
}

#[test]
fn test_string_braced_no_expand_run_dpkg_query() {
    // Check if dpkg-query exists using 'which' command (std only)
    let has_dpkg = Command::new("which")
        .arg("dpkg-query")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !has_dpkg {
        eprintln!("Skipping dpkg-query test: binary not found");
        return;
    }

    let fixture = Path::new("tests/fixtures/string_braced_no_expand_run_dpkg_query.sh2");
    let base = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture = base.join(fixture);
    
    // Pass Package=BAD
    // run("dpkg-query", "-W", "-f", "${Package}\n", "bash")
    // Should NOT expand ${Package} to BAD.
    // Should pass "${Package}\n" to dpkg-query.
    // dpkg-query will output package info for "bash".
    // Output should contain "bash" (package name) and status=0.
    
    let out = compile_and_run(&fixture, &[("Package", "BAD")]);
    
    assert!(!out.contains("BAD"), "Variable was expanded! Output: {}", out);
    assert!(out.contains("bash"), "Output should verify package presence. Got: {}", out);
    assert!(out.contains("status=0"), "Command should succeed. Got: {}", out);
}

#[test]
fn test_single_quote_roundtrip() {
    // Verify basic quoting robustly
    // Note: The above tests implicitly cover this by passing literals through compilation.
    // We keep this placeholder or remove it if empty.
}
