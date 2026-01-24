mod common;
use common::TargetShell;

// Basic tests - verify compilation succeeds and output is reasonable
// Note: These don't actually run sudo, they just verify the generated shell code

#[test]
fn test_sudo_basic_compiles() {
    let src = r#"
func main() {
    let out = capture(sudo("echo", "hi"))
    print(out)
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_ok(), "sudo() should compile. Got: {:?}", result);
    let shell = result.unwrap();
    assert!(shell.contains("sudo"), "Output should contain sudo");
    assert!(shell.contains("--"), "Output should contain -- separator");
}

#[test]
fn test_sudo_with_user_compiles() {
    let src = r#"
func main() {
    let out = capture(sudo("id", user="root"))
    print(out)
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_ok(), "sudo(user=) should compile. Got: {:?}", result);
    let shell = result.unwrap();
    assert!(shell.contains("-u"), "Output should contain -u flag");
    assert!(shell.contains("root"), "Output should contain user name");
}

#[test]
fn test_sudo_with_n_flag_compiles() {
    let src = r#"
func main() {
    let out = capture(sudo("true", n=true))
    print(out)
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_ok(), "sudo(n=true) should compile. Got: {:?}", result);
    let shell = result.unwrap();
    assert!(shell.contains("-n"), "Output should contain -n flag");
}

#[test]
fn test_sudo_with_env_keep_compiles() {
    let src = r#"
func main() {
    let out = capture(sudo("env", env_keep=["http_proxy", "https_proxy"]))
    print(out)
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_ok(), "sudo(env_keep=) should compile. Got: {:?}", result);
    let shell = result.unwrap();
    assert!(shell.contains("--preserve-env=http_proxy,https_proxy"), "Output should contain --preserve-env");
}

// Compile-fail tests

#[test]
fn test_sudo_no_args_error() {
    let src = r#"
func main() {
    let out = capture(sudo())
    print(out)
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);
    if let Ok(res) = &result {
        println!("Unexpected success: {:?}", res);
    }
    assert!(result.is_err(), "sudo() with no args should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("requires at least one positional argument"),
        "Error should mention missing command. Got: {}",
        err
    );
}

#[test]
fn test_sudo_unknown_option_error() {
    let src = r#"
func main() {
    let out = capture(sudo("echo", xyz=true))
    print(out)
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "sudo() with unknown option should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("unknown sudo() option") && err.contains("xyz"),
        "Error should mention unknown option. Got: {}",
        err
    );
}

#[test]
fn test_sudo_duplicate_option_error() {
    let src = r#"
func main() {
    let out = capture(sudo("echo", n=true, n=false))
    print(out)
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "sudo() with duplicate option should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("more than once"),
        "Error should mention duplicate. Got: {}",
        err
    );
}

#[test]
fn test_sudo_n_not_bool_error() {
    let src = r#"
func main() {
    let out = capture(sudo("echo", n=1))
    print(out)
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "sudo(n=1) should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("must be a boolean literal"),
        "Error should mention bool. Got: {}",
        err
    );
}

#[test]
fn test_sudo_env_keep_not_list_error() {
    let src = r#"
func main() {
    let out = capture(sudo("echo", env_keep="var"))
    print(out)
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "sudo(env_keep=string) should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("must be a list"),
        "Error should mention list. Got: {}",
        err
    );
}

#[test]
fn test_sudo_allow_fail_expr_error() {
    let src = r#"
func main() {
    let out = capture(sudo("echo", allow_fail=true))
    print(out)
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err(), "sudo() expr form should not allow allow_fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("only valid on statement-form"),
        "Error should mention statement-form. Got: {}",
        err
    );
}

#[test]
fn test_sudo_stmt_allow_fail_compiles() {
    let src = r#"
func main() {
    sudo("echo", "might fail", allow_fail=true)
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_ok(), "sudo(allow_fail=true) in stmt context should compile. Got: {:?}", result);
    let shell = result.unwrap();
    assert!(shell.contains("sudo"), "Output should contain sudo");
    assert!(shell.contains("--"), "Output should contain -- separator");
}

#[test]
// #[ignore] // TODO: fix parser regression with run after variable
fn test_repro_run_after_let_concat() {
    let src = r#"
func main() {
    let space_str = "hello world"
    let quote_str = "he said \"hello\""
    let pipe_str = "a | b"
    
    let combined = space_str
    run("echo", combined)
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);

    assert!(result.is_ok(), "run after let concat (complex) should compile. Got: {:?}", result);
}


#[test]
fn test_sudo_mixed_ordering_compiles() {
    // named then positional
    let src1 = r#"
func main() {
    sudo(user="root", "id")
}
"#;
    let res1 = common::try_compile_to_shell(src1, TargetShell::Bash);
    assert!(res1.is_ok(), "sudo(user=..., cmd) should compile");
    assert!(res1.unwrap().contains("-u"), "Should contain -u");

    // positional then named
    let src2 = r#"
func main() {
    sudo("id", user="root")
}
"#;
    let res2 = common::try_compile_to_shell(src2, TargetShell::Bash);
    assert!(res2.is_ok(), "sudo(cmd, user=...) should compile");
    assert!(res2.unwrap().contains("-u"), "Should contain -u");


    // mixed: named, positional, named
    let src3 = r#"
func main() {
    sudo(n=true, "ls", user="root")
}
"#;
    let res3 = common::try_compile_to_shell(src3, TargetShell::Bash);
    assert!(res3.is_ok(), "sudo(n=true, cmd, user=...) should compile");
    let shell = res3.unwrap();
    // generated shell is quoted: 'sudo' '-n' '-u' 'root' '--' 'ls'
    assert!(shell.contains("-u") && shell.contains("root"), "Should contain -u root");
    assert!(shell.contains("-n"), "Should contain -n");
    assert!(shell.contains("ls"), "Should contain cmd");
}

#[test]
fn test_sudo_expr_allow_fail_error_msg() {
    let src = r#"
func main() {
    let x = capture(sudo("ls", allow_fail=true))
}
"#;
    let result = common::try_compile_to_shell(src, TargetShell::Bash);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("allow_fail is only valid on statement-form sudo(...)"));
    assert!(err.contains("use capture(sudo(...), allow_fail=true)"));
}

#[test]
fn test_sudo_mixed_ordering_shorthand() {
    let src = r#"
func main() {
    // Shorthand command substitution
    let x = $(sudo("id", user="root"))
}
"#;
    let res = common::try_compile_to_shell(src, TargetShell::Bash);
    assert!(res.is_ok(), "shorthand $(sudo(cmd, user=...)) should compile");
    let shell = res.unwrap();
    assert!(shell.contains("-u") && shell.contains("root"), "Should contain -u root flag");
}
