//! Integration tests for sh() codegen: Ticket 10.
//!
//! Verifies that sh("...") executes the string as shell code via `sh -c`
//! in both statement and expression (capture) forms, for Bash and POSIX targets.
//! Also verifies that bare `sh file` (without parens) is NOT rewritten with -c.

mod common;
use common::{compile_to_shell, run_bash_script, run_shell_script, TargetShell};

// --- Statement form ---

#[test]
fn test_sh_c_echo() {
    // sh("echo hi") in statement form
    let src = std::fs::read_to_string(
        format!("{}/tests/fixtures/sh_c_echo.sh2", env!("CARGO_MANIFEST_DIR"))
    ).expect("fixture");

    for (target, shell) in &[(TargetShell::Bash, "bash"), (TargetShell::Posix, "sh")] {
        let script = compile_to_shell(&src, *target);
        let (stdout, stderr, code) = run_shell_script(&script, shell, &[], &[], None, None);
        assert_eq!(code, 0, "target={}: exit code (stderr={})", shell, stderr);
        assert_eq!(stdout.trim(), "hi", "target={}: stdout", shell);
        assert!(!stderr.contains("Error in"), "target={}: unexpected error in stderr: {}", shell, stderr);
    }
}

// --- Expression form (capture) — core Ticket 10 bug ---

#[test]
fn test_sh_c_capture() {
    let src = std::fs::read_to_string(
        format!("{}/tests/fixtures/sh_c_capture.sh2", env!("CARGO_MANIFEST_DIR"))
    ).expect("fixture");

    for (target, shell) in &[(TargetShell::Bash, "bash"), (TargetShell::Posix, "sh")] {
        let script = compile_to_shell(&src, *target);
        let (stdout, stderr, code) = run_shell_script(&script, shell, &[], &[], None, None);
        assert_eq!(code, 0, "target={}: exit code (stderr={})", shell, stderr);
        assert_eq!(stdout.trim(), "hi", "target={}: stdout", shell);
        assert!(!stderr.contains("Error in"), "target={}: unexpected error: {}", shell, stderr);

        // Whitebox: the generated command substitution must contain -c
        // Accept any quoting variant the codegen may produce
        assert!(
            script.contains("-c"),
            "target={}: codegen for capture(sh(...)) must contain -c",
            shell
        );
    }
}

// --- Pipeline ---

#[test]
fn test_sh_c_pipe() {
    let src = std::fs::read_to_string(
        format!("{}/tests/fixtures/sh_c_pipe.sh2", env!("CARGO_MANIFEST_DIR"))
    ).expect("fixture");

    for (target, shell) in &[(TargetShell::Bash, "bash"), (TargetShell::Posix, "sh")] {
        let script = compile_to_shell(&src, *target);
        let (stdout, stderr, code) = run_shell_script(&script, shell, &[], &[], None, None);
        assert_eq!(code, 0, "target={}: exit code (stderr={})", shell, stderr);
        assert!(stdout.contains("2"), "target={}: stdout should contain 2, got: {}", shell, stdout);
        assert!(!stderr.contains("Error in"), "target={}: unexpected error: {}", shell, stderr);
    }
}

// --- Glob expansion ---

#[test]
fn test_sh_c_glob() {
    let src = std::fs::read_to_string(
        format!("{}/tests/fixtures/sh_c_glob.sh2", env!("CARGO_MANIFEST_DIR"))
    ).expect("fixture");

    for (target, shell) in &[(TargetShell::Bash, "bash"), (TargetShell::Posix, "sh")] {
        let script = compile_to_shell(&src, *target);
        let (stdout, stderr, code) = run_shell_script(&script, shell, &[], &[], None, None);
        assert_eq!(code, 0, "target={}: exit code (stderr={})", shell, stderr);
        assert!(stdout.contains("a.txt"), "target={}: should contain a.txt, got: {}", shell, stdout);
        assert!(stdout.contains("b.txt"), "target={}: should contain b.txt, got: {}", shell, stdout);
        assert!(!stderr.contains("Error in"), "target={}: unexpected error: {}", shell, stderr);
    }
}

// --- Variable form ---

#[test]
fn test_sh_c_var() {
    let src = std::fs::read_to_string(
        format!("{}/tests/fixtures/sh_c_var.sh2", env!("CARGO_MANIFEST_DIR"))
    ).expect("fixture");

    for (target, shell) in &[(TargetShell::Bash, "bash"), (TargetShell::Posix, "sh")] {
        let script = compile_to_shell(&src, *target);
        let (stdout, stderr, code) = run_shell_script(&script, shell, &[], &[], None, None);
        assert_eq!(code, 0, "target={}: exit code (stderr={})", shell, stderr);
        assert_eq!(stdout.trim(), "hi_var", "target={}: stdout", shell);
        assert!(!stderr.contains("Error in"), "target={}: unexpected error: {}", shell, stderr);
    }
}

// --- Negative regression: run("sh", "file") must NOT get -c injected ---

#[test]
fn test_sh_file_no_dash_c() {
    // When running a file via sh using run("sh", "file"), the generated code
    // must NOT inject -c. This is the "file runner" path.
    //
    // Note: `sh` is a keyword in sh2, so `capture(sh "file")` is a parse error.
    // The valid way to run a file is `capture(run("sh", "file"))`.
    // This goes through the run() path (not the sh() shorthand) and should
    // produce $( 'sh' 'file' ), NOT $( 'sh' '-c' 'file' ).
    let src = r#"
func main() {
    let x = capture(run("sh", "echo_from_file.sh"))
    print(x)
}
"#;
    let script = compile_to_shell(src, TargetShell::Bash);

    // The command substitution should contain `'sh' 'echo_from_file.sh'`
    // and must NOT contain `'-c'` between them.
    assert!(
        script.contains("'sh' 'echo_from_file.sh'"),
        "run(sh, file) should produce 'sh' 'file', got:\n{}",
        script
    );
    assert!(
        !script.contains("'sh' '-c' 'echo_from_file.sh'"),
        "run(sh, file) must NOT get -c injected, got:\n{}",
        script
    );
}
