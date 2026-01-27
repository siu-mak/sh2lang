use common::{assert_exec_matches_fixture_target, TargetShell};

mod common;

#[test]
fn test_arg_dynamic_index_posix() {
    // 1. Run execution test with args (loaded from .args file) on POSIX target
    // This ensures no "unsupported" error is raised and logic works.
    assert_exec_matches_fixture_target("arg_dynamic_index", TargetShell::Posix);
}
