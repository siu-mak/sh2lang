mod common;
use common::*;

#[test]
fn exec_starts_with_basic() {
    assert_exec_matches_fixture_target("starts_with_basic", TargetShell::Bash);
    assert_exec_matches_fixture_target("starts_with_basic", TargetShell::Posix);
}
