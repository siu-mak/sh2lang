mod common;
use common::*;

struct ShellScript {
    code: String,
    target: TargetShell,
}

impl ShellScript {
    fn run_with_args(&self, args: &[&str]) -> TestResult {
        let shell_bin = match self.target {
            TargetShell::Bash => "bash",
            TargetShell::Posix => "sh",
        };
        let (stdout, stderr, status) = common::run_shell_script(&self.code, shell_bin, &[], args, None, None);
        TestResult { stdout, stderr, status }
    }
}

struct TestResult {
    stdout: String,
    stderr: String,
    status: Option<i32>,
}

impl TestResult {
    fn assert_stdout(&self, expected: &str) -> &Self {
        assert_eq!(self.stdout, expected);
        self
    }
    fn assert_stderr_contains(&self, expected: &str) -> &Self {
        assert!(self.stderr.contains(expected), "Stderr '{}' did not contain '{}'", self.stderr, expected);
        self
    }
    fn assert_success(&self) -> &Self {
        assert_eq!(self.status, Some(0), "Expected success, got status {:?}. Stderr: {}", self.status, self.stderr);
        self
    }
    fn assert_fail(&self) -> &Self {
        assert_ne!(self.status, Some(0), "Expected failure, got success. Stdout: {}", self.stdout);
        self
    }
}

fn compile(script: &str, target: TargetShell) -> ShellScript {
    let code = common::compile_to_shell(script, target);
    ShellScript { code, target }
}

#[test]
fn test_arg_var_success() {
    let script = r#"
    func main() {
        let i = 1;
        print(arg(i));
    }
    "#;
    compile(script, TargetShell::Bash)
        .run_with_args(&["AAA"])
        .assert_stdout("AAA\n")
        .assert_success();

    compile(script, TargetShell::Posix)
        .run_with_args(&["AAA"])
        .assert_stdout("AAA\n")
        .assert_success();
}

#[test]
fn test_arg_var_bounds_zero() {
    let script = r#"
    func main() {
        let i = 0;
        print(arg(i));
    }
    "#;
    // Should fail with runtime error
    compile(script, TargetShell::Bash)
        .run_with_args(&["A"])
        .assert_stderr_contains("index must be an integer >= 1")
        .assert_fail();

    compile(script, TargetShell::Posix)
        .run_with_args(&["A"])
        .assert_stderr_contains("index must be an integer >= 1")
        .assert_fail();
}

#[test]
fn test_arg_var_bounds_out_of_range() {
    let script = r#"
    func main() {
        let i = 2; // Only 1 arg provided
        print(arg(i));
    }
    "#;
    compile(script, TargetShell::Bash)
        .run_with_args(&["A"])
        .assert_stderr_contains("out of range")
        .assert_fail();

    compile(script, TargetShell::Posix)
        .run_with_args(&["A"])
        .assert_stderr_contains("out of range")
        .assert_fail();
}

#[test]
fn test_arg_var_non_numeric() {
    let script = r#"
    func main() {
        let i = "abc";
        print(arg(i));
    }
    "#;
    compile(script, TargetShell::Bash)
        .run_with_args(&["A"])
        .assert_stderr_contains("index must be an integer >= 1") 
        .assert_fail();

    compile(script, TargetShell::Posix)
        .run_with_args(&["A"])
        .assert_stderr_contains("index must be an integer >= 1")
        .assert_fail();
}

#[test]
fn test_arg_var_injection_attempt() {
    let script = r#"
    func main() {
        let i = "1; echo pwned >&2";
        print(arg(i));
    }
    "#;
    // Must NOT execute injected code
    let res = compile(script, TargetShell::Bash)
        .run_with_args(&["SAFE"]);
    
    // Check strict numeric validation caught it
    res.assert_fail();
    res.assert_stderr_contains("index must be an integer >= 1");
    assert!(!res.stderr.contains("pwned"), "Bash injection succeeded!");

    let res_posix = compile(script, TargetShell::Posix)
        .run_with_args(&["SAFE"]);
    
    res_posix.assert_fail();
    res_posix.assert_stderr_contains("index must be an integer >= 1");
    assert!(!res_posix.stderr.contains("pwned"), "POSIX injection succeeded!");
}
