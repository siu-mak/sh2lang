use common::check_err_contains;

mod common;

#[test]
fn test_named_arg_policy_generic_call() {
    check_err_contains(
        "named_arg_policy_generic_call",
        "Named arguments are only supported for builtins: run, sudo, sh, capture, confirm"
    );
}
