use std::path::Path;

mod common;
use common::{compile_path_to_shell, run_bash_script_in_tempdir, TargetShell};

#[test]
fn test_runtime_tilde_hint_cwd() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/hint_no_tilde_expand_with_cwd.sh2");
    
    let bash_script = compile_path_to_shell(&fixture_path, TargetShell::Bash);
    let (_stdout, stderr, exit_code, _temp_dir) = run_bash_script_in_tempdir(&bash_script);
    
    assert_ne!(exit_code, 0, "Script should fail due to missing directory");
    
    // Check for the hint
    assert!(stderr.contains("hint: '~' is not expanded"), 
            "stderr should contain expansion hint, got: {}", stderr);
    
    // Check for canonical advice
    assert!(stderr.contains("use env.HOME"), 
            "stderr should advise using env.HOME, got: {}", stderr);
}

#[test]
fn test_runtime_tilde_user_hint_cwd() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/hint_no_tilde_expand_with_cwd_user.sh2");
    
    let bash_script = compile_path_to_shell(&fixture_path, TargetShell::Bash);
    let (_stdout, stderr, exit_code, _temp_dir) = run_bash_script_in_tempdir(&bash_script);
    
    assert_ne!(exit_code, 0, "Script should fail due to missing directory");
    
    assert!(stderr.contains("hint: '~' is not expanded"), 
            "stderr should contain expansion hint for ~user, got: {}", stderr);
}

#[test]
fn test_no_hint_for_expr() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/hint_no_tilde_expand_with_cwd_non_literal.sh2");
    
    // Should fail compilation (cwd requires literal)
    let result = common::try_compile_path_to_shell(&fixture_path, TargetShell::Bash);
    assert!(result.is_err(), "Compilation should fail for non-literal cwd path");
    
    let err = result.err().unwrap();
    assert!(err.contains("string literal"), "Error should mention string literal requirement, got: {}", err);
    // Ensure we do NOT see the runtime hint in the compile error
    assert!(!err.contains("hint: '~' is not expanded"), "Compile error should NOT contain runtime tilde expansion hint");
}
