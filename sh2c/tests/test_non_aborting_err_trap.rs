mod common;

use std::process::Command;
use tempfile::TempDir;

/// Compile a .sh2 fixture to bash and execute it, returning (stdout, stderr, exit_code)
fn compile_and_run_fixture(fixture_name: &str) -> (String, String, i32) {
    let repo_root = common::repo_root();
    let fixture_path = repo_root.join(format!("sh2c/tests/fixtures/{}.sh2", fixture_name));
    
    if !fixture_path.exists() {
        panic!("Fixture {} does not exist at {:?}", fixture_name, fixture_path);
    }
    
    // Create temp dir for output
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("script.sh");
    
    // Compile .sh2 -> .sh using sh2c binary
    let compile_result = Command::new("cargo")
        .args(&["run", "--quiet", "--bin", "sh2c", "--", 
                fixture_path.to_str().unwrap(), 
                "-o", output_path.to_str().unwrap()])
        .current_dir(&repo_root)
        .output()
        .expect("Failed to run cargo/sh2c");
    
    if !compile_result.status.success() {
        panic!(
            "Compilation failed for {}:\nstdout: {}\nstderr: {}",
            fixture_name,
            String::from_utf8_lossy(&compile_result.stdout),
            String::from_utf8_lossy(&compile_result.stderr)
        );
    }
    
    // Execute the compiled bash script
    let run_result = Command::new("bash")
        .arg(&output_path)
        .output()
        .expect("Failed to run bash");
    
    let stdout = String::from_utf8_lossy(&run_result.stdout).to_string();
    let stderr = String::from_utf8_lossy(&run_result.stderr).to_string();
    let exit_code = run_result.status.code().unwrap_or(-1);
    
    (stdout, stderr, exit_code)
}

#[test]
fn test_which_missing_noerr() {
    let (stdout, stderr, exit_code) = compile_and_run_fixture("which_missing_noerr");
    
    // Should succeed (exit 0)
    assert_eq!(exit_code, 0, "which() missing command should not abort. stderr: {}", stderr);
    
    // Should NOT print "Error in ..."
    assert!(
        !stderr.contains("Error in"),
        "which() missing command should not emit ERR trap message. stderr: {}",
        stderr
    );
    
    // Should print expected message
    assert!(
        stdout.contains("which correctly returned empty"),
        "Expected output not found. stdout: {}",
        stdout
    );
}

#[test]
fn test_wait_allow_fail_noerr() {
    let (stdout, stderr, exit_code) = compile_and_run_fixture("wait_allow_fail_noerr");
    
    // Should succeed (exit 0)
    assert_eq!(exit_code, 0, "wait(allow_fail=true) should not abort. stderr: {}", stderr);
    
    // Should NOT print "Error in ..."
    assert!(
        !stderr.contains("Error in"),
        "wait(allow_fail=true) should not emit ERR trap message. stderr: {}",
        stderr
    );
    
    // Should print expected message
    assert!(
        stdout.contains("wait correctly captured failure"),
        "Expected output not found. stdout: {}",
        stdout
    );
}

#[test]
fn test_wait_all_allow_fail_noerr() {
    let (stdout, stderr, exit_code) = compile_and_run_fixture("wait_all_allow_fail_noerr");
    
    // Should succeed (exit 0)
    assert_eq!(exit_code, 0, "wait_all(allow_fail=true) should not abort. stderr: {}", stderr);
    
    // Should NOT print "Error in ..."
    assert!(
        !stderr.contains("Error in"),
        "wait_all(allow_fail=true) should not emit ERR trap message. stderr: {}",
        stderr
    );
    
    // Should print expected message
    assert!(
        stdout.contains("wait_all correctly captured failure"),
        "Expected output not found. stdout: {}",
        stdout
    );
}

#[test]
fn test_wait_all_abort_still_errs() {
    let (stdout, stderr, exit_code) = compile_and_run_fixture("wait_all_abort_still_errs");
    
    // Should fail (non-zero exit)
    assert_ne!(exit_code, 0, "wait_all() without allow_fail should abort. stdout: {}", stdout);
    
    // SHOULD print "Error in ..."
    assert!(
        stderr.contains("Error in"),
        "wait_all() without allow_fail SHOULD emit ERR trap message. stderr: {}",
        stderr
    );
    
    // Should NOT print the message after wait_all
    assert!(
        !stdout.contains("should not reach here"),
        "Script should have aborted before final print. stdout: {}",
        stdout
    );
}
