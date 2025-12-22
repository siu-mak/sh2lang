mod common;
use sh2c::ast::Stmt;
use common::{parse_fixture, assert_codegen_matches_snapshot, assert_exec_matches_fixture};

#[test]
fn parse_try_catch_basic() {
    let program = parse_fixture("try_catch_basic");
    let func = &program.functions[0];
    assert!(matches!(func.body[0], Stmt::TryCatch { .. }));
}

#[test]
fn codegen_try_catch_basic() { assert_codegen_matches_snapshot("try_catch_basic"); }
#[test]
fn codegen_try_catch_success() { assert_codegen_matches_snapshot("try_catch_success"); }

#[test]
fn exec_try_catch_basic() { assert_exec_matches_fixture("try_catch_basic"); }
