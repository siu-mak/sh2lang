//! Tests for variable declaration policy: declare-before-use, set requires prior let, no redeclare

mod common;
use common::{try_compile_to_shell, TargetShell};

#[test]
fn test_undeclared_in_print_is_error() {
    let src = r#"func main(){ print(b) }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected compile error for undefined variable");
    let err = result.unwrap_err();
    assert!(err.contains("undefined variable 'b'"), "Expected 'undefined variable' error, got: {}", err);
}

#[test]
fn test_undeclared_in_concat_is_error() {
    let src = r#"func main(){ print("x=" & b) }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected compile error for undefined variable");
    let err = result.unwrap_err();
    assert!(err.contains("undefined variable 'b'"), "Expected 'undefined variable' error, got: {}", err);
}

#[test]
fn test_set_before_let_is_error() {
    let src = r#"func main(){ set b = "1" }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected compile error for undeclared set");
    let err = result.unwrap_err();
    assert!(err.contains("cannot set undeclared variable 'b'"), "Expected 'cannot set undeclared' error, got: {}", err);
}

#[test]
fn test_redeclare_let_same_scope_is_error() {
    let src = r#"func main(){ let b="0"; let b="1" }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected compile error for redeclaration");
    let err = result.unwrap_err();
    assert!(err.contains("already declared"), "Expected 'already declared' error, got: {}", err);
}

#[test]
fn test_let_then_set_ok() {
    let src = r#"func main(){ let b="0"; set b="1"; print(b) }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_ok(), "Expected successful compile, got error: {:?}", result.err());
}

#[test]
fn test_conditional_let_usage_is_error() {
    let src = r#"func main(){ if true { let x = "1" }; print(x) }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected compile error for conditionally declared variable");
    let err = result.unwrap_err();
    assert!(err.contains("undefined variable 'x'"), "Expected 'undefined variable' error, got: {}", err);
}

#[test]
fn test_set_in_branch_ok() {
    let src = r#"func main(){ let x="0"; if true { set x="1" }; print(x) }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_ok(), "Expected successful compile, got error: {:?}", result.err());
}

#[test]
fn test_for_loop_var_scoped_to_function() {
    // For-loop variable persists after loop (function scope)
    let src = r#"func main(){ for i in [1, 2] { print(i) }; print(i) }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_ok(), "For-loop var should be usable after loop, got error: {:?}", result.err());
}

#[test]
fn test_for_loop_var_redeclare_is_error() {
    // Can't redeclare for-loop variable
    let src = r#"func main(){ for i in [1, 2] { print(i) }; let i = "x" }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected error for redeclaring for-loop var");
    let err = result.unwrap_err();
    assert!(err.contains("already declared"), "Expected 'already declared' error, got: {}", err);
}

#[test]
fn test_function_param_is_declared() {
    // Function parameters are automatically declared
    let src = r#"func greet(name){ print("Hello " & name) } func main(){ greet("world") }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_ok(), "Function param should be declared, got error: {:?}", result.err());
}

#[test]
fn test_function_param_set_ok() {
    // Can set function parameter (already declared)
    let src = r#"func f(x){ set x = "new"; print(x) } func main(){ f("old") }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_ok(), "Setting function param should be OK, got error: {:?}", result.err());
}

#[test]
fn test_function_param_redeclare_is_error() {
    // Can't redeclare function parameter with let
    let src = r#"func f(x){ let x = "new"; print(x) } func main(){ f("old") }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected error for redeclaring function param");
    let err = result.unwrap_err();
    assert!(err.contains("already declared"), "Expected 'already declared' error, got: {}", err);
}

#[test]
fn test_run_arg_undeclared_is_error() {
    let src = r#"func main() { run("echo", b) }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected error for undeclared run arg");
    let err = result.unwrap_err();
    assert!(err.contains("undefined variable 'b'"), "Expected 'undefined variable' error, got: {}", err);
}

#[test]
fn test_sudo_arg_undeclared_is_error() {
    let src = r#"func main() { sudo("echo", b) }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected error for undeclared sudo arg");
    let err = result.unwrap_err();
    assert!(err.contains("undefined variable 'b'"), "Expected 'undefined variable' error, got: {}", err);
}

#[test]
fn test_exec_arg_undeclared_is_error() {
    let src = r#"func main() { exec(["echo", b]) }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected error for undeclared exec arg");
    let err = result.unwrap_err();
    assert!(err.contains("undefined variable 'b'"), "Expected 'undefined variable' error, got: {}", err);
}

#[test]
fn test_try_run_unbound_is_error() {
    // try_run not bound to let is error (priority check)
    let src = r#"func main() { print(try_run("ls")) }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected error for unbound try_run");
    let err = result.unwrap_err();
    assert!(err.contains("try_run() must be bound via let"), "Expected 'bound via let' error, got: {}", err);
}

#[test]
fn test_write_file_bad_arg_priority() {
    // Should error on boolean literal requirement BEFORE undefined variable
    let src = r#"func main() { write_file("path", "content", invalid_var) }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected error for bad boolean arg");
    let err = result.unwrap_err();
    assert!(err.contains("append must be boolean literal"), "Expected 'boolean literal' error, got: {}", err);
    assert!(!err.contains("undefined variable"), "Should not report undefined variable");
}
