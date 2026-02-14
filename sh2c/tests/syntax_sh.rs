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

#[test]
fn test_sh_args_default_empty() {
    common::assert_exec_matches_fixture("sh_args_default_empty");
}

#[test]
fn test_sh_args_forward_one() {
    common::assert_exec_matches_fixture("sh_args_forward_one");
}

#[test]
fn test_sh_args_forward_count() {
    common::assert_exec_matches_fixture("sh_args_forward_count");
}

#[test]
fn test_sh_args_forward_fail() {
    common::assert_exec_matches_fixture("sh_args_forward_fail");
}
