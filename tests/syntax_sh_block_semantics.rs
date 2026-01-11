use std::path::Path;
mod common;
use common::*;

#[test]
fn test_sh_block_direct_pipeline() {
    let name = "sh_block_direct_pipeline";
    let path = Path::new("tests/fixtures").join(format!("{}.sh2", name));
    
    // Bash
    let bash_out = compile_path_to_shell(&path, TargetShell::Bash);
    
    // Assert no "bash -c" generated for this fixture
    assert!(!bash_out.contains("bash -c"), "Generated script contained 'bash -c' but should use direct pipeline");
    
    let (stdout, _, status) = run_shell_script(&bash_out, "bash", &[], &[], None, None);
    assert_eq!(status, 0, "Bash execution failed");
    assert_eq!(stdout.trim(), "after");

    // Posix
    let posix_out = compile_path_to_shell(&path, TargetShell::Posix);
    
    // Assert no "sh -c" generated
     assert!(!posix_out.contains("sh -c"), "Generated script contained 'sh -c' but should use direct pipeline");

    let (stdout, _, status) = run_shell_script(&posix_out, "sh", &[], &[], None, None);
    assert_eq!(status, 0, "Posix execution failed");
    assert_eq!(stdout.trim(), "after");
}

#[test]
fn test_sh_block_fail_fast() {
    let name = "sh_block_fail_fast";
    let path = Path::new("tests/fixtures").join(format!("{}.sh2", name));
    
    // Bash
    let bash_out = compile_path_to_shell(&path, TargetShell::Bash);
    let (stdout, _, status) = run_shell_script(&bash_out, "bash", &[], &[], None, None);
    
    // Should fail fast (exit code != 0)
    assert_ne!(status, 0, "Bash should have failed fast");
    assert!(!stdout.contains("should_not_print"));

    // Posix
    let posix_out = compile_path_to_shell(&path, TargetShell::Posix);
    let (stdout, _, status) = run_shell_script(&posix_out, "sh", &[], &[], None, None);
    
    // Should fail fast
    assert_ne!(status, 0, "Posix should have failed fast");
    assert!(!stdout.contains("should_not_print"));
}
