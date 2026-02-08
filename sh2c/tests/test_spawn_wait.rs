//! Integration tests for spawn/wait job control builtins.
//!
//! These tests verify:
//! - spawn(run(...)) starts background job and returns PID
//! - wait(pid) returns exit code and sets status()
//! - allow_fail=true prevents abort on non-zero exit
//! - Context restrictions are enforced

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
fn test_spawn_wait_basic() {
    let script = compile_fixture("spawn_wait_basic", "bash").expect("compile failed");
    let (stdout, code) = run_script(&script, "bash").expect("run failed");
    
    assert!(stdout.contains("exit code: 7"), "Expected exit code 7, got: {}", stdout);
    assert!(stdout.contains("status: 7"), "Expected status 7, got: {}", stdout);
    assert_eq!(code, 0, "Script should exit 0 with allow_fail=true");
}

#[test]
fn test_spawn_wait_multi_reverse_order() {
    let script = compile_fixture("spawn_wait_multi", "bash").expect("compile failed");
    let (stdout, code) = run_script(&script, "bash").expect("run failed");
    
    assert!(stdout.contains("job B exit: 9"), "Expected job B exit 9, got: {}", stdout);
    assert!(stdout.contains("job A exit: 3"), "Expected job A exit 3, got: {}", stdout);
    assert_eq!(code, 0, "Script should exit 0 with allow_fail=true");
}

#[test]
fn test_spawn_wait_default_allow_fail() {
    let script = compile_fixture("spawn_wait_default", "bash").expect("compile failed");
    let (stdout, code) = run_script(&script, "bash").expect("run failed");
    
    assert!(stdout.contains("exit code: 0"), "Expected exit code 0, got: {}", stdout);
    assert_eq!(code, 0);
}

#[test]
fn test_spawn_invalid_arg_rejected() {
    // spawn("ls") should be rejected - only run() or sudo() allowed
    let fixture = r#"
func main() {
    let job_pid = spawn("ls")
}
"#;
    std::fs::write("/tmp/spawn_invalid.sh2", fixture).unwrap();
    let output = Command::new("cargo")
        .args(["run", "--bin", "sh2c", "--", "-o", "-", "/tmp/spawn_invalid.sh2"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run sh2c");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "Should fail compilation");
    assert!(stderr.contains("spawn() only accepts run(") || stderr.contains("spawn() requires exactly 1 argument"),
            "Expected clear error about spawn args, got: {}", stderr);
}

#[test]
fn test_spawn_in_invalid_context_rejected() {
    // print(spawn(run("ls"))) should be rejected
    let fixture = r#"
func main() {
    print(spawn(run("ls")))
}
"#;
    std::fs::write("/tmp/spawn_context.sh2", fixture).unwrap();
    let output = Command::new("cargo")
        .args(["run", "--bin", "sh2c", "--", "-o", "-", "/tmp/spawn_context.sh2"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run sh2c");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "Should fail compilation");
    assert!(stderr.contains("spawn()") && stderr.contains("let"),
            "Expected error about spawn context, got: {}", stderr);
}

#[test]
fn test_wait_in_invalid_context_rejected() {
    // run("echo", wait(job_pid)) should be rejected
    let fixture = r#"
func main() {
    let job_pid = spawn(run("true"))
    run("echo", wait(job_pid))
}
"#;
    std::fs::write("/tmp/wait_context.sh2", fixture).unwrap();
    let output = Command::new("cargo")
        .args(["run", "--bin", "sh2c", "--", "-o", "-", "/tmp/wait_context.sh2"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run sh2c");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "Should fail compilation");
    assert!(stderr.contains("wait()") && stderr.contains("let"),
            "Expected error about wait context, got: {}", stderr);
}

#[test]
fn test_spawn_wait_posix_target() {
    // spawn/wait should work on POSIX target (uses standard &, $!, wait)
    let script = compile_fixture("spawn_wait_default", "posix");
    assert!(script.is_ok(), "spawn/wait should compile for POSIX target: {:?}", script);
    
    // Run with dash (POSIX shell)
    if let Ok(compiled) = script {
        if Command::new("dash").arg("-c").arg("exit 0").status().is_ok() {
            let (stdout, code) = run_script(&compiled, "dash").expect("run failed");
            assert!(stdout.contains("exit code: 0"), "Expected exit code 0, got: {}", stdout);
            assert_eq!(code, 0);
        }
    }
}

#[test]
fn test_spawn_allow_fail_rejected() {
    let fixture_path = "tests/fixtures/spawn_allow_fail_rejected.sh2";
    let output = Command::new("cargo")
        .args(["run", "--bin", "sh2c", "--", "-o", "-", fixture_path])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run sh2c");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "Should fail compilation");
    assert!(stderr.contains("allow_fail=true is not supported in spawn") || 
            stderr.contains("wait(pid, allow_fail=true)") ||
            stderr.contains("run() options are only allowed inside capture"),
            "Expected error about allow_fail in spawn, got: {}", stderr);
}

#[test]
fn test_wait_error_location() {
    // wait(pid) failure should point to the wait line, not previous lines
    let fixture = r#"
func main() {
    let job = spawn(run("sh", "-c", "exit 1"))
    
    // Line 5: Wait statement that should fail
    let rc = wait(job)
}
"#;
    let path = "/tmp/wait_error_loc.sh2";
    std::fs::write(path, fixture).unwrap();
    
    // Compile and run the output script
    // Note: compilation should succeed, runtime execution should fail and print diagnostic
    // We use a temporary output file
    let path_out = "/tmp/wait_error_loc_test.sh";
    let output = Command::new("cargo")
        .args(["run", "--bin", "sh2c", "--", "-o", path_out, path])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to compile sh2c");
    
    assert!(output.status.success(), "Compilation should succeed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Run the script
    let run_output = Command::new("bash")
        .arg(path_out)
        .output()
        .expect("Failed to run script");
        
    assert!(!run_output.status.success(), "Script should fail due to wait()");
    let stderr = String::from_utf8_lossy(&run_output.stderr);
    
    // Check for location
    // The format is typically: "Error: ... at /tmp/wait_error_loc.sh2:5"
    // We expect line 6 (wait call), because sh2c uses 1-based indexing and line 5 is comment
    // Actually the wait(job) is on line 6:
    // 1: func
    // 2:   let job
    // 3:   
    // 4:   // comment
    // 5:   let rc = wait(job) 
    // Wait, let's count:
    // 1: func main() {
    // 2:     let job = spawn(...)
    // 3:     
    // 4:     // Line 5: ...
    // 5:     let rc = wait(job)
    // 6: }
    // So line 5.
    
    assert!(stderr.contains("wait_error_loc.sh2:6"), 
            "Expected error location at line 6 (wait call), got stderr:\n{}", stderr);
}
