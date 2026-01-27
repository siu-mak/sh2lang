use assert_cmd::Command;
use std::fs;

#[test]
fn test_sh2do_run_mv_implicit_cwd() {
    // Scenario: sh2do 'run("mv", "src", "dst")'
    // Executed in a temp dir containing src/file.txt
    
    let work_dir = tempfile::tempdir().unwrap();
    let src_dir = work_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("file.txt"), "content").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2do")); // wrapper
    cmd.current_dir(work_dir.path())
        .arg("run(\"mv\", \"src\", \"dst\")")
        .assert()
        .success();

    assert!(!src_dir.exists(), "src should be gone");
    assert!(work_dir.path().join("dst").exists(), "dst should exist");
    assert!(work_dir.path().join("dst/file.txt").exists(), "file.txt should exist");
}

#[test]
fn test_sh2do_run_with_cwd() {
    // Scenario: sh2do 'with cwd("<abs_tmp>") { run("mv", "src", "dst") }'
    
    let work_dir = tempfile::tempdir().unwrap();
    let src_dir = work_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("file.txt"), "content").unwrap();
    
    let work_path = work_dir.path().to_string_lossy();
    
    let snippet = format!(
        r#"with cwd("{}") {{ run("mv", "src", "dst") }}"#, 
        work_path
    );

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sh2do")); // wrapper
    // We can run from anywhere, but let's run from a different temp dir to be sure
    let run_dir = tempfile::tempdir().unwrap();
    
    cmd.current_dir(run_dir.path())
        .arg(snippet)
        .assert()
        .success();

    assert!(!src_dir.exists(), "src should be gone");
    assert!(work_dir.path().join("dst").exists(), "dst should exist");
}
