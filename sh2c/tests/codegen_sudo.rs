mod common;

#[test]
fn test_sudo_basic() {
    common::assert_codegen_matches_snapshot("sudo_basic");
}

#[test]
fn test_sudo_user() {
    common::assert_codegen_matches_snapshot("sudo_user");
}

#[test]
fn test_sudo_opts_order() {
    common::assert_codegen_matches_snapshot("sudo_opts_order");
}

#[test]
fn test_sudo_allow_fail_stmt() {
    common::assert_codegen_matches_snapshot("sudo_allow_fail_stmt");
}

#[test]
fn test_sudo_env_keep() {
    common::assert_codegen_matches_snapshot("sudo_env_keep");
}
