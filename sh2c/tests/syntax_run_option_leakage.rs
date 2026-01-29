mod common;
use common::check_err_contains;

#[test]
fn test_cmd_sub_run_unknown_option() {
    check_err_contains("cmd_sub_run_unknown_option", "run() options are not allowed in command substitution $(...)");
}

#[test]
fn test_capture_run_unknown_option() {
    check_err_contains("capture_run_unknown_option", "Unknown option 'nope' for run()");
}

#[test]
fn test_capture_run_allow_fail_duplicate() {
    check_err_contains("capture_run_allow_fail_duplicate", "allow_fail specified more than once");
}

#[test]
fn test_capture_run_allow_fail_non_bool() {
    check_err_contains("capture_run_allow_fail_non_bool", "allow_fail must be a boolean literal");
}
