use std::process::Command;
use std::path::PathBuf;

fn sh2c_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sh2c"))
}

fn assert_diag_output(fixture: &str) {
    let fixture_path = PathBuf::from("tests/fixtures").join(fixture);
    let expected_path = fixture_path.with_extension("stderr.expected");
    
    let output = Command::new(sh2c_path())
        .arg(&fixture_path)
        .output()
        .expect("Failed to run sh2c");
        
    assert!(!output.status.success(), "sh2c should fail");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    let expected = std::fs::read_to_string(&expected_path).expect("Missing expected file");
    
    // Normalize newlines
    let stderr = stderr.replace("\r\n", "\n").trim().to_string();
    let expected = expected.replace("\r\n", "\n").trim().to_string();
    
    assert_eq!(stderr, expected, "Stderr mismatch for {}", fixture);
}

#[test]
fn diag_span_try_run_expr_invalid() {
    assert_diag_output("diag_span_try_run_expr_invalid.sh2");
}

#[test]
fn diag_span_bad_option_loc() {
    assert_diag_output("diag_span_bad_option_loc.sh2");
}

#[test]
fn diag_span_multiline_expr() {
    assert_diag_output("diag_span_multiline_expr.sh2");
}
