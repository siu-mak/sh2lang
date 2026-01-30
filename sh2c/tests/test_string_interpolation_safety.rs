mod common;

#[test]
fn test_unbound_variable_is_literal() {
    // $FOO should be treated as literal if FOO is not bound in sh2
    let src = r#"
        func main() {
            print("$FOO")
        }
    "#;
    
    // We run with FOO=EXPANDED in the environment.
    // If sh2c emits "$FOO" (quoted), bash prints $FOO.
    // If sh2c emits $FOO (unquoted), bash prints EXPANDED.
    let bash = common::compile_to_bash(src);
    let (stdout, _, _) = common::run_bash_script(&bash, &[("FOO", "EXPANDED")], &[]);
    assert_eq!(stdout.trim(), "$FOO");
}

#[test]
fn test_unbound_braced_variable_preserved() {
    // ${Package} should be preserved exactly if Package is not bound
    let src = r#"
        func main() {
            print("${Package}")
        }
    "#;
    
    let bash = common::compile_to_bash(src);
    let (stdout, _, _) = common::run_bash_script(&bash, &[("Package", "EXPANDED")], &[]);
    assert_eq!(stdout.trim(), "${Package}");
}

#[test]
fn test_bound_variable_interpolates() {
    // $x should be interpolated if x is bound
    let src = r#"
        func main() {
            let x = "val"
            print("High $x")
        }
    "#;
    
    let bash = common::compile_to_bash(src);
    let (stdout, _, _) = common::run_bash_script(&bash, &[], &[]);
    assert_eq!(stdout.trim(), "High val");
}

#[test]
fn test_escaped_dollar_is_literal() {
    // \$FOO should be literal $FOO
    let src = r#"
        func main() {
            print("\$FOO")
        }
    "#;
    
    let bash = common::compile_to_bash(src);
    let (stdout, _, _) = common::run_bash_script(&bash, &[("FOO", "EXPANDED")], &[]);
    assert_eq!(stdout.trim(), "$FOO");
}

#[test]
fn test_function_param_interpolates() {
    let src = r#"
        func greet(name) {
            print("Hello $name")
        }
        func main() {
            greet("World")
        }
    "#;
    
    let bash = common::compile_to_bash(src);
    let (stdout, _, _) = common::run_bash_script(&bash, &[], &[]);
    assert_eq!(stdout.trim(), "Hello World");
}

#[test]
fn test_loop_var_interpolates() {
    let src = r#"
        func main() {
            for i in ["a", "b"] {
                print("Item $i")
            }
        }
    "#;
    
    let bash = common::compile_to_bash(src);
    let (stdout, _, _) = common::run_bash_script(&bash, &[], &[]);
    assert_eq!(stdout.trim(), "Item a\nItem b");
}

#[test]
fn test_literal_in_arithmnetic_fails() {
    // We use an unbound variable interpolation. The parser sees StringInterpVar,
    // so lower.rs doesn't convert to Concat (it only checks for ExprKind::Literal).
    // But lower_expr returns Val::Literal("$UNBOUND"), which then enters Arith.
    // This should trigger the safety check in lowering.
    let src = "func main() { let x = 1 + \"$UNBOUND\"; }";
    let res = common::try_compile_to_shell(src, common::TargetShell::Bash);
    match res {
        Ok(_) => panic!("Expected compilation failure, but succeeded"),
        Err(msg) => {
            assert!(msg.contains("string literal '$UNBOUND' not allowed in arithmetic context"), "Unexpected error: {}", msg);
        }
    }
}

#[test]
fn test_raw_sh_expands() {
    // raw sh("...") should still allow expansion in the subshell
    let src = r#"
        func main() {
            sh("echo $FOO")
        }
    "#;
    let bash = common::compile_to_bash(src);
    let (stdout, _, _) = common::run_bash_script(&bash, &[("FOO", "EXPANDED")], &[]);
    assert_eq!(stdout.trim(), "EXPANDED");
}

#[test]
fn test_run_sh_c_expands() {
    // run("sh", "-c", "...") should still pass the string as-is, expanding in the subshell
    let src = r#"
        func main() {
            run("sh", "-c", "echo $FOO")
        }
    "#;
    let bash = common::compile_to_bash(src);
    let (stdout, _, _) = common::run_bash_script(&bash, &[("FOO", "EXPANDED")], &[]);
    assert_eq!(stdout.trim(), "EXPANDED");
}
