use std::process::Command;

fn sh2c_path() -> String {
    env!("CARGO_BIN_EXE_sh2c").to_string()
}

#[test]
fn test_cli_out_flag_after_script() {
    // This previously panicked due to lexer corruption when processing files with '-'
    // and potentially CLI parsing issues (though main.rs looked ok).
    
    // We need a script that triggers the lexer bug. The bug is in '-' handling.
    // So script must contain '-'.
    let script_path = "tests/fixtures/cli_repro_panic.sh2";
    let out_path = "tests/fixtures/cli_repro_panic.sh";
    
    std::fs::write(script_path, "run(\"echo\", \"-foo\")").unwrap();
    
    let output = Command::new(sh2c_path())
        .arg("--target")
        .arg("bash")
        .arg(script_path)
        .arg("-o")
        .arg(out_path)
        .output()
        .expect("Failed to run sh2c");
        
    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        panic!("sh2c failed with exit code {}", output.status);
    }
    
    // Cleanup
    let _ = std::fs::remove_file(script_path);
    let _ = std::fs::remove_file(out_path);
}

#[test]
fn test_lexer_panic_on_arrow() {
    // This triggers the specific "Need multiple hunks" panic in the corrupted lexer
    let script_path = "tests/fixtures/cli_repro_arrow.sh2";
    let out_path = "tests/fixtures/cli_repro_arrow.sh";
    
    // '->' was the trigger sequence in the corrupted lexer code
    std::fs::write(script_path, "->").unwrap();
    
    let output = Command::new(sh2c_path())
        .arg("--target")
        .arg("bash")
        .arg(script_path)
        .arg("-o")
        .arg(out_path)
        .output()
        .expect("Failed to run sh2c");
        
    // Should NOT panic (exit code 101). It might fail compilation (syntax error) or succeed if inside string.
    // Wait, lexer handles strings separately. so "->" inside string wouldn't trigger it.
    // It must be bare `->`.  
    // But bare `->` is invalid syntax anyway. It should return exit code 1 or 2 (checking/compile error), not 101 (panic).
    
    if output.status.code() == Some(101) {
        panic!("sh2c panicked (exit 101) on arrow syntax");
    }

     // Cleanup
    let _ = std::fs::remove_file(script_path);
    let _ = std::fs::remove_file(out_path);
}

#[test]
fn test_cli_missing_arg() {
    // sh2c ./a.sh2 -o  (missing value) -> should error cleanly
    let script_path = "tests/fixtures/cli_missing_arg.sh2";
    std::fs::write(script_path, "run(\"true\")").unwrap();

    let output = Command::new(sh2c_path())
        .arg(script_path)
        .arg("-o")
        .output()
        .expect("Failed to run sh2c");
        
    if output.status.success() {
        panic!("sh2c should have failed");
    }
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("panicked") {
        panic!("sh2c panicked instead of clean error");
    }
    assert!(stderr.contains("requires an argument")); // Matches main.rs message

    let _ = std::fs::remove_file(script_path);
}

#[test]
fn test_cli_out_flag_before_script() {
    let script_path = "tests/fixtures/cli_repro_panic_pre.sh2";
    let out_path = "tests/fixtures/cli_repro_panic_pre.sh";
    
    std::fs::write(script_path, "run(\"echo\", \"-foo\")").unwrap();
    
    let output = Command::new(sh2c_path())
        .arg("--target")
        .arg("bash")
        .arg("-o")
        .arg(out_path)
        .arg(script_path)
        .output()
        .expect("Failed to run sh2c");
        
    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("sh2c failed with exit code {}", output.status);
    }

    // Cleanup
    let _ = std::fs::remove_file(script_path);
    let _ = std::fs::remove_file(out_path);
}
