use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

mod common;
use common::{compile_path_to_shell, TargetShell};

/// Helper to run a compiled bash script in a temp directory and capture output
fn run_bash_script_in_tempdir(script_content: &str) -> (String, String, i32, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("test_script.sh");
    
    fs::write(&script_path, script_content).expect("Failed to write script");
    
    let output = Command::new("bash")
        .arg(&script_path)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute script");
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    
    (stdout, stderr, exit_code, temp_dir)
}

/// Helper to read file content from temp dir
fn read_file_in_dir(dir: &Path, filename: &str) -> Option<String> {
    let path = dir.join(filename);
    fs::read_to_string(path).ok()
}

#[test]
fn test_stdout_file_and_inherit() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/redirect_multi_stdout_file_and_inherit.sh2");
    
    let bash_script = compile_path_to_shell(&fixture_path, TargetShell::Bash);
    let (stdout, _stderr, exit_code, temp_dir) = run_bash_script_in_tempdir(&bash_script);
    
    // Assert exit code
    assert_eq!(exit_code, 0, "Script should exit successfully");
    
    // Assert stdout contains the line (inherit_stdout)
    assert!(stdout.contains("fan-out test line"), 
            "stdout should contain the line due to inherit_stdout, got: {}", stdout);
    
    // Assert file contains the line
    let file_content = read_file_in_dir(temp_dir.path(), "out.log")
        .expect("out.log should exist");
    assert_eq!(file_content.trim(), "fan-out test line",
               "out.log should contain exactly the test line");
}

#[test]
fn test_stdout_two_files_no_inherit() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/redirect_multi_stdout_two_files_no_inherit.sh2");
    
    let bash_script = compile_path_to_shell(&fixture_path, TargetShell::Bash);
    let (stdout, _stderr, exit_code, temp_dir) = run_bash_script_in_tempdir(&bash_script);
    
    assert_eq!(exit_code, 0, "Script should exit successfully");
    
    // Assert stdout does NOT contain the line (no inherit)
    assert!(!stdout.contains("two-file test line"), 
            "stdout should NOT contain the line (no inherit_stdout), got: {}", stdout);
    
    // Assert both files contain the line
    let file_a_content = read_file_in_dir(temp_dir.path(), "out_a.log")
        .expect("out_a.log should exist");
    let file_b_content = read_file_in_dir(temp_dir.path(), "out_b.log")
        .expect("out_b.log should exist");
    
    assert_eq!(file_a_content.trim(), "two-file test line");
    assert_eq!(file_b_content.trim(), "two-file test line");
}

#[test]
fn test_stderr_file_and_inherit() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/redirect_multi_stderr_file_and_inherit.sh2");
    
    let bash_script = compile_path_to_shell(&fixture_path, TargetShell::Bash);
    let (_stdout, stderr, exit_code, temp_dir) = run_bash_script_in_tempdir(&bash_script);
    
    assert_eq!(exit_code, 0, "Script should exit successfully");
    
    // Assert stderr contains the line (inherit_stderr)
    assert!(stderr.contains("stderr test line"), 
            "stderr should contain the line due to inherit_stderr, got: {}", stderr);
    
    // Assert file contains the line
    let file_content = read_file_in_dir(temp_dir.path(), "err.log")
        .expect("err.log should exist");
    assert_eq!(file_content.trim(), "stderr test line");
}

#[test]
fn test_inherit_only_no_tee() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/redirect_multi_inherit_only.sh2");
    
    let bash_script = compile_path_to_shell(&fixture_path, TargetShell::Bash);
    let (stdout, _stderr, exit_code, temp_dir) = run_bash_script_in_tempdir(&bash_script);
    
    assert_eq!(exit_code, 0, "Script should exit successfully");
    
    // Assert stdout contains the line
    assert!(stdout.contains("inherit-only line"), 
            "stdout should contain the line, got: {}", stdout);
    
    // Assert no files created (inherit-only should not spawn tee)
    let files: Vec<_> = fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.file_name() != "test_script.sh")
        .collect();
    
    assert!(files.is_empty(), 
            "No output files should be created for inherit-only redirect, found: {:?}", 
            files.iter().map(|f| f.path()).collect::<Vec<_>>());
    
    // Behavior assertion: stdout has content, no files.
    // We removed the brittle check for "tee" substring in the bash script.
}

#[test]
fn test_status_cmd_fail() {
    // Test that command failure exit code is propagated through redirect
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/redirect_multi_status_cmd_fail.sh2");
    
    let bash_script = compile_path_to_shell(&fixture_path, TargetShell::Bash);
    let (_stdout, _stderr, exit_code, _temp_dir) = run_bash_script_in_tempdir(&bash_script);
    
    assert_eq!(exit_code, 7, "Command exit code 7 should be propagated");
}

#[test]
fn test_nested_redirects() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/redirect_multi_nested.sh2");
    
    let bash_script = compile_path_to_shell(&fixture_path, TargetShell::Bash);
    let (stdout, stderr, exit_code, temp_dir) = run_bash_script_in_tempdir(&bash_script);
    
    assert_eq!(exit_code, 0, "Script should exit successfully (no variable collision)");
    
    // Assert stdout contains outer lines (inner has no inherit, so not visible)
    assert!(stdout.contains("outer line"), "stdout should contain outer line");
    assert!(!stdout.contains("inner line"), 
            "stdout should NOT contain inner line (no inherit_stdout on inner block)");
    assert!(stdout.contains("outer again"), "stdout should contain outer again");
    
    // Assert outer.log contains outer lines only
    let outer_content = read_file_in_dir(temp_dir.path(), "outer.log")
        .expect("outer.log should exist");
    assert!(outer_content.contains("outer line"), "outer.log should contain 'outer line'");
    assert!(outer_content.contains("outer again"), "outer.log should contain 'outer again'");
    assert!(!outer_content.contains("inner line"), 
            "outer.log should NOT contain 'inner line', got: {}", outer_content);
    
    // Assert inner.log contains inner line only
    let inner_content = read_file_in_dir(temp_dir.path(), "inner.log")
        .expect("inner.log should exist");
    assert_eq!(inner_content.trim(), "inner line", 
               "inner.log should contain only 'inner line', got: {}", inner_content);
}

#[test]
fn test_heredoc_in_redirect() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/redirect_multi_heredoc.sh2");
    
    let bash_script = compile_path_to_shell(&fixture_path, TargetShell::Bash);
    let (stdout, stderr, exit_code, temp_dir) = run_bash_script_in_tempdir(&bash_script);
    
    assert_eq!(exit_code, 0, "Script should exit successfully");
    
    let expected_lines = vec!["Line 1", "Line 2", "Line 3"];
    
    // Assert stdout contains all lines
    for line in &expected_lines {
        assert!(stdout.contains(line), "stdout should contain '{}', got: {}", line, stdout);
    }
    
    // Assert file contains all lines
    let file_content = read_file_in_dir(temp_dir.path(), "heredoc_out.log")
        .expect("heredoc_out.log should exist");
    for line in &expected_lines {
        assert!(file_content.contains(line), 
                "heredoc_out.log should contain '{}', got: {}", line, file_content);
    }
}
