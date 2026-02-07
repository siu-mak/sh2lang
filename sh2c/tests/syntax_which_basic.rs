mod common;
use common::*;
use sh2c::codegen::TargetShell;

#[test]
fn codegen_which_basic() {
    assert_codegen_matches_snapshot("which_basic");
}

#[test]
fn exec_which_basic_bash() {
    assert_exec_matches_fixture_target("which_basic", TargetShell::Bash);
}

#[test]
fn exec_which_basic_posix() {
    assert_exec_matches_fixture_target("which_basic", TargetShell::Posix);
}
