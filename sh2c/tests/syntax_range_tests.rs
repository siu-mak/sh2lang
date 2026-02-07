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
}
