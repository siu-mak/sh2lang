use common::assert_exec_matches_fixture;
use assert_cmd::Command;
use predicates::prelude::*;

mod common;

#[test]
fn compile_split_basic() {
    assert_exec_matches_fixture("split_basic");
}

#[test]
fn compile_split_empty_fields() {
    assert_exec_matches_fixture("split_empty_fields");
}

#[test]
fn compile_split_not_found() {
    assert_exec_matches_fixture("split_not_found");
}

#[test]
fn compile_split_multichar() {
    assert_exec_matches_fixture("split_multichar");
}

#[test]
fn compile_split_usage_error() {
    let mut cmd = Command::cargo_bin("sh2c").expect("Failed to find binary");
    cmd.arg("tests/fixtures/split_usage_error.sh2")
       .assert()
       .failure()
       .stderr(predicate::str::contains("Cannot emit boolean/list value as string"))
       .stderr(predicate::str::contains("Internal error").not());
}


#[test]
fn compile_split_basic_posix() {
    let mut cmd = Command::cargo_bin("sh2c").expect("Failed to find binary");
    cmd.arg("--target").arg("posix");
    cmd.arg("tests/fixtures/split_basic.sh2");
    
    // Assert compilation succeeds and emits valid POSIX loop structure
    cmd.assert()
       .success()
       .stdout(predicate::str::contains("__sh2_split() {"))
       .stdout(predicate::str::contains("awk -v s=\"$1\"")) // POSIX simple awk
       .stdout(predicate::str::contains("xs=\"$(__sh2_tmpfile)\""))
       .stdout(predicate::str::contains("__sh2_split \"$s\" ',' > \"$xs\""))
       .stdout(predicate::str::contains("while IFS= read -r x || [ -n \"$x\" ]; do")); // File-backed iteration
}

#[test]
fn exec_split_empty_fields_posix() {
    let src = std::fs::read_to_string("tests/fixtures/split_empty_fields_posix.sh2").unwrap();
    let sh_code = common::compile_to_shell(&src, common::TargetShell::Posix);
    let (stdout, _, _) = common::run_shell_script(&sh_code, "sh", &[], &[], None, None);
    let expected = std::fs::read_to_string("tests/fixtures/split_empty_fields_posix.stdout").unwrap();
    assert_eq!(stdout, expected);
}
