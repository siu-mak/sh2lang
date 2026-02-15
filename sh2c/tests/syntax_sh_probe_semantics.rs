use std::path::Path;
mod common;
use common::*;

#[test]
fn test_sh_probe_status() {
    let name = "sh_probe_status";
    let path = Path::new("tests/fixtures").join(format!("{}.sh2", name));
    
    // Bash
    let bash_out = compile_path_to_shell(&path, TargetShell::Bash);
    let (stdout, _, status) = run_shell_script(&bash_out, "bash", &[], &[], None, None);
    assert_eq!(status, Some(0), "Bash execution failed");
    
    // Expect: 1, 2, 0, 0 (and "ok" from echo) -> Order depends on buffering, but output should contain these.
    // print(status) puts output on stdout.
    // "exit 1" -> output 1
    // "exit 2" -> output 2
    // "true" -> output 0
    // "echo ok" -> output ok \n 0
    
    let expected_output_snippets = vec!["1", "2", "0", "ok"];
    for s in expected_output_snippets {
        assert!(stdout.contains(s), "Bash stdout missing '{}':\n{}", s, stdout);
    }

    // Posix
    let posix_out = compile_path_to_shell(&path, TargetShell::Posix);
    let (stdout, _, status) = run_shell_script(&posix_out, "sh", &[], &[], None, None);
    assert_eq!(status, Some(0), "Posix execution failed");
    
    let expected_output_snippets = vec!["1", "2", "0", "ok"];
    for s in expected_output_snippets {
        assert!(stdout.contains(s), "Posix stdout missing '{}':\n{}", s, stdout);
    }
}

#[test]
fn test_sh_probe_no_fail_fast() {
    // Tests that sh("false") does not abort the script
    // Can reuse sh_probe_status.sh2 logic implicitly since it continues after exit 1
    // But explicit check:
    let code = r#"
        func main() {
            sh("false")
            print("alive")
        }
    "#;
    
    // Bash
    let bs_out = compile_to_shell(code, TargetShell::Bash);
    let (stdout, _, status) = run_shell_script(&bs_out, "bash", &[], &[], None, None);
    assert_eq!(status, Some(0));
    assert!(stdout.contains("alive"));
    
    // Posix
    let ps_out = compile_to_shell(code, TargetShell::Posix);
    let (stdout, _, status) = run_shell_script(&ps_out, "sh", &[], &[], None, None);
    assert_eq!(status, Some(0));
    assert!(stdout.contains("alive"));
}

#[test]
fn test_sh_probe_errexit_safe() {
    // Regression test: ensure probe doesn't abort under set -e
    // The helper uses 'if ...; then ... else ...' to safely capture status.
    let code = r#"
        func main() {
             sh("false")
             print("after")
             print(status())
        }
    "#;
    
    // Bash
    let bs_out = compile_to_shell(code, TargetShell::Bash);
    // Explicitly run with -e to prove safety
    let (stdout, _, status) = run_shell_script_with_flags(&bs_out, "bash", &["-e"], &[], &[], None, None);
    assert_eq!(status, Some(0), "Script aborted early under set -e (Bash)");
    assert!(stdout.contains("after"), "Execution did not reach 'after' (Bash)");
    assert!(stdout.contains("1"), "Status code 1 missing from output (Bash)");
    
    // Posix
    let ps_out = compile_to_shell(code, TargetShell::Posix);
    let (stdout, _, status) = run_shell_script_with_flags(&ps_out, "sh", &["-e"], &[], &[], None, None);
    assert_eq!(status, Some(0), "Script aborted early under set -e (Posix)");
    assert!(stdout.contains("after"), "Execution did not reach 'after' (Posix)");
    assert!(stdout.contains("1"), "Status code 1 missing from output (Posix)");
}
