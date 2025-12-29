mod common;
use common::*;

#[test]
fn codegen_try_run_captures_bash() {
    assert_codegen_matches_snapshot("try_run_captures");
}

#[test]
fn codegen_try_run_captures_posix() {
    assert_codegen_matches_snapshot_target("try_run_captures", TargetShell::Posix);
}

#[test]
fn exec_try_run_captures_bash() {
    assert_exec_matches_fixture_target("try_run_captures", TargetShell::Bash);
}

#[test]
fn exec_try_run_captures_posix() {
    assert_exec_matches_fixture_target("try_run_captures", TargetShell::Posix);
}

#[test]
fn codegen_try_run_success_bash() {
    assert_codegen_matches_snapshot("try_run_success");
}

#[test]
fn codegen_try_run_success_posix() {
    assert_codegen_matches_snapshot_target("try_run_success", TargetShell::Posix);
}

#[test]
fn exec_try_run_success_bash() {
    assert_exec_matches_fixture_target("try_run_success", TargetShell::Bash);
}

#[test]
fn exec_try_run_success_posix() {
    assert_exec_matches_fixture_target("try_run_success", TargetShell::Posix);
}

#[test]
fn compile_try_run_stmt_invalid() {
    assert_codegen_panics("try_run_stmt_invalid", "try_run() must be bound via let");
}

#[test]
fn compile_try_run_expr_invalid() {
    assert_codegen_panics("try_run_expr_invalid", "try_run() must be bound via let");
}

#[test]
fn compile_try_run_field_unknown_invalid() {
    assert_codegen_panics(
        "try_run_field_unknown_invalid",
        "Unknown field 'nope'. Supported: status, stdout, stderr, flags, positionals.",
    );
}

#[test]
fn compile_try_run_field_base_invalid() {
    assert_codegen_panics(
        "try_run_field_base_invalid",
        ".stdout is only valid on try_run() results (bind via let)",
    );
}

#[test]
fn compile_try_run_field_shadow_invalid() {
    assert_codegen_panics(
        "try_run_field_shadow_invalid",
        ".stdout is only valid on try_run() results (bind via let)",
    );
}

#[test]
fn exec_try_run_fields_bash() {
    assert_exec_matches_fixture_target("try_run_fields", TargetShell::Bash);
}

#[test]
fn exec_try_run_fields_posix() {
    assert_exec_matches_fixture_target("try_run_fields", TargetShell::Posix);
}

#[test]
fn codegen_try_run_fields_bash() {
    assert_codegen_matches_snapshot("try_run_fields");
}

#[test]
fn codegen_try_run_fields_posix() {
    assert_codegen_matches_snapshot_target("try_run_fields", TargetShell::Posix);
}

#[test]
fn compile_try_run_field_if_partial_invalid() {
    assert_codegen_panics(
        "try_run_field_if_partial_invalid",
        ".stdout is only valid on try_run() results (bind via let)",
    );
}

#[test]
fn compile_try_run_field_if_overwrite_invalid() {
    assert_codegen_panics(
        "try_run_field_if_overwrite_invalid",
        ".stdout is only valid on try_run() results (bind via let)",
    );
}

#[test]
fn exec_try_run_field_if_both_ok_bash() {
    assert_exec_matches_fixture_target("try_run_field_if_both_ok", TargetShell::Bash);
}

#[test]
fn exec_try_run_field_if_both_ok_posix() {
    assert_exec_matches_fixture_target("try_run_field_if_both_ok", TargetShell::Posix);
}

#[test]
fn exec_try_run_field_subshell_no_leak_ok_bash() {
    assert_exec_matches_fixture_target("try_run_field_subshell_no_leak_ok", TargetShell::Bash);
}

#[test]
fn exec_try_run_field_subshell_no_leak_ok_posix() {
    assert_exec_matches_fixture_target("try_run_field_subshell_no_leak_ok", TargetShell::Posix);
}

#[test]
fn compile_try_run_field_group_leak_invalid() {
    assert_codegen_panics(
        "try_run_field_group_leak_invalid",
        ".stdout is only valid on try_run() results (bind via let)",
    );
}

#[test]
fn compile_try_run_field_with_env_leak_invalid() {
    assert_codegen_panics(
        "try_run_field_with_env_leak_invalid",
        ".stdout is only valid on try_run() results (bind via let)",
    );
}

#[test]
fn compile_try_run_field_with_cwd_leak_invalid() {
    assert_codegen_panics(
        "try_run_field_with_cwd_leak_invalid",
        ".stdout is only valid on try_run() results (bind via let)",
    );
}

#[test]
fn compile_try_run_field_with_log_leak_invalid() {
    assert_codegen_panics(
        "try_run_field_with_log_leak_invalid",
        ".stdout is only valid on try_run() results (bind via let)",
    );
}

#[test]
fn compile_try_run_field_with_redirect_leak_invalid() {
    // This is invalid because the inner 'let r = "shadowed"' PERSISTS to the outer scope,
    // shadowing the outer 'let r = try_run(...)'.
    assert_codegen_panics(
        "try_run_field_with_redirect_leak_invalid",
        ".stdout is only valid on try_run() results (bind via let)",
    );
}

#[test]
fn exec_try_run_field_with_redirect_leak_ok_bash() {
    assert_exec_matches_fixture_target("try_run_field_with_redirect_leak_ok", TargetShell::Bash);
}

#[test]
fn exec_try_run_field_with_redirect_leak_ok_posix() {
    assert_exec_matches_fixture_target("try_run_field_with_redirect_leak_ok", TargetShell::Posix);
}
