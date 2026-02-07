mod common;
use common::*;
use sh2c::codegen::TargetShell;

#[test]
fn exec_which_symlink_ok_bash() {
    assert_exec_matches_fixture_target("which_symlink_ok", TargetShell::Bash);
}

#[test]
fn exec_which_symlink_ok_posix() {
    assert_exec_matches_fixture_target("which_symlink_ok", TargetShell::Posix);
}
