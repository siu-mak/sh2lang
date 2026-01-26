
mod common;
use common::{compile_to_bash, try_compile_to_shell, TargetShell};

#[test]
fn test_missing_whitespace_env_concat() {
    let src = r#"
        func main() {
            let x = env.HOME&"/x"
        }
    "#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(err.contains("The & operator requires whitespace: env.HOME & \"/x\""), "Got: {}", err);
}

#[test]
fn test_missing_whitespace_around_concat() {
    let src = r#"
        func main() {
            let x = "a"&"b"
        }
    "#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(err.contains("The & operator requires whitespace: env.HOME & \"/x\""), "Got: {}", err);
}

#[test]
fn test_missing_whitespace_before_concat() {
    let src = r#"
        func main() {
            let x = "a"& "b"
        }
    "#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(err.contains("The & operator requires whitespace: env.HOME & \"/x\""), "Got: {}", err);
}

#[test]
fn test_missing_whitespace_after_concat() {
    let src = r#"
        func main() {
            let x = "a" &"b"
        }
    "#;
    let res = try_compile_to_shell(src, TargetShell::Bash);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(err.contains("The & operator requires whitespace: env.HOME & \"/x\""), "Got: {}", err);
}
