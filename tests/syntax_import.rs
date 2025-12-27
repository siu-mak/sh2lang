mod common;
use common::*;

#[test]
fn compile_import_basic() {
    // For compile test, we need to point to the main file.
    // The fixture system copies the directory.
    // But `assert_codegen_matches_snapshot` works on a single file name basis,
    // assuming it's in tests/fixtures/<name>.sh2.
    // Here we have structure tests/fixtures/imports/basic/main.sh2.
    
    // We need to support nested paths in our test harness or manually specify full relative path.
    // `assert_codegen_matches_snapshot` uses `tests/fixtures/{fixture_name}.sh2`
    // So we can pass "imports/basic/main".
    assert_codegen_matches_snapshot("imports/basic/main");
}

#[test]
fn exec_import_basic() {
    // Similarly for exec.
    assert_exec_matches_fixture("imports/basic/main");
}

#[test]
fn compile_import_chain() {
    assert_codegen_matches_snapshot("imports/chain/main");
}

#[test]
fn exec_import_chain() {
    assert_exec_matches_fixture("imports/chain/main");
}

#[test]
fn compile_import_duplicate() {
    assert_codegen_panics("imports/duplicate/main", "Function 'dup' is already defined");
}

#[test]
fn compile_import_cycle() {
    assert_codegen_panics("imports/cycle/main", "Import cycle detected");
}

#[test]
fn compile_import_missing() {
    // Check if error message contains "Failed to read"
    assert_codegen_panics("imports/missing/main", "Failed to resolve path");
}

#[test]
fn compile_import_not_top_level() {
    assert_codegen_panics("imports/not_top_level/main", "import is only allowed at top-level");
}
