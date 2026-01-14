mod common;
use common::*;

#[test]
fn parse_write_file_stmts() {
    let _ = parse_fixture("write_file_overwrite");
}

#[test]
fn exec_write_file_overwrite() {
    assert_exec_matches_fixture("write_file_overwrite");
}

#[test]
fn exec_append_file_basic() {
    assert_exec_matches_fixture("append_file_basic");
}

#[test]
fn exec_write_file_error_is_dir() {
    // Should fail with exit code != 0
    let (_stdout, stderr) = compile_and_run_err("write_file_error_is_dir", TargetShell::Bash);
    assert!(stderr.contains("Is a directory") || stderr.contains("directory"));
}

#[test]
fn exec_write_file_error_is_dir_posix() {
    let (_stdout, stderr) = compile_and_run_err("write_file_error_is_dir", TargetShell::Posix);
    assert!(stderr.contains("Is a directory") || stderr.contains("directory"));
}
