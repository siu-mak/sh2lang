mod common;

#[test]
fn test_sh_basic() {
    common::assert_exec_matches_fixture("sh_basic");
}

#[test]
fn test_sh_allow_fail_capture() {
    common::assert_exec_matches_fixture("sh_allow_fail_capture");
}

#[test]
fn test_sh_shell_option() {
    common::assert_exec_matches_fixture("sh_shell_option");
}

#[test]
fn test_sh_allow_fail_stmt() {
    common::assert_exec_matches_fixture("sh_allow_fail_stmt");
}
