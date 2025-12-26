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

#[test]
fn compile_panic_with_log_unknown_option() {
    assert_codegen_panics_target("with_log_unknown_option", TargetShell::Bash, "unknown log option: nope");
    assert_codegen_panics_target("with_log_unknown_option", TargetShell::Posix, "unknown log option: nope");
}

#[test]
fn compile_panic_with_log_append_non_bool() {
    assert_codegen_panics_target("with_log_append_non_bool", TargetShell::Bash, "append must be true/false literal");
    assert_codegen_panics_target("with_log_append_non_bool", TargetShell::Posix, "append must be true/false literal");
}

#[test]
fn compile_panic_with_log_append_duplicate() {
    assert_codegen_panics_target("with_log_append_duplicate", TargetShell::Bash, "append specified more than once");
    assert_codegen_panics_target("with_log_append_duplicate", TargetShell::Posix, "append specified more than once");
}

#[test]
fn compile_panic_with_log_append_duplicate_first_false() {
    assert_codegen_panics_target("with_log_append_duplicate_first_false", TargetShell::Bash, "append specified more than once");
    assert_codegen_panics_target("with_log_append_duplicate_first_false", TargetShell::Posix, "append specified more than once");
}
