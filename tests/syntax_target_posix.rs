mod common;
use common::{assert_codegen_matches_snapshot_target, assert_exec_matches_fixture_target, TargetShell};

#[test]
fn codegen_posix_params() {
    assert_codegen_matches_snapshot_target("posix_params", TargetShell::Posix);
}

#[test]
fn exec_posix_params() {
    assert_exec_matches_fixture_target("posix_params", TargetShell::Posix);
}
