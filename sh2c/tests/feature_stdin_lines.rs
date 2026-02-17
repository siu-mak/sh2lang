
mod common;
use common::{try_compile_to_shell, TargetShell};

fn check_stdin_lines(src: &str, stdin: &str, expected_stdout: &str) {
    common::run_test_in_targets_with_stdin("stdin_lines_test", src, stdin, expected_stdout);
}

#[test]
fn test_stdin_lines_basic() {
    let src = r#"
        func main() {
            let i = 0
            for line in stdin_lines() {
                print("Line " & i & ": " & line)
                set i = i + 1
            }
        }
    "#;
    let stdin = "first\nsecond\nthird\n";
    let expected = "Line 0: first\nLine 1: second\nLine 2: third\n";
    check_stdin_lines(src, stdin, expected);
}

#[test]
fn test_stdin_lines_no_trailing_newline() {
    let src = r#"
        func main() {
            for line in stdin_lines() {
                print("Got: " & line)
            }
        }
    "#;
    let stdin = "one\ntwo"; // "two" has no newline
    let expected = "Got: one\nGot: two\n";
    check_stdin_lines(src, stdin, expected);
}

#[test]
fn test_stdin_lines_empty_input() {
    // Correct test case: relying on loop variable persistence without prior declaration
    let src_persist = r#"
        func main() {
            for line in stdin_lines() {
                print("Should not run")
            }
            // Policy A: Loop variable persists and is definitely assigned (empty string if 0 iterations)
            if line == "" {
                print("Empty as expected")
            } else {
                print("Unexpected: '" & line & "'")
            }
        }
    "#;
    let stdin = "";
    let expected = "Empty as expected\n";
    check_stdin_lines(src_persist, stdin, expected);
}

#[test]
fn test_stdin_lines_whitespace_preservation() {
    let src = r#"
        func main() {
            for line in stdin_lines() {
                print("'" & line & "'")
            }
        }
    "#;
    let stdin = "  leading\ntrailing  \n  both  \n";
    let expected = "'  leading'\n'trailing  '\n'  both  '\n";
    check_stdin_lines(src, stdin, expected);
}

#[test]
fn test_stdin_lines_compile_fail_expression() {
    let src = r#"
        func main() {
            let x = stdin_lines()
        }
    "#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    assert!(err.contains("stdin_lines() can only be used as the iterable in a for-loop"), "Got: {}", err);
    
    // Check Posix too
    let res = try_compile_to_shell(src, TargetShell::Posix);
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    assert!(err.contains("stdin_lines() can only be used as the iterable in a for-loop"), "Got: {}", err);
}

#[test]
fn test_stdin_lines_compile_fail_list() {
    let src = r#"
        func main() {
            for x in [stdin_lines()] {
                continue
            }
        }
    "#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    assert!(err.contains("stdin_lines() can only be used as the iterable in a for-loop"), "Got: {}", err);
}

#[test]
fn test_stdin_lines_compile_fail_args() {
    let src = r#"
        func main() {
            for x in stdin_lines("arg") {
                continue
            }
        }
    "#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    assert!(err.contains("stdin_lines() takes no arguments"), "Got: {}", err);
}

#[test]
fn test_stdin_lines_mixed_with_args() {
    let src = r#"
        func main() {
            let x = 1 
            // Ensures parser doesn't swallow other things
            for line in stdin_lines() {
                 print(line)
            }
        }
    "#;
    let stdin = "ok\n";
    let expected = "ok\n";
    check_stdin_lines(src, stdin, expected);
}
