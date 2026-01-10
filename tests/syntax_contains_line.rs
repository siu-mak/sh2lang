use std::path::Path;
mod common;
use common::*;

fn run_test(fixture_name: &str, expected_stdout: &str) {
    let path = Path::new("tests/fixtures").join(format!("{}.sh2", fixture_name));
    
    // Bash
    let bash_out = compile_path_to_shell(&path, TargetShell::Bash);
    let (stdout, _, status) = run_shell_script(&bash_out, "bash", &[], &[], None, None);
    assert_eq!(status, 0, "Bash execution failed");
    if expected_stdout == "pass" {
        assert_eq!(stdout.trim(), "pass");
    } else {
        assert!(stdout.contains(expected_stdout));
    }

    // Posix
    let posix_out = compile_path_to_shell(&path, TargetShell::Posix);
    let (stdout, _, status) = run_shell_script(&posix_out, "sh", &[], &[], None, None);
    assert_eq!(status, 0, "Posix execution failed");
    if expected_stdout == "pass" {
        assert_eq!(stdout.trim(), "pass");
    } else {
        assert!(stdout.contains(expected_stdout));
    }
}

fn run_hostile_test(fixture_name: &str) {
    let path = Path::new("tests/fixtures").join(format!("{}.sh2", fixture_name));
    
    // Bash
    let bash_out = compile_path_to_shell(&path, TargetShell::Bash);
    let (stdout, _, status) = run_shell_script(&bash_out, "bash", &[], &[], None, None);
    assert_eq!(status, 0, "Bash execution failed");
    assert!(stdout.contains("ok 1"));
    assert!(stdout.contains("ok 2"));
    assert!(stdout.contains("ok 3"));
    assert!(stdout.contains("ok 4"));
    assert!(stdout.contains("ok 5"));
    assert!(!stdout.contains("fail"));

    // Posix
    let posix_out = compile_path_to_shell(&path, TargetShell::Posix);
    let (stdout, _, status) = run_shell_script(&posix_out, "sh", &[], &[], None, None);
    assert_eq!(status, 0, "Posix execution failed");
    assert!(stdout.contains("ok 1"));
    assert!(stdout.contains("ok 2"));
    assert!(stdout.contains("ok 3"));
    assert!(stdout.contains("ok 4"));
    assert!(stdout.contains("ok 5"));
    assert!(!stdout.contains("fail"));
}

#[test]
fn test_contains_line_cond_true() {
    run_test("contains_line_cond_true", "pass");
}

#[test]
fn test_contains_line_cond_false() {
    run_test("contains_line_cond_false", "pass");
}

#[test]
fn test_contains_line_trailing() {
    run_test("contains_line_trailing", "pass");
}

#[test]
fn test_contains_line_empty_interior() {
    run_test("contains_line_empty_interior", "pass");
}

#[test]
fn test_contains_line_partial_last() {
    run_test("contains_line_partial_last", "pass");
}

#[test]
fn test_contains_line_backslash() {
    run_test("contains_line_backslash", "pass");
}

#[test]
fn test_contains_line_hostile() {
    run_hostile_test("contains_line_hostile");
}
