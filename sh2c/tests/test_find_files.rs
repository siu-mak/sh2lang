mod common;
use common::*;

#[test]
fn exec_find_files_basic() {
    assert_exec_matches_fixture_target("find_files_basic", TargetShell::Bash);
}

#[test]
fn exec_find_files_subdir() {
    assert_exec_matches_fixture_target("find_files_subdir", TargetShell::Bash);
}

#[test]
fn exec_find_files_weird_names() {
    assert_exec_matches_fixture_target("find_files_weird_names", TargetShell::Bash);
}

#[test]
fn fail_find_files_posix_target() {
    let path = std::path::Path::new("tests/fixtures/find_files_posix_reject.sh2");
    let res = try_compile_path_to_shell(path, TargetShell::Posix);
    match res {
        Err(msg) => assert!(msg.contains("find_files() is only supported in Bash"), "Unexpected error: {}", msg),
        Ok(_) => panic!("Expected compilation failure for find_files on POSIX target"),
    }
}
