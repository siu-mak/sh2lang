mod common;
use common::*;

#[test]
fn compile_toplevel_print() {
    assert_codegen_matches_snapshot("toplevel_print");
}

#[test]
fn exec_toplevel_print() {
    assert_exec_matches_fixture("toplevel_print");
}

#[test]
fn exec_toplevel_print_posix() {
    assert_exec_matches_fixture_target("toplevel_print", TargetShell::Posix);
}

#[test]
fn compile_toplevel_let_and_run() {
    assert_codegen_matches_snapshot("toplevel_let_and_run");
}

#[test]
fn exec_toplevel_let_and_run() {
    assert_exec_matches_fixture("toplevel_let_and_run");
}

#[test]
fn exec_toplevel_let_and_run_posix() {
    assert_exec_matches_fixture_target("toplevel_let_and_run", TargetShell::Posix);
}

#[test]
fn compile_toplevel_with_explicit_main_is_error() {
    assert_codegen_panics("toplevel_with_explicit_main_is_error", "Top-level statements are not allowed when `func main` is defined");
}

#[test]
fn compile_no_entrypoint_is_error() {
    assert_codegen_panics("no_entrypoint_is_error", "No entrypoint");
}

#[test]
fn compile_imported_module_has_toplevel_is_error() {
    assert_codegen_panics("imports/module_has_toplevel_is_error/main", "Top-level statements are only allowed in the entry file");
}
