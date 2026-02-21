//! Tests for variable declaration policy: declare-before-use, set requires prior let, no redeclare

mod common;
use common::{try_compile_to_shell, compile_to_shell, run_bash_script, TargetShell};
use std::fs;

#[test]
fn test_undeclared_in_print_is_error() {
    let src = r#"func main(){ print(b) }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected compile error for undefined variable");
    let err = result.unwrap_err();
    assert!(err.contains("undefined variable 'b'"), "Expected 'undefined variable' error, got: {}", err);
}

fn compile_to_bash_test(fixture_path: &str) -> (String, String) {
    let root = common::repo_root();
    let abs_path = root.join("sh2c/tests").join(fixture_path);
    let src = fs::read_to_string(&abs_path)
        .expect(&format!("Failed to read fixture: {:?}", abs_path));
    
    match try_compile_to_shell(&src, TargetShell::Bash) {
        Ok(stdout) => (stdout, String::new()),
        Err(stderr) => (String::new(), stderr),
    }
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
    assert!(err.contains("Did you mean to use `let b = ...`?"), "Expected hint in error output");
}

#[test]
fn test_redeclare_let_same_scope_is_error() {
    let src = r#"func main(){ let b="0"; let b="1" }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected compile error for redeclaration");
    let err = result.unwrap_err();
    assert!(err.contains("already declared"), "Expected 'already declared' error, got: {}", err);
    assert!(err.contains("Did you mean to use `set b = ...`?"), "Expected hint in error output");
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
fn test_if_else_disjoint_let_ok() {
    // Policy A: declare in both branches (disjoint) is OK and variable persists
    let src = r#"func main(){ if true { let x="1" } else { let x="2" }; print(x) }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_ok(), "Disjoint let in if/else should be valid in Policy A. Error: {:?}", result.err());
}

#[test]
fn test_if_else_partial_let_ok() {
    // Policy A: declare in one branch is OK, but variable is NOT guaranteed
    let src = r#"func main(){ if true { let x="1" } else { }; let x="2" }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_ok(), "Partial let should be valid (x not definitely declared). Error: {:?}", result.err());
}

#[test]
fn test_redeclare_after_merge_fail() {
    // x declared in both branches -> definitely declared -> subsequent let is error
    let src = r#"func main(){ if true { let x="1" } else { let x="2" }; let x="3" }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected error for redeclaring guaranteed variable");
    let err = result.unwrap_err();
    assert!(err.contains("already declared"), "Expected 'already declared' error");
}

#[test]
fn test_maybe_declared_usage_fail() {
    // x declared in one branch -> possibly declared but not definitely -> usage error
    let src = r#"func main(){ if true { let x="1" } else { }; print(x) }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected error for using maybe-declared variable");
    let err = result.unwrap_err();
    assert!(err.contains("undefined variable 'x'"), "Expected 'undefined variable' error");
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
    assert!(err.contains("Did you mean to use `set i = ...`?"), "Expected hint in error output");
}

#[test]
fn test_each_line_var_redeclare_is_error() {
    // Can't redeclare each_line variable
    let src = r#"func main(){ let l = "init"; pipe { print("hi"); } | each_line l { } }"#;
    let result = try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "Expected error for redeclaring each_line var");
    let err = result.unwrap_err();
    assert!(err.contains("already declared"), "Expected 'already declared' error, got: {}", err);
    assert!(err.contains("Did you mean to use `set l = ...`?"), "Expected hint in error output");
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

#[test]
fn test_binder_ok_disjoint_if() {
    let (_stdout, stderr) = compile_to_bash_test("fixtures/binder_ok_disjoint_if.sh2");
    assert!(stderr.is_empty(), "Expected no error, got: {}", stderr);
    // x is declared in branches, so it is available after. 
    // No implicit init needed because `let` initialized it in both paths.
    // The usage print(x) implies it knows x is declared.
}

#[test]
fn test_binder_ok_case_wildcard() {
    let (_stdout, stderr) = compile_to_bash_test("fixtures/binder_ok_case_wildcard.sh2");
    assert!(stderr.is_empty(), "Expected no error, got: {}", stderr);
}

#[test]
fn test_binder_err_case_no_wildcard() {
    let (_, stderr) = compile_to_bash_test("fixtures/binder_err_case_no_wildcard.sh2");
    assert!(stderr.contains("undefined variable 'x'"));
}

#[test]
fn test_parse_err_top_level_shim() {
    let (_, stderr) = compile_to_bash_test("fixtures/parse_err_top_level_shim.sh2");
    assert!(stderr.contains("Top-level statements are not allowed"));
}

#[test]
fn test_for_loop_zero_iter_preserves() {
    // 0 iterations -> i preserves value if declared in partial branch
    // At runtime, i="preserved" exists.
    let src = r#"
        func main() {
            if true {
                let i = "preserved"
            } else {
            }
            
            # Policy A: i is not declared on straight-line, so for loop implicit let is allowed.
            # But "preserved" value should persist if loop doesn't run.
            for i in [] { // Empty list
                print("inside")
            }
            
            print($"after: '{i}'")
        }
    "#;
    
    let script = compile_to_shell(src, TargetShell::Bash);
    let (stdout, _, _) = run_bash_script(&script, &[], &[]);
    assert_eq!(stdout.trim(), "after: 'preserved'");
}
