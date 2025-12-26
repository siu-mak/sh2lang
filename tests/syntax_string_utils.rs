mod common;
use common::*;

#[test]
fn exec_string_coalesce_basic() {
    assert_exec_matches_fixture_target("string_coalesce_basic", TargetShell::Bash);
    assert_exec_matches_fixture_target("string_coalesce_basic", TargetShell::Posix);
}

#[test]
fn exec_string_trim_basic() {
    assert_exec_matches_fixture_target("string_trim_basic", TargetShell::Bash);
    assert_exec_matches_fixture_target("string_trim_basic", TargetShell::Posix);
}

#[test]
fn exec_string_before_after_basic() {
    assert_exec_matches_fixture_target("string_before_after_basic", TargetShell::Bash);
    assert_exec_matches_fixture_target("string_before_after_basic", TargetShell::Posix);
}

#[test]
fn exec_string_replace_basic() {
    assert_exec_matches_fixture_target("string_replace_basic", TargetShell::Bash);
    assert_exec_matches_fixture_target("string_replace_basic", TargetShell::Posix);
}

#[test]
fn exec_string_utils_multiline() {
    assert_exec_matches_fixture_target("string_utils_multiline", TargetShell::Bash);
    assert_exec_matches_fixture_target("string_utils_multiline", TargetShell::Posix);
}

#[test]
fn codegen_string_utils() {
    assert_codegen_matches_snapshot("string_coalesce_basic");
    assert_codegen_matches_snapshot("string_trim_basic");
    assert_codegen_matches_snapshot("string_before_after_basic");
    assert_codegen_matches_snapshot("string_replace_basic");
    assert_codegen_matches_snapshot("string_utils_multiline");
}
