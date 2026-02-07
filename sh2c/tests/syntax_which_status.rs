
mod common;

#[test]
fn exec_which_status_no_abort_bash() {
    common::assert_exec_matches_fixture_target("which_status_no_abort", sh2c::codegen::TargetShell::Bash);
}

#[test]
fn exec_which_status_no_abort_posix() {
    common::assert_exec_matches_fixture_target("which_status_no_abort", sh2c::codegen::TargetShell::Posix);
}
