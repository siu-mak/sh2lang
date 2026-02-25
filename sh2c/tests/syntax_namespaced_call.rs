mod common;

use sh2c::target::TargetShell;

#[test]
fn compile_namespaced_call_basic() {
    common::assert_codegen_matches_snapshot("namespaced_call/basic_call");
}

#[test]
fn compile_namespaced_call_basic_posix() {
    common::assert_codegen_matches_snapshot_target("namespaced_call/basic_call", TargetShell::Posix);
}

#[test]
fn exec_namespaced_call_basic() {
    common::assert_exec_matches_fixture_target("namespaced_call/basic_call", TargetShell::Bash);
}

#[test]
fn compile_namespaced_call_err_unknown_alias() {
    common::assert_codegen_panics("namespaced_call/err_unknown_alias", "unknown import alias 'unknown'");
}

#[test]
fn compile_namespaced_call_err_unknown_func() {
    common::assert_codegen_panics("namespaced_call/err_unknown_func", "unknown function 'mylib.does_not_exist'");
}

#[test]
fn compile_namespaced_call_err_chained_call() {
    common::assert_codegen_panics("namespaced_call/err_chained_call", "chained qualified calls are not supported");
}

#[test]
fn compile_namespaced_call_err_missing_paren() {
    common::assert_codegen_panics("namespaced_call/err_missing_paren", "Expected '(' after 'mylib.greet'");
}
