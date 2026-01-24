mod common;
use common::TargetShell;

#[test]
fn test_confirm_noninteractive_default_false() {
    common::assert_exec_matches_fixture("confirm_noninteractive_default_false");
}

#[test]
fn test_confirm_noninteractive_default_true() {
    common::assert_exec_matches_fixture("confirm_noninteractive_default_true");
}

#[test]
fn test_confirm_env_override_yes() {
    common::assert_exec_matches_fixture("confirm_env_override_yes");
}

#[test]
fn test_confirm_env_override_no() {
    common::assert_exec_matches_fixture("confirm_env_override_no");
}

// Compile-fail tests for confirm() diagnostics

#[test]
fn test_confirm_default_not_bool_literal() {
    let src = r#"
func main() {
    if confirm("go?", default=1) { print("yes") }
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected compile error for default=1");
    let err = result.unwrap_err();
    assert!(
        err.contains("confirm(default=...) must be a true/false literal"),
        "Error should mention bool literal. Got: {}",
        err
    );
}

#[test]
fn test_confirm_unknown_option() {
    let src = r#"
func main() {
    if confirm("go?", x=true) { print("yes") }
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected compile error for unknown option");
    let err = result.unwrap_err();
    assert!(
        err.contains("unknown confirm() option 'x'") && err.contains("supported: default"),
        "Error should mention unknown option and list supported. Got: {}",
        err
    );
}

#[test]
fn test_confirm_duplicate_default() {
    let src = r#"
func main() {
    if confirm("go?", default=true, default=false) { print("yes") }
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected compile error for duplicate default");
    let err = result.unwrap_err();
    assert!(
        err.contains("default specified more than once"),
        "Error should mention duplicate. Got: {}",
        err
    );
}
