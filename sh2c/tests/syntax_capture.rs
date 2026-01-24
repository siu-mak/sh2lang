mod common;

fn assert_compile_fail(src: &str, expected_msg_part: &str) {
    match common::try_compile_to_shell(src, common::TargetShell::Bash) {
        Ok(_) => panic!("Expected compilation failure for invalid source, but it succeeded"),
        Err(msg) => {
            assert!(
                msg.contains(expected_msg_part),
                "Expected diagnostic message containing '{}', got '{}'",
                expected_msg_part,
                msg
            );
        }
    }
}

#[test]
fn test_capture_allow_fail_execution() {
    common::assert_exec_matches_fixture("capture_allow_fail");
}

#[test]
fn test_capture_allow_fail_invalid_usage() {
    // Should fail if not assigned to let
    // E.g. print(capture(..., allow_fail=true))
    let src = r#"
        func main() {
            print(capture(run("sh", "-c", "exit 1"), allow_fail=true))
        }
    "#;
    assert_compile_fail(src, "capture(..., allow_fail=true) is only allowed in 'let' assignment");
}

#[test]
fn test_capture_options_parsing() {
    // Test that allow_fail=false works (parsed but treated correctly)
    let res = common::try_compile_to_shell(r#"
        func main() {
            let s = capture(run("ls"), allow_fail=false)
        }
    "#, common::TargetShell::Bash);
    assert!(res.is_ok(), "Failed to compile allow_fail=false: {:?}", res.err());
    
    // Test unknown option
    assert_compile_fail(r#"
        func main() {
            let s = capture(run("ls"), unknown=true)
        }
    "#, "Unknown option 'unknown'");
    
    // Test options in $() syntax forbidden
    assert_compile_fail(r#"
        func main() {
            let s = $(run("ls"), allow_fail=true)
        }
    "#, "run options like allow_fail=... are not supported inside command substitution");
}

#[test]
fn test_capture_allow_fail_non_bool() {
    // Test that allow_fail must be a boolean literal
    assert_compile_fail(r#"
        func main() {
            let s = capture(run("ls"), allow_fail=123)
        }
    "#, "allow_fail must be true/false literal");
    
    // Test with variable (also invalid)
    assert_compile_fail(r#"
        func main() {
            let x = true
            let s = capture(run("ls"), allow_fail=x)
        }
    "#, "allow_fail must be true/false literal");
}
