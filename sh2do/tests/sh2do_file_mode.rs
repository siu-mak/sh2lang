use std::process::Command;
use std::fs;
use tempfile::TempDir;

fn sh2do_path() -> String {
    env!("CARGO_BIN_EXE_sh2do").to_string()
}

#[test]
fn test_file_mode_basic() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("hello.sh2");
    
    // File mode compiles the file as-is. Users must provide valid top-level code or main func.
    fs::write(&file, r#"
        func main() {
            print("hello file mode")
        }
    "#).unwrap();

    let output = Command::new(sh2do_path())
        .arg(&file)
        .output()
        .expect("Failed to run sh2do");
        
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(stdout.contains("hello file mode"));
    
    // Also test without extension existence check (if supported by logic)
    // The logic: ends_with(".sh2") OR exists().
    // So `hello.sh2` works.
}

#[test]
fn test_file_mode_compile_error() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("broken.sh2");
    fs::write(&file, "func main() { syntax error }").unwrap();
    
    let output = Command::new(sh2do_path())
        .arg(file)
        .output()
        .expect("Failed to run sh2do");
        
    assert!(!output.status.success());
}

#[test]
fn test_emit_mode() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("test_emit.sh2");
    fs::write(&file, r#"func main() { print("emitted") }"#).unwrap();
    
    let output = Command::new(sh2do_path())
        .arg(&file)
        .arg("--emit")
        .output()
        .expect("Failed to run sh2do");
        
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("emitted"));
    
    // Check file existence
    let expected_sh = tmp.path().join("test_emit.sh");
    assert!(expected_sh.exists(), "Emit should create .sh file");
}

#[test]
fn test_output_flag() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("test_out.sh2");
    fs::write(&file, r#"func main() { print("out flag") }"#).unwrap();
    
    let out_file = tmp.path().join("custom.sh");
    
    let output = Command::new(sh2do_path())
        .arg(&file)
        .arg("-o")
        .arg(&out_file)
        .output()
        .expect("Failed to run sh2do");
        
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("out flag"));
    assert!(out_file.exists());
}

#[test]
fn test_flags_after_file() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("flags_after.sh2");
    fs::write(&file, r#"func main() { print("flags after") }"#).unwrap();
    
    // sh2do file.sh2 --target posix
    let output = Command::new(sh2do_path())
        .arg(&file)
        .arg("--target")
        .arg("posix")
        .output()
        .expect("Failed to run sh2do");
        
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("flags after"));
}

#[test]
fn test_passthrough_safety() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("safety.sh2");
    // Print first arg to verify it was received.
    // arg(1) should be -n. "--" is consumed by shell.
    fs::write(&file, r#"func main() { print(arg(1)) }"#).unwrap();
    
    // sh2do safety.sh2 -- -n
    let output = Command::new(sh2do_path())
        .arg(&file)
        .arg("--")
        .arg("-n")
        .output()
        .expect("Failed to run sh2do");
        
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "-n");
}

#[test]
#[cfg(unix)]
fn test_unreadable_file() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("secret.sh2");
    fs::write(&file, "func main(){}").unwrap();
    
    // chmod 000
    let mut perms = fs::metadata(&file).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&file, perms).unwrap();
    
    let output = Command::new(sh2do_path())
        .arg(&file)
        .output()
        .expect("Failed to run sh2do");
        
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unable to read file"));
    assert!(stderr.contains("secret.sh2"));
    assert!(stderr.contains("Try inline usage"));
}

#[test]
fn target_bash_shell_sh_is_error() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("dummy.sh2");
    fs::write(&file, "func main(){}").unwrap();
    
    let output = Command::new(sh2do_path())
        .arg(&file)
        .arg("--target")
        .arg("bash")
        .arg("--shell")
        .arg("sh")
        .output()
        .expect("Failed to run sh2do");
        
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("bash target requires bash runtime"));
}

#[test]
fn test_inline_mode_preserved() {
    let output = Command::new(sh2do_path())
        .arg(r#"print("inline")"#)
        .output()
        .unwrap();
        
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "inline");
}

#[test]
fn test_file_not_found() {
    let output = Command::new(sh2do_path())
        .arg("non_existent.sh2")
        .output()
        .unwrap();
        
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("File not found: non_existent.sh2"));
}

