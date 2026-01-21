mod common;
use common::*;

#[test]
fn compile_toplevel_print_fails() {
    assert_codegen_panics(
        "toplevel_print",
        "Top-level statements are not allowed",
    );
}

#[test]
fn compile_toplevel_let_fails() {
    assert_codegen_panics(
        "toplevel_let_and_run",
        "Top-level statements are not allowed",
    );
}

#[test]
fn compile_toplevel_with_explicit_main_fails() {
    assert_codegen_panics(
        "toplevel_with_explicit_main_is_error",
        "Top-level statements are not allowed",
    );
}

#[test]
fn compile_imported_module_has_toplevel_fails() {
    // Parser now catches this before loader
    assert_codegen_panics(
        "imports/module_has_toplevel_is_error/main",
        "Top-level statements are not allowed",
    );
}

#[test]
fn compile_no_entrypoint_is_error() {
    assert_codegen_panics("no_entrypoint_is_error", "No entrypoint");
}

#[test]
fn compile_minimal_main_works() {
    assert_codegen_matches_snapshot("toplevel_minimal_main_ok");
    assert_exec_matches_fixture("toplevel_minimal_main_ok");
}
