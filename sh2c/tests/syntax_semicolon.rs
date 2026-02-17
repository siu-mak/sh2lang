
mod common;
use common::{try_compile_to_shell, TargetShell};


fn check_exec(src: &str, expected_contains: &[&str]) {
    let script = common::compile_to_bash(src);
    // Use common::run_bash_script which handles temporary directory creation etc.
    let (stdout, stderr, status) = common::run_bash_script(&script, &[], &[]);
    if status != Some(0) {
        panic!("Script failed with status {:?}.\nStderr: {}\nScript:\n{}", status, stderr, script);
    }
    for expected in expected_contains {
        assert!(stdout.contains(expected), "Output missing '{}'. Got:\n{}", expected, stdout);
    }
}

#[test]
fn test_semicolon_multi_stmt() {
    let src = r#"
        func main() {
            print("a"); print("b")
        }
    "#;
    check_exec(src, &["a", "b"]);
}

#[test]
fn test_semicolon_trailing() {
    let src = r#"
        func main() {
            print("a");
        }
    "#;
    check_exec(src, &["a"]);
}

#[test]
fn test_semicolon_multiple_empty() {
    let src = r#"
        func main() {
            print("a");; print("b");;;; print("c")
        }
    "#;
    check_exec(src, &["a", "b", "c"]);
}

#[test]
fn test_semicolon_subshell() {
    let src = r#"
        func main() {
            subshell {
                print("a"); print("b")
            }
        }
    "#;
    check_exec(src, &["a", "b"]);
}

#[test]
fn test_semicolon_case_arms() {
    let src = r#"
        func main() {
            case "x" {
                "x" => { print("yes"); };
                _ => { print("no"); }
            }
        }
    "#;
    check_exec(src, &["yes"]);
}

#[test]
fn test_semicolon_in_expr_fail() {
    let src = r#"
        func main() {
            let x = ;
        }
    "#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    assert!(res.is_err(), "Should fail with semicolon as expression start");
    let err = res.err().unwrap();
    // Expect specific error checking logic
    assert!(err.contains("Unexpected statement separator"), 
            "Error should cover semicolon usage in expr, got: {}", err);

    let src2 = r#"
        func main() {
            let x = (1; 2)
        }
    "#;
    let res2 = try_compile_to_shell(src2, TargetShell::Bash);
    assert!(res2.is_err());
    let err2 = res2.err().unwrap();
    // This now gives usage specific error
    assert!(err2.contains("Unexpected statement separator"), "Got: {}", err2);
}