#[test]
fn test_passthrough_double_dash_present() {
    // This test verifies that sh2do inserts `--` before passthrough args.
    // It creates a fake `bash` that prints its arguments and then executes real bash.
    
    use std::env;
    use std::fs;
    use tempfile::TempDir;
    
    let tmp = TempDir::new().unwrap();
    let wrapper_dir = tmp.path().join("bin");
    fs::create_dir(&wrapper_dir).unwrap();
    
    // Find real bash
    let real_bash = String::from_utf8(
        Command::new("bash").arg("-c").arg("command -v bash").output().unwrap().stdout
    ).unwrap().trim().to_string();
    
    let wrapper_script = wrapper_dir.join("bash");
    // Wrapper script: print all args, then exec real bash
    let script_content = format!(r#"#!/bin/sh
for arg in "$@"; do
    echo "wrapper_saw: $arg"
done
exec {} "$@"
"#, real_bash);

    fs::write(&wrapper_script, script_content).unwrap();
    
    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&wrapper_script).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&wrapper_script, perms).unwrap();
    }
    
    let file = tmp.path().join("test_wrapper.sh2");
    fs::write(&file, "func main(){ print(arg(1)) }").unwrap();
    
    // Add wrapper dir to PATH
    let original_path = env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", wrapper_dir.display(), original_path);
    
    let output = Command::new(sh2do_path())
        .env("PATH", new_path)
        .arg(&file)
        .arg("--shell")
        .arg("bash") // Force use of our wrapper "bash" (which sh2do invokes as 'bash')
        .arg("--")
        .arg("-n")
        .output()
        .expect("Failed to run sh2do");
        
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check usage of -- in the wrapper output
    // Invocation should be: bash -- <script> -n
    // So wrapper args should be: "--", "<script>", "-n"
    
    let lines: Vec<&str> = stdout.lines().filter(|l| l.starts_with("wrapper_saw:")).collect();
    
    let double_dash_idx = lines.iter().position(|l| *l == "wrapper_saw: --");
    // The script is passed as the first argument. Temp files might not have .sh extension.
    let script_idx = lines.iter().position(|l| l.starts_with("wrapper_saw: /"));
    let n_flag_idx = lines.iter().position(|l| *l == "wrapper_saw: -n");
    
    assert!(script_idx.is_some(), "Did not find script passed to shell (args: {:?})", lines);
    assert!(double_dash_idx.is_some(), "Did not find '--' passed to shell");
    assert!(n_flag_idx.is_some(), "Did not find '-n' passed to shell");
    
    // Check order: -- <script> -n
    let sc = script_idx.unwrap();
    let dd = double_dash_idx.unwrap();
    let nf = n_flag_idx.unwrap();
    
    assert!(dd < sc, "-- must precede script (found -- at {}, script at {})", dd, sc);
    assert!(sc < nf, "script must precede -n (found script at {}, -n at {})", sc, nf);
}

#[test]
fn test_script_path_starting_with_dash() {
    // Regression test for safety:
    // If we rely on `bash <script>` and the script is named `-bad.sh`, bash treats it as a flag.
    // sh2do MUST use `bash -- <script>` to prevent this.
    
    let tmp = TempDir::new().unwrap();
    // Force a filename starting with -
    let file = tmp.path().join("-badname.sh2");
    fs::write(&file, r#"func main() { print("safe") }"#).unwrap();
    
    // sh2do -badname.sh2
    let output = Command::new(sh2do_path())
        .arg(&file)
        .output()
        .expect("Failed to run sh2do");
        
    assert!(output.status.success(), "Failed to run script starting with dash. stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).trim().contains("safe"));
}

#[test]
fn flags_after_positional_are_accepted() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("mixed_flags.sh2");
    fs::write(&file, r#"func main() { print("mixed") }"#).unwrap();
    
    // sh2do --target posix mixed_flags.sh2 --shell sh
    let output = Command::new(sh2do_path())
        .arg("--target")
        .arg("posix")
        .arg(&file)
        .arg("--shell")
        .arg("sh")
        .output()
        .expect("Failed to run sh2do");
        
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("mixed"));
}

#[test]
fn emit_is_rejected_in_inline_mode() {
    let output = Command::new(sh2do_path())
        .arg(r#"print("inline")"#)
        .arg("--emit")
        .output()
        .expect("Failed to run sh2do");
        
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--emit is only valid when running a file"));
}

#[test]
fn directory_argument_is_rejected() {
    let tmp = TempDir::new().unwrap();
    // Use the temp dir itself as the argument
    let dir = tmp.path();
    
    let output = Command::new(sh2do_path())
        .arg(dir)
        .output()
        .expect("Failed to run sh2do");
        
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Path is a directory"));
}

#[test]
fn invalid_shell_value_is_error() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("ok.sh2");
    fs::write(&file, "func main(){}").unwrap();
    
    let output = Command::new(sh2do_path())
        .arg(&file)
        .arg("--shell")
        .arg("zsh")
        .output()
        .expect("Failed to run sh2do");
        
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid shell: 'zsh'"));
}

#[test]
fn compile_error_exit_code_is_2() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("broken.sh2");
    fs::write(&file, "func main() { syntax error }").unwrap();
    
    let output = Command::new(sh2do_path())
        .arg(&file)
        .output()
        .expect("Failed to run sh2do");
        
    // Compile error should be 2
    assert_eq!(output.status.code(), Some(2));
}
