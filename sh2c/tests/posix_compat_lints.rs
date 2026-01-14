use std::fs;
use sh2c::codegen::posix_lint::{lint_script, PosixLintKind};
use assert_cmd::Command;


#[test]
fn test_lint_detects_bashisms() {
    let script = fs::read_to_string("tests/fixtures/posix_lint_bad.sh")
        .expect("Failed to read posix_lint_bad.sh");
    
    let lints = lint_script(&script);
    
    // Should detect at least 3 distinct bashisms
    assert!(lints.len() >= 3, "Expected at least 3 lints, got {}", lints.len());
    
    // Check for specific bashism types
    assert!(
        lints.iter().any(|l| matches!(l.kind, PosixLintKind::DoubleBracketTest)),
        "Should detect double-bracket test"
    );
    assert!(
        lints.iter().any(|l| matches!(l.kind, PosixLintKind::LocalOrDeclare)),
        "Should detect local/declare"
    );
    assert!(
        lints.iter().any(|l| matches!(l.kind, PosixLintKind::ProcessSubstitution)),
        "Should detect process substitution"
    );
}

#[test]
fn test_lint_accepts_clean_posix() {
    let script = fs::read_to_string("tests/fixtures/posix_lint_ok.sh")
        .expect("Failed to read posix_lint_ok.sh");
    
    let lints = lint_script(&script);
    
    assert!(
        lints.is_empty(),
        "POSIX-compatible script should not trigger lints, but got: {:?}",
        lints
    );
}

#[test]
fn test_cli_posix_check_valid_passes() {
    // Test that --target posix --check succeeds for valid POSIX scripts
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2c"));
    
    cmd.arg("--target")
        .arg("posix")
        .arg("--check")
        .arg("tests/fixtures/cli_check_ok.sh2")
        .assert()
        .success()
        .stdout("OK\n");
}
