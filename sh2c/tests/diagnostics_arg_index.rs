use common::{try_compile_path_to_shell, TargetShell, repo_root};
use std::fs;

mod common;

fn assert_compile_fail_matches_stderr(fixture_name: &str) {
    let sh2_path = repo_root().join(format!("sh2c/tests/fixtures/{}.sh2", fixture_name));
    let stderr_path = repo_root().join(format!("sh2c/tests/fixtures/{}.stderr", fixture_name));

    let res = try_compile_path_to_shell(&sh2_path, TargetShell::Bash);
    
    match res {
        Ok(_) => panic!("Expected compilation error for {}, but it succeeded", fixture_name),
        Err(actual_err) => {
            if std::env::var("SH2C_UPDATE_SNAPSHOTS").is_ok() {
               fs::write(&stderr_path, &actual_err).expect("Failed to write stderr snapshot");
            } else if !stderr_path.exists() {
                 panic!("Snapshot missing: {}. run with SH2C_UPDATE_SNAPSHOTS=1", stderr_path.display());
            }
            
            let expected_err = fs::read_to_string(&stderr_path).expect("Failed to read expected stderr");
            // Check if actual error contains the expected text (simplest check)
            // Or better, normalize and strict check. For now, contains is safer for partial matches.
            if !actual_err.contains(expected_err.trim()) {
                 panic!("Expected stderr to contain:\n{}\nGot:\n{}", expected_err, actual_err);
            }
        }
    }
}

#[test]
fn test_arg_index_string_literal_error() {
    assert_compile_fail_matches_stderr("arg_index_string_literal_error");
}

#[test]
fn test_arg_index_nested_arg_error() {
    assert_compile_fail_matches_stderr("arg_index_nested_arg_error");
}
