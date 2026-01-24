mod common;
use std::fs;
use std::path::Path;

#[test]
fn test_run_exec_cwd_implicit() {
    // Scenario: Correct directory, no with cwd
    // func main() { run("mv", "a.txt", "b.txt") }
    // Executed in a dir containing a.txt.
    
    // Create a temp dir for the *compilation source*
    let source_dir = tempfile::tempdir().unwrap();
    let script_path = source_dir.path().join("test.sh2");
    
    fs::write(&script_path, r#"
        func main() {
            run("mv", "a.txt", "b.txt")
        }
    "#).unwrap();
    
    // Create 'a.txt' in a separate runtime dir that we control
    let runtime_dir = tempfile::tempdir().unwrap();
    let a_path = runtime_dir.path().join("a.txt");
    let b_path = runtime_dir.path().join("b.txt");
    fs::write(&a_path, "content").unwrap();

    let shell = common::compile_path_to_shell(&script_path, common::TargetShell::Bash);

    // Run the shell script with CWD = runtime_dir
    // We can't use common::run_shell_script easily if we want to control CWD and keep it.
    // So we use std::process::Command directly here or modify common to accept existing dir?
    // modifying common::run_shell_script is intrusive.
    // Let's just run Command here.
    
    let status = std::process::Command::new("bash")
        .arg("-c")
        .arg(&shell)
        .current_dir(runtime_dir.path())
        .status()
        .expect("Failed to execute bash");
        
    assert!(status.success(), "Script failed");
    
    assert!(!a_path.exists(), "a.txt should be gone");
    assert!(b_path.exists(), "b.txt should exist");
}

#[test]
fn test_run_with_cwd() {
    // Scenario: Literal cwd + relative args
    // func main() { with cwd("<TEMP_DIR>") { run("mv", "a.txt", "b.txt") } }
    
    let work_dir = tempfile::tempdir().unwrap();
    let work_path = work_dir.path().to_string_lossy(); // Absolute path
    
    let a_path = work_dir.path().join("a.txt");
    let b_path = work_dir.path().join("b.txt");
    fs::write(&a_path, "content").unwrap();

    let script_dir = tempfile::tempdir().unwrap();
    let script_path = script_dir.path().join("test_cwd.sh2");
    
    // We inject the absolute path into the script logic.
    // The bug report says this triggers loader resolution.
    // By fixing loader to NOT resolve this, we allow this test to pass.
    let src = format!(r#"
        func main() {{
            with cwd("{}") {{
                run("mv", "a.txt", "b.txt")
            }}
        }}
    "#, work_path);
    
    fs::write(&script_path, src).unwrap();

    let shell = common::compile_path_to_shell(&script_path, common::TargetShell::Bash);
    
    // Run content. We don't care about CWD here because script sets it.
    let status = std::process::Command::new("bash")
        .arg("-c")
        .arg(&shell)
        .status()
        .expect("Failed to execute bash");

    assert!(status.success(), "Script failed");
    
    assert!(!a_path.exists(), "a.txt should be gone in work_dir");
    assert!(b_path.exists(), "b.txt should exist in work_dir");
}

#[test]
fn test_run_absolute_exec_portable() {
    // Scenario: Absolute executable logic check
    // "run(path, args)" should not trigger loader.
    // Instead of verifying /bin/mv, we use "sh" which is standard, 
    // but users might not put sh in absolute path.
    // BUT, the ticket asks to verify checking absolute executable paths.
    // "A command invoked via a known absolute path is NOT required; portability > absolute exec."
    // "Implement it by invoking sh ... using sh -c logic"
    
    // We want to test `run("/path/to/something")`.
    // Let's use `run("/bin/sh")` if it exists, or skip?
    // Most linux/unix has /bin/sh.
    // But nixos might not?
    // sh2lang runtime assumes `sh` is in path for some things anyway?
    // Let's try to detect `sh` path via `which` or just check /bin/sh.
    
    let sh_path = if Path::new("/bin/sh").exists() {
        "/bin/sh"
    } else if Path::new("/usr/bin/sh").exists() {
        "/usr/bin/sh"
    } else {
        // Fallback for weird compile environments, assume /bin/sh or skip
        "/bin/sh" 
    };
    
    if !Path::new(sh_path).exists() {
        eprintln!("Skipping test_run_absolute_exec_portable: /bin/sh not found");
        return;
    }

    let work_dir = tempfile::tempdir().unwrap();
    let a_path = work_dir.path().join("a.txt");
    fs::write(&a_path, "content").unwrap();
    
    let script_dir = tempfile::tempdir().unwrap();
    let script_path = script_dir.path().join("test_abs.sh2");
    
    // run("/bin/sh", "-c", "echo hello > b.txt")
    // This tests if loader crashes on "/bin/sh" string.
    let src = format!(r#"
        func main() {{
            with cwd("{}") {{
                run("{}", "-c", "echo hello > b.txt")
            }}
        }}
    "#, work_dir.path().to_string_lossy(), sh_path);
    
    fs::write(&script_path, src).unwrap();

    let shell = common::compile_path_to_shell(&script_path, common::TargetShell::Bash);
    
    let status = std::process::Command::new("bash")
        .arg("-c")
        .arg(&shell)
        .status()
        .expect("Failed to execute bash");
        
    assert!(status.success(), "Script failed");
    
    let b_path = work_dir.path().join("b.txt");
    assert!(b_path.exists(), "b.txt should be created by absolute sh run");
}
