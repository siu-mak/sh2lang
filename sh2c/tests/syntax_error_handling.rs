mod common;
use common::{assert_codegen_matches_snapshot, assert_exec_matches_fixture, parse_fixture};
use sh2c::ast::{Stmt, StmtKind};

#[test]
fn parse_try_catch_basic() {
    let program = parse_fixture("try_catch_basic");
    let func = &program.functions[0];
    assert!(matches!(
        func.body[0],
        Stmt {
            node: StmtKind::TryCatch { .. },
            ..
        }
    ));
}

#[test]
fn codegen_try_catch_basic() {
    assert_codegen_matches_snapshot("try_catch_basic");
}

#[test]
fn codegen_try_catch_basic_posix() {
    common::assert_codegen_matches_snapshot_target("try_catch_basic", common::TargetShell::Posix);
}

#[test]
fn codegen_try_catch_success() {
    assert_codegen_matches_snapshot("try_catch_success");
}

#[test]
fn codegen_try_catch_success_posix() {
    common::assert_codegen_matches_snapshot_target("try_catch_success", common::TargetShell::Posix);
}

#[test]
fn exec_try_catch_basic() {
    assert_exec_matches_fixture("try_catch_basic");
}

#[test]
fn exec_try_catch_basic_posix() {
    common::assert_exec_matches_fixture_target("try_catch_basic", common::TargetShell::Posix);
}

#[test]
fn codegen_try_catch_nounset() {
    assert_codegen_matches_snapshot("try_catch_nounset");
}

#[test]
fn codegen_try_catch_nounset_posix() {
    common::assert_codegen_matches_snapshot_target("try_catch_nounset", common::TargetShell::Posix);
}

#[test]
fn exec_try_catch_nounset_bash() {
    common::assert_exec_matches_fixture_target("try_catch_nounset", common::TargetShell::Bash);
}

#[test]
fn exec_try_catch_nounset_posix() {
    common::assert_exec_matches_fixture_target("try_catch_nounset", common::TargetShell::Posix);
}

#[test]
#[should_panic(expected = "no runtime expectation files")]
fn exec_fixture_missing_payload_fails() {
    common::assert_exec_matches_fixture_target("exec_harness/no_payload", common::TargetShell::Bash);
}
