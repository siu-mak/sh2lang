mod common;
use common::*;

#[test]
fn compile_import_basic() {
    assert_codegen_matches_snapshot("imports/basic/main");
    assert_codegen_matches_snapshot_target("imports/basic/main", TargetShell::Posix);
}

#[test]
fn exec_import_basic() {
    assert_exec_matches_fixture("imports/basic/main");
}

#[test]
fn compile_import_chain() {
    assert_codegen_matches_snapshot("imports/chain/main");
    assert_codegen_matches_snapshot_target("imports/chain/main", TargetShell::Posix);
}

#[test]
fn exec_import_chain() {
    assert_exec_matches_fixture("imports/chain/main");
}

#[test]
fn compile_import_duplicate() {
    assert_codegen_panics(
        "imports/duplicate/main",
        "Function 'dup' is already defined",
    );
}

#[test]
fn compile_import_cycle() {
    assert_codegen_panics("imports/cycle/main", "Import cycle detected");
}

#[test]
fn compile_import_missing() {
    assert_codegen_panics("imports/missing/main", "Failed to resolve path");
}

#[test]
fn compile_import_not_top_level() {
    assert_codegen_panics(
        "imports/not_top_level/main",
        "import is only allowed at top-level",
    );
}

#[test]
fn compile_import_double_import() {
    assert_codegen_matches_snapshot("imports/double_import/main");
    assert_codegen_matches_snapshot_target("imports/double_import/main", TargetShell::Posix);
}

#[test]
fn exec_import_double_import() {
    assert_exec_matches_fixture("imports/double_import/main");
}

#[test]
fn compile_import_diamond() {
    assert_codegen_matches_snapshot("imports/diamond/main");
    assert_codegen_matches_snapshot_target("imports/diamond/main", TargetShell::Posix);
}

#[test]
fn exec_import_diamond() {
    assert_exec_matches_fixture("imports/diamond/main");
}

#[test]
fn exec_import_double_import_posix() {
    assert_exec_matches_fixture_target("imports/double_import/main", TargetShell::Posix);
}

#[test]
fn exec_import_diamond_posix() {
    assert_exec_matches_fixture_target("imports/diamond/main", TargetShell::Posix);
}
