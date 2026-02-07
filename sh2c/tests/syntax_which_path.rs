
mod common;

#[test]
fn exec_which_path_empty_segments_bash() {
    common::assert_exec_matches_fixture_target("which_path_empty_segments", sh2c::codegen::TargetShell::Bash);
}

#[test]
fn exec_which_path_empty_segments_posix() {
    common::assert_exec_matches_fixture_target("which_path_empty_segments", sh2c::codegen::TargetShell::Posix);
}
