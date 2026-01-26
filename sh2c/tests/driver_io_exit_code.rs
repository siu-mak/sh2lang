use std::process::Command;

fn sh2c_path() -> String {
    env!("CARGO_BIN_EXE_sh2c").to_string()
}

#[test]
fn missing_file_exits_with_code_1() {
    let output = Command::new(sh2c_path())
        .arg("non_existent_file.sh2")
        .output()
        .expect("Failed to run sh2c");
        
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1), "Expected exit code 1 for IO error, got {:?}", output.status.code());
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("File not found"), "Stderr: {}", stderr);
}

#[test]
#[cfg(unix)]
fn unreadable_file_exits_with_code_1() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;
    
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("secret.sh2");
    fs::write(&file, "func main(){}").unwrap();
    
    let mut perms = fs::metadata(&file).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&file, perms).unwrap();
    
    let output = Command::new(sh2c_path())
        .arg(&file)
        .output()
        .expect("Failed to run sh2c");
        
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1), "Expected exit code 1 for IO error");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unable to read file"));
}
