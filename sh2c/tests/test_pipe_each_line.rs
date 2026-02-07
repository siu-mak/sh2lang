mod common;
use common::*;

#[test]
fn test_pipe_each_line_streaming() {
    let sh2 = r#"
    func main() {
        print("start")
        run("printf", "line1\n\nline3") | each_line l {
            print($"got: '{l}'")
        }
        print("end")
    }
    "#;
    
    // We expect line2 to be empty string. line3 has no newline in input?
    // printf "line1\n\nline3" -> line1, , line3 (no newline at end)
    // The loop should handle missing final newline.
    
    let expected = r#"start
got: 'line1'
got: ''
got: 'line3'
end
"#;

    run_test_bash_only("each_line_streaming", sh2, expected);
}

#[test]
fn test_pipe_each_line_status_propagation() {
    let sh2 = r#"
    func main() {
        run("sh", "-c", "echo out; exit 7", allow_fail=true) | each_line l {
            print($"got: {l}")
        }
        print($"status: {status()}")
    }
    "#;

    let expected = r#"got: out
status: 7
"#;

    run_test_bash_only("each_line_status", sh2, expected);
}

#[test]
fn test_pipe_each_line_scope_persistence() {
    let sh2 = r#"
    func main() {
        let cnt = 0
        run("seq", "3") | each_line unused {
            cnt = cnt + 1
        }
        print($"count: {cnt}")
    }
    "#;

    let expected = "count: 3\n";

    run_test_bash_only("each_line_scope", sh2, expected);
}

#[test]
fn test_pipe_each_line_empty_producer() {
    let sh2 = r#"
    func main() {
        print("start")
        run("true") | each_line l {
            print("FAIL: $l")
        }
        print("end")
    }
    "#;
    
    let expected = "start\nend\n";
    
    run_test_bash_only("each_line_empty", sh2, expected);
}

#[test]
fn test_pipe_each_line_multi_segment_status() {
    // Validates that status from failing segment in multi-stage producer is captured
    // We expect the pipeline to fail with 7 (pipefail)
    let sh2 = r#"
    func main() {
        try {
            run("sh", "-c", "echo line; exit 7") | run("cat") | each_line l {
                print($"got: {l}")
            }
        } catch {
            print($"caught: {status()}")
        }
    }
    "#;
    // With pipefail: first exits 7, cat succeeds but pipeline fails with 7
    let expected = "got: line\ncaught: 7\n";
    run_test_bash_only("each_line_multi_status", sh2, expected);
}

#[test]
fn test_pipe_each_line_allow_fail_status() {
    // Ticket 7: allow_fail should prevent abort but status() should reflect the failure
    let sh2 = r#"
    func main() {
        print("start")
        run("sh", "-c", "exit 7", allow_fail=true) | each_line l {
             print("never")
        }
        print($"status: {status()}")
    }
    "#;
    
    // allow_fail prevents abort. exit 7 is preserved in status().
    // Empty output from producer means body doesn't run.
    let expected = "start\nstatus: 7\n";
    
    run_test_bash_only("each_line_allow_fail", sh2, expected);
}

#[test]
fn test_pipe_each_line_posix_error() {
    let sh2 = r#"func main() { run("ls") | each_line l { print(l) } }"#;
    let result = try_compile_to_shell(sh2, TargetShell::Posix);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("only supported in Bash"), 
        "Expected error about Bash-only feature");
}

#[test]
fn test_pipe_each_line_semantics_scoping_fail() {
    // Loop variable 'l' should NOT be definitely declared after loop
    let sh2 = r#"
    func main() {
        run("ls") | each_line l {
            print(l)
        }
        print(l) // Error: l might be uninitialized if loop didn't run
    }
    "#;
    let result = try_compile_to_shell(sh2, TargetShell::Bash);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("undefined variable 'l'"), 
        "Expected error about undeclared variable usage after loop");
}

#[test]
fn test_pipe_each_line_semantics_scoping_ok() {
    // Loop variable 'l' declared before loop should be accessible after
    let sh2 = r#"
    func main() {
        let l = "init"
        run("ls") | each_line m {
            print(m)
        }
        print(l) // OK: l was definitely declared before
    }
    "#;
    let result = try_compile_to_shell(sh2, TargetShell::Bash);
    assert!(result.is_ok(), "Expected valid compilation when variable is pre-declared");
}
