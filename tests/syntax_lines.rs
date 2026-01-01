mod common;
use common::*;

#[test]
fn exec_lines_basic() {
    assert_exec_matches_fixture_target("stdlib/lines", TargetShell::Bash);
}

#[test]
fn codegen_lines() {
    assert_codegen_matches_snapshot("stdlib/lines");
}

#[test]
fn fail_lines_posix_target() {
    // Should fail compilation on POSIX target
    assert_codegen_panics_target("stdlib/lines", TargetShell::Posix, "lines() iteration not supported in POSIX");
}

#[test]
fn fail_lines_invalid_context() {
    // Should fail compilation if used outside let/for
    assert_codegen_panics("stdlib/lines_invalid_ctx", "lines() is only valid in 'for' loops or 'let'");
}
