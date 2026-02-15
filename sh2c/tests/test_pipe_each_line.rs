mod common;
use common::*;

#[test]
fn test_each_line_posix_fails() {
    let src = r#"
        func main() {
            run("echo", "foo") | each_line l { print(l) }
        }
    "#;
    let err = try_compile_to_shell(src, TargetShell::Posix).unwrap_err();
    assert!(err.contains("each_line is only supported in Bash"), "Unexpected error: {}", err);
}

#[test]
fn test_each_line_bash_zero_iter_init() {
    // 0 iterations (empty input) -> l should be ""
    let src = r#"
        func main() {
            run("true") | each_line l { 
                print($"inside: {l}") 
            }
            // l should be initialized to "" if loop didn't run
            print($"after: '{l}'")
        }
    "#;
    let script = compile_to_shell(src, TargetShell::Bash);
    let (stdout, _, _) = run_bash_script(&script, &[], &[]);
    assert_eq!(stdout.trim(), "after: ''");
}

#[test]
fn test_each_line_bash_zero_iter_preserves() {
    // 0 iterations -> l preserves value if declared in partial branch
    // At runtime, l="preserved" exists.
    let src = r#"
        func main() {
            if true {
                let l = "preserved"
            } else {
                // empty else ensures 'l' is NOT definitely assigned on all paths
            }
            
            # Now l is not declared in the straight-line path (Policy A), 
            # so each_line can declare it (implicit let).
            # But at runtime, l="preserved" exists and should be picked up.
            
            run("true") | each_line l {
                print($"inside: {l}")
            }
            
            # l persists
            print($"after: '{l}'")
        }
    "#;
    
    let script = compile_to_shell(src, TargetShell::Bash);
    let (stdout, _, _) = run_bash_script(&script, &[], &[]);
    assert_eq!(stdout.trim(), "after: 'preserved'");
}

#[test]
fn test_each_line_bash_iterates() {
    let src = r#"
        func main() {
            run("printf", "line1\nline2\n") | each_line l {
                print($"got: {l}")
            }
            print($"last: {l}")
        }
    "#;
    let script = compile_to_shell(src, TargetShell::Bash);
    let (stdout, _, _) = run_bash_script(&script, &[], &[]);
    let expected = "got: line1\ngot: line2\nlast: line2";
    assert_eq!(stdout.trim(), expected);
}

#[test]
fn test_each_line_streaming_correctness() {
    // Case a) Streaming correctness: preserves empty lines, handles missing trailing newline
    // Input: "line1\n\nline3" (no trailing newline at end)
    // Expected iterations: "line1", "", "line3"
    let src = r#"
        func main() {
            run("printf", "line1\n\nline3") | each_line l {
                print($"line: '{l}'")
            }
        }
    "#;
    let script = compile_to_shell(src, TargetShell::Bash);
    let (stdout, _, _) = run_bash_script(&script, &[], &[]);
    let expected = "line: 'line1'\nline: ''\nline: 'line3'";
    assert_eq!(stdout.trim(), expected);
}

#[test]
fn test_each_line_status_propagation() {
    // Case b) Status propagation: producer exit status is reflected by status() after loop
    // Using sh -c to adhere to strict compilation, and check failure propagation.
    let src = r#"
        func main() {
            run("sh", "-c", "echo out; exit 7", allow_fail=true) | each_line l {
                print($"got: {l}")
            }
            print($"status: {status()}")
        }
    "#;
    let script = compile_to_shell(src, TargetShell::Bash);
    let (stdout, _, _) = run_bash_script(&script, &[], &[]);
    // Output should contain "got: out" and "status: 7"
    assert!(stdout.contains("got: out"));
    assert!(stdout.contains("status: 7"));
}

#[test]
fn test_each_line_allow_fail_interaction() {
    // Case c) allow_fail interaction
    // Producer exits 7 with allow_fail=true.
    let src = r#"
        func main() {
            print("start")
            run("sh", "-c", "exit 7", allow_fail=true) | each_line l {
                print("inside")
            }
            print($"end status: {status()}")
        }
    "#;
    let script = compile_to_shell(src, TargetShell::Bash);
    let (stdout, _, _) = run_bash_script(&script, &[], &[]);
    assert_eq!(stdout.trim(), "start\nend status: 7");
}

#[test]
fn test_each_line_multistage_pipefail() {
    // Case d) Multi-stage producer / pipefail checks
    // The middle stage succeeds, but first stage fails. 
    // Bash with pipefail should propagate 7.
    let src = r#"
        func main() {
            try {
                // First stage fails with 7. pipefail should make the whole pipe fail with 7.
                run("sh", "-c", "echo line; exit 7") | run("cat") | each_line l {
                    print($"got: {l}")
                }
            } catch {
                print($"caught: {status()}")
            }
        }
    "#;
    let script = compile_to_shell(src, TargetShell::Bash);
    let (stdout, _, _) = run_bash_script(&script, &[], &[]);
    // Should print "got: line" then "caught: 7"
    assert!(stdout.contains("got: line"));
    assert!(stdout.contains("caught: 7"));
}
