//! Integration tests for wait_all(pids) job control builtin.
//!
//! These tests verify:
//! - wait_all([pid1, pid2]) returns 0 when all jobs succeed
//! - wait_all returns first non-zero exit code in list order
//! - allow_fail=true prevents abort on non-zero exit
//! - POSIX target coverage

use std::process::Command;

fn compile_fixture(name: &str, target: &str) -> Result<String, String> {
    let fixture_path = format!("tests/fixtures/{}.sh2", name);
    let output = Command::new("cargo")
        .args(["run", "--bin", "sh2c", "--", "--target", target, &fixture_path])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .map_err(|e| format!("Failed to run sh2c: {}", e))?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn run_script(script: &str, shell: &str) -> Result<(String, i32), String> {
    let output = Command::new(shell)
        .args(["-c", script])
        .output()
        .map_err(|e| format!("Failed to run shell: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let code = output.status.code().unwrap_or(-1);
    Ok((stdout, code))
}

#[test]
fn test_wait_all_all_succeed() {
    let script = compile_fixture("wait_all_ok", "bash").expect("compile failed");
    let (stdout, code) = run_script(&script, "bash").expect("run failed");
    
    assert!(stdout.contains("all succeeded, rc: 0"), "Expected rc 0, got: {}", stdout);
    assert_eq!(code, 0, "Script should exit 0 when all jobs succeed");
}

#[test]
fn test_wait_all_allow_fail() {
    let script = compile_fixture("wait_all_allow_fail", "bash").expect("compile failed");
    let (stdout, code) = run_script(&script, "bash").expect("run failed");
    
    assert!(stdout.contains("first fail code: 3"), "Expected first fail 3, got: {}", stdout);
    assert!(stdout.contains("status: 3"), "Expected status 3, got: {}", stdout);
    assert_eq!(code, 0, "Script should exit 0 with allow_fail=true");
}

#[test]
fn test_wait_all_preserves_order_first_failure() {
    // Jobs finish out of order, but wait_all should return first LIST failure
    let script = compile_fixture("wait_all_order", "bash").expect("compile failed");
    let (stdout, code) = run_script(&script, "bash").expect("run failed");
    
    // p1 (exit 5) is first in list, p2 (exit 7) finishes first chronologically
    // We expect 5 because it's first in list order
    assert!(stdout.contains("first list fail: 5"), "Expected first list fail 5, got: {}", stdout);
    assert_eq!(code, 0, "Script should exit 0 with allow_fail=true");
}

#[test]
fn test_wait_all_default_aborts() {
    // Without allow_fail, a failing job should cause abort
    let fixture = r#"
func main() {
    let p1 = spawn(run("sh", "-c", "exit 1"))
    let pids = [p1]
    let rc = wait_all(pids)
    print("should not reach here")
}
"#;
    let path = "/tmp/wait_all_abort.sh2";
    std::fs::write(path, fixture).unwrap();
    
    let output = Command::new("cargo")
        .args(["run", "--bin", "sh2c", "--", "-o", "/tmp/wait_all_abort.sh", path])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to compile");
    
    assert!(output.status.success(), "Compilation should succeed");
    
    let run_output = Command::new("bash")
        .arg("/tmp/wait_all_abort.sh")
        .output()
        .expect("Failed to run script");
    
    assert!(!run_output.status.success(), "Script should fail due to wait_all abort");
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(!stdout.contains("should not reach here"), "Script should abort before print");
}

#[test]
fn test_wait_all_posix_target() {
    // Use a fixture with inline list (no array variable assignment)
    let script = compile_fixture("wait_all_posix", "posix");
    assert!(script.is_ok(), "wait_all should compile for POSIX target: {:?}", script);
    
    if let Ok(compiled) = script {
        if Command::new("dash").arg("-c").arg("exit 0").status().is_ok() {
            let (stdout, code) = run_script(&compiled, "dash").expect("run failed");
            assert!(stdout.contains("all succeeded"), "Expected success, got: {}", stdout);
            assert_eq!(code, 0);
        }
    }
}

#[test]
fn test_wait_all_posix_rejects_list_variable() {
    // POSIX target should reject wait_all with list variable (only literals supported)
    let result = compile_fixture("wait_all_posix_var_rejected", "posix");
    assert!(result.is_err(), "wait_all with list variable should fail on POSIX");
    let err = result.unwrap_err();
    assert!(
        err.contains("list literal") || err.contains("not supported") || err.contains("Array assignment"),
        "Error should mention list literal requirement: {}", err
    );
}
