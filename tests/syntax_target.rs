mod common;
use common::*;
use sh2c::codegen::TargetShell;

#[test]
fn codegen_target_bash() {
    assert_codegen_matches_snapshot_target("target_scalar_print", TargetShell::Bash);
}

#[test]
fn codegen_target_posix_scalar() {
    assert_codegen_matches_snapshot_target("target_scalar_print", TargetShell::Posix);
}

#[test]
fn codegen_target_posix_list_fail() {
    assert_codegen_panics_target(
        "target_posix_list_unsupported",
        TargetShell::Posix,
        "supported in POSIX",
    );
}

#[test]
fn exec_target_posix_scalar() {
    // Requires 'dash' or similar, strict check handled in helper
    assert_exec_matches_fixture_target("target_scalar_print", TargetShell::Posix);
}

#[test]
fn exec_target_bash_scalar() {
    assert_exec_matches_fixture_target("target_scalar_print", TargetShell::Bash);
}

#[test]
fn codegen_target_posix_params_snapshot() {
    assert_codegen_matches_snapshot_target("target_posix_params", TargetShell::Posix);
}

#[test]
fn exec_target_posix_params() {
    assert_exec_matches_fixture_target("target_posix_params", TargetShell::Posix);
}

#[test]
fn codegen_target_bash_params_snapshot() {
    assert_codegen_matches_snapshot_target("target_posix_params", TargetShell::Bash);
}

#[test]
fn exec_target_bash_params() {
    assert_exec_matches_fixture_target("target_posix_params", TargetShell::Bash);
}

#[test]
fn codegen_target_posix_env_indirect_fail() {
    assert_codegen_panics_target(
        "target_posix_env_indirect_unsupported",
        TargetShell::Posix,
        "env(var_name) is not supported in POSIX",
    );
}
#[test]
fn codegen_target_posix_basic_bash_snapshot() {
    assert_codegen_matches_snapshot_target("target_posix_basic", TargetShell::Bash);
}

#[test]
fn codegen_target_posix_basic_posix_snapshot() {
    assert_codegen_matches_snapshot_target("target_posix_basic", TargetShell::Posix);
}

#[test]
fn exec_target_posix_basic_under_posix_shell() {
    assert_exec_matches_fixture_target("target_posix_basic", TargetShell::Posix);
}

#[test]
fn posix_rejects_indexing() {
    assert_codegen_panics_target(
        "target_posix_rejects_index",
        TargetShell::Posix,
        "supported in POSIX",
    );
}
