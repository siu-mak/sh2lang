mod common;
use common::*;

#[test]
fn test_unbound_variable_remains_literal() {
    let src = r#"
    func main() {
        print("A: $FOO")
        // Braced form
        print("B: ${FOO}")
    }
    "#;
    let compiled = try_compile_to_shell(src, TargetShell::Bash).expect("Compilation failed");
    
    // We expect the compiled bash to be safe.
    // Checking exact string is fragile due to concatenation optimizations or splitting.
    // assert!(compiled.contains("'A: $FOO'"));
    // assert!(compiled.contains("'B: ${FOO}'"));
    
    // Heuristic: Ensure no double-single-quote artifacts
    assert!(!compiled.contains("''$FOO''"));
    assert!(!compiled.contains("''${FOO}''"));
    
    // Run it with FOO in env
    // We expect output to be literal $FOO, not expanded
    let (stdout, _, _) = run_bash_script(&compiled, &[("FOO", "EXPANDED")], &[]);
    assert!(stdout.contains("A: $FOO"));
    assert!(!stdout.contains("EXPANDED"));
    assert!(stdout.contains("B: ${FOO}"));
}

#[test]
fn test_bound_variable_interpolates_only_with_prefix() {
    let src = r#"
    func main() {
        let name = "World"
        // Normal string: NO interpolation
        print("Literal: $name")
        // Interpolated string: YES interpolation
        print($"Interp: $name")
        print($"InterpBrace: ${name}")
    }
    "#;
    let compiled = try_compile_to_shell(src, TargetShell::Bash).expect("Compilation failed");
    
    // Run it
    let (stdout, _, _) = run_bash_script(&compiled, &[], &[]);
    assert!(stdout.contains("Literal: $name"));
    assert!(stdout.contains("Interp: World"));
    assert!(stdout.contains("InterpBrace: World"));
}

#[test]
fn test_loop_var_interpolation() {
    let src = r#"
    func main() {
        let items = ["a", "b"]
        for x in items {
            print($"Item: $x")
        }
    }
    "#;
     let compiled = try_compile_to_shell(src, TargetShell::Bash).expect("Compilation failed");
    let (stdout, _, _) = run_bash_script(&compiled, &[], &[]);
    assert!(stdout.contains("Item: a"));
    assert!(stdout.contains("Item: b"));
}

#[test]
fn test_param_interpolation() {
    let src = r#"
    func main() {
        greet("Alice")
    }
    func greet(name) {
        print($"Hi $name")
    }
    "#;
    let compiled = try_compile_to_shell(src, TargetShell::Bash).expect("Compilation failed");
    let (stdout, _, _) = run_bash_script(&compiled, &[], &[]);
    assert!(stdout.contains("Hi Alice"));
}

#[test]
fn test_map_loop_interpolation() {
    let src = r#"
    func main() {
        let m = {"k1": "v1"}
        for(k, v) in m {
            print($"$k=$v")
        }
    }
    "#;
    // Map loop requires map support (Bash only for now usually)
    let compiled = try_compile_to_shell(src, TargetShell::Bash).expect("Compilation failed");
    let (stdout, _, _) = run_bash_script(&compiled, &[], &[]);
    assert!(stdout.contains("k1=v1"));
}

#[test]
fn test_sh_raw_expansion() {
    // This tests that sh(...) passes the string preserving $ so child shell can expand it.
    let src = r#"
    func main() {
        // sh(...) should still allow expansion in the CHILD shell
        // Because $FOO is unbound in sh2, it's literal '$FOO'.
        // Passed to sh -c '... $FOO ...'.
        sh("echo RAW: $FOO")
    }
    "#;
    
    let compiled = try_compile_to_shell(src, TargetShell::Bash).expect("Compilation failed");
    
    // Check codegen: sh -c 'echo RAW: $FOO'
    // Or similar structure.
    
    let (stdout, _, _) = run_bash_script(&compiled, &[("FOO", "EXPANDED")], &[]);
    assert!(stdout.contains("RAW: EXPANDED"));
    
    // Verify codegen quality (no suspicious quoting like "''$FOO''")
    assert!(!compiled.contains("''$FOO''"));
}

#[test]
fn test_nested_string_interpolation() {
    // Unbound var inside interpolated string with bound var
    let src = r#"
    func main() {
        let x = "bound"
        // $"..." enables interpolation. Unbound vars expand to empty in Bash.
        print($"Mixed: $x and $UNBOUND")
    }
    "#;
    let compiled = try_compile_to_shell(src, TargetShell::Bash).expect("Compilation failed");
    let (stdout, _, _) = run_bash_script(&compiled, &[("UNBOUND", "should_not_expand")], &[]);
    // If UNBOUND is in env (passed in run_bash_script), it expands!
    // Wait, the test passes env "UNBOUND"="should_not_expand".
    // So it SHOULD expand to "should_not_expand".
    assert!(stdout.contains("Mixed: bound and should_not_expand"));
}

#[test]
fn test_dpkg_surrogate() {
    // Regression test for dpkg-query usage:
    // run("dpkg-query", "-W", "-f", "${Package}\n")
    // Should preserve ${Package} literally.
    
    // We use python3 to verify argv[1] is passed literally.
    let src = r#"
    func main() {
        // unbound ${Package}, inside literal string
        run("python3", "-c", "import sys; print(sys.argv[1])", "${Package}")
    }
    "#;
    let compiled = try_compile_to_shell(src, TargetShell::Bash).expect("Compilation failed");
    
    // Run with Package=BAD in env.
    // If unsafe: expands to BAD.
    // If safe: prints ${Package}
    let (stdout, _, _) = run_bash_script(&compiled, &[("Package", "BAD")], &[]);
    assert!(stdout.contains("${Package}"));
    assert!(!stdout.contains("BAD"));
}

#[test]
fn test_run_sh_c_expansion() {
    // run("sh", "-c", "echo RAW: $FOO")
    // Must expand $FOO in child shell even if unbound in sh2.
    let src = r#"
    func main() {
        run("sh", "-c", "echo RAW: $FOO")
    }
    "#;
    let compiled = try_compile_to_shell(src, TargetShell::Bash).expect("Compilation failed");
    let (stdout, _, _) = run_bash_script(&compiled, &[("FOO", "EXPANDED")], &[]);
    assert!(stdout.contains("RAW: EXPANDED"));
}

#[test]
fn test_mixed_interpolation() {
    let src = r#"
    func main() {
        let x = "bound"
        // Mix: "Bound: $x, Unbound: $y" using interp string
        // $y is not in env, so it expands to empty
        print($"Bound: $x, Unbound: $y")
    }
    "#;
    let compiled = try_compile_to_shell(src, TargetShell::Bash).expect("Compilation failed");
    let (stdout, _, _) = run_bash_script(&compiled, &[], &[]);
    assert!(stdout.contains("Bound: bound, Unbound: "));
}

#[test]
fn test_sh_raw_allow_fail() {
    // run("sh", "-c", "exit 7", allow_fail=true)
    // Child exit code should not crash the script, and status() should be 7.
    let src = r#"
    func main() {
        run("sh", "-c", "exit 7", allow_fail=true)
        print("Status: " + status())
    }
    "#;
    let compiled = try_compile_to_shell(src, TargetShell::Bash).expect("Compilation failed");
    
    let (stdout, _, status) = run_bash_script(&compiled, &[], &[]);
    assert_eq!(status, 0); // Script itself should succeed
    assert!(stdout.contains("Status: 7"));
}
