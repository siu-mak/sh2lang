use crate::common::*;

mod common;

#[test]
fn test_sh_raw_positional_args_forwarding() {
    // Check that sh("...") receives the current function's arguments as "$@"
    let src = r#"
    func echo_arg_1() {
        sh("printf 'Arg1: %s' \"$1\"")
    }
    
    func main() {
        echo_arg_1("hello")
    }
    "#;
    let compiled = try_compile_to_shell(src, TargetShell::Bash).expect("Compilation failed");
    let (stdout, _, status) = run_bash_script(&compiled, &[], &[]);
    assert_eq!(status, 0);
    assert_eq!(stdout.trim(), "Arg1: hello");
}

#[test]
fn test_sh_raw_all_args_forwarding() {
    // Check that sh("...") receives all arguments as "$@"
    let src = r#"
    func echo_all() {
        sh("echo \"All: $@\"")
    }
    
    func main() {
        echo_all("one", "two", "three")
    }
    "#;
    let compiled = try_compile_to_shell(src, TargetShell::Bash).expect("Compilation failed");
    let (stdout, _, status) = run_bash_script(&compiled, &[], &[]);
    assert_eq!(status, 0);
    assert!(stdout.contains("All: one two three"));
}

#[test]
fn test_run_sh_c_explicit_args() {
    // Check that run("sh", "-c", cmd, arg1, arg2) passes explicit args, masking implicit ones
    let src = r#"
    func main() {
        // Implicit args (from main) should be ignored/shadowed by explicit ones
        run("sh", "-c", "echo \"$1 $2\"", "explicit1", "explicit2")
    }
    "#;
    // We pass some args to the script itself, to verify they are NOT used
    assert_exec_matches_fixture_target("run_sh_c_explicit_args", TargetShell::Bash);
}
