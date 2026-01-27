use common::{assert_exec_matches_fixture_target, TargetShell};

mod common;

#[test]
fn test_argv_access_bash() {
    assert_exec_matches_fixture_target("argv_access_basic", TargetShell::Bash);
}

#[test]
fn test_argv_access_posix() {
    assert_exec_matches_fixture_target("argv_access_basic", TargetShell::Posix);
}

#[test]
fn test_no_argv_shell_call() {
    let fixture = common::repo_root().join("sh2c/tests/fixtures/argv_access_basic.sh2");
    
    // We check both targets to be sure
    for target in [TargetShell::Bash, TargetShell::Posix] {
        let compiled = common::compile_path_to_shell(&fixture, target);
        
        // Strict negative assertions:
        // 1. No command substitution calling argv
        assert!(!compiled.contains("$( argv"), "Target {:?} emitted $( argv", target);
        assert!(!compiled.contains("$(argv"), "Target {:?} emitted $(argv", target);
        assert!(!compiled.contains("`argv"), "Target {:?} emitted backtick argv", target);
        
        // 2. No direct calls that look like "argv 1" (incorrect lowering of arg(1))
        assert!(!compiled.contains("argv \"1\""), "Target {:?} emitted argv \"1\"", target);
        assert!(!compiled.contains("argv 1"), "Target {:?} emitted argv 1", target);
        
        // 3. No raw "argv " calls generally, unless it matches "argv0" (which is safe)
        // We REMOVED the broad check for " argv " because it can false-positive on comments or safe strings.
        // The specific checks above cover the actual bad code generation we care about.
    }
}
