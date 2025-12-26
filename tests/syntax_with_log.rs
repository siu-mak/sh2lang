use crate::common::*;

mod common;

#[test]
fn exec_with_log_basic() {
    assert_exec_matches_fixture_target("with_log_basic", TargetShell::Bash);
}

#[test]
fn exec_with_log_append() {
    assert_exec_matches_fixture_target("with_log_append", TargetShell::Bash);
}

#[test]
fn exec_with_log_try_catch() {
    assert_exec_matches_fixture_target("with_log_try_catch", TargetShell::Bash);
}

#[test]
fn compile_panic_with_log_posix_unsupported() {
    assert_codegen_panics_target("with_log_posix_unsupported", TargetShell::Posix, "with log(...) is not supported in POSIX sh target");
}
