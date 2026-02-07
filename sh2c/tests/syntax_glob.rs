mod common;
use common::*;

#[test]
fn exec_glob_direct_for() {
    assert_exec_matches_fixture_target("glob_direct_for", TargetShell::Bash);
}

#[test]
fn check_bash_version_guard() {
    // Compile a simple glob usage and check that the preamble contains the version check
    let src = r#"func main() { let x = glob("*") }"#;
    let out = compile_to_shell(src, TargetShell::Bash);
    // Check for the error message which implies the version check block is present
    assert!(out.contains("glob() requires Bash 4.3+"), "Generated script missing version error message");
    // Also check for BASH_VERSINFO to ensure we're checking versions
    assert!(out.contains("BASH_VERSINFO"), "Generated script missing BASH_VERSINFO check");
}

#[test]
fn exec_glob_empty_pattern() {
    assert_exec_matches_fixture_target("glob_empty_pattern", TargetShell::Bash);
}


#[test]
fn exec_glob_basic() {
    assert_exec_matches_fixture_target("glob_basic", TargetShell::Bash);
}

#[test]
fn exec_glob_no_matches() {
    assert_exec_matches_fixture_target("glob_no_matches", TargetShell::Bash);
}

#[test]
fn fail_glob_posix_target() {
    // Should fail compilation on POSIX target with specific error
    let path = std::path::Path::new("tests/fixtures/glob_posix_reject.sh2");
    let res = try_compile_path_to_shell(path, TargetShell::Posix);
    match res {
        Err(msg) => assert!(msg.contains("glob() requires bash target"), "Unexpected error: {}", msg),
        Ok(_) => panic!("Expected compilation failure for glob on POSIX target"),
    }
}

#[test]
fn fail_glob_invalid_context() {
    // Should fail compilation if used outside let/for with specific error
    let path = std::path::Path::new("tests/fixtures/glob_invalid_ctx.sh2");
    let res = try_compile_path_to_shell(path, TargetShell::Bash);
    match res {
        Err(msg) => assert!(msg.contains("glob() must be bound to 'let' or used in 'for' loop"), "Unexpected error: {}", msg),
        Ok(_) => panic!("Expected compilation failure for glob in invalid context"),
    }
}
