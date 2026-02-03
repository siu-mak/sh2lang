//! Tests for boolean variable support

mod common;
use common::{assert_exec_matches_fixture, assert_exec_matches_fixture_target, run_test_in_targets, TargetShell};
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn bool_var_basic_bash() {
    assert_exec_matches_fixture("bool_var_basic");
}

#[test]
fn bool_var_basic_posix() {
    assert_exec_matches_fixture_target("bool_var_basic", TargetShell::Posix);
}

#[test]
fn bool_var_print_success() {
    // print(bool_var) must now succeed and verify implicit string conversion at runtime
    // We ignore the old failure fixture and test inline for clarity
    let code = r#"
        func main() {
            let ok = true
            print(ok)
            let fail = false
            print(fail)
        }
    "#;
    run_test_in_targets("bool_var_print_success", code, "true\nfalse");
}
