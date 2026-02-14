
mod common;
use common::{assert_exec_matches_fixture_target, compile_path_to_shell, TargetShell};
use std::path::Path;

#[test]
fn test_capture_allow_fail_status_empty() {
    let fixture = "capture_allow_fail_status_empty";
    for target in [TargetShell::Bash, TargetShell::Posix] {
        verify_codegen_restore_pattern(fixture, target);
        assert_exec_matches_fixture_target(fixture, target);
    }
}

#[test]
fn test_capture_allow_fail_status_with_stdout() {
    let fixture = "capture_allow_fail_status_with_stdout";
    for target in [TargetShell::Bash, TargetShell::Posix] {
        verify_codegen_restore_pattern(fixture, target);
        assert_exec_matches_fixture_target(fixture, target);
    }
}

#[test]
fn test_capture_allow_fail_status_persists_after_print() {
    let fixture = "capture_allow_fail_status_persists_after_print";
    for target in [TargetShell::Bash, TargetShell::Posix] {
        verify_codegen_restore_pattern(fixture, target);
        assert_exec_matches_fixture_target(fixture, target);
    }
}

// Whitebox check: explicitly ensure the generated script restores __sh2_status from 
// the capture status variable AFTER cleanup.
fn verify_codegen_restore_pattern(fixture: &str, target: TargetShell) {
    let fixture_path = format!("tests/fixtures/{}.sh2", fixture);
    let script = compile_path_to_shell(Path::new(&fixture_path), target);
    
    // Pattern we expect:
    // ... capture ...
    // clean up (rm -f ...)
    // ...
    // __sh2_status="${...}"
    
    // We can't predict exact var names easily without robust parsing, but we can check order.
    // The emitted code for capture(allow_fail=true) ends with:
    // ... rm -f ...
    // __sh2_status="${...}"
    
    // Let's find the capture block's cleanup and verify strictly that __sh2_status is restored *after* it.
    // Since main() is the only place we use capture in these fixtures, we can search the whole string.
    
    // We look for 'rm -f ' followed eventually by '__sh2_status="${'
    // This is heuristic but sufficient for this specific regression test.
    
    let rm_idx = script.find("rm -f ").expect("Generated script should contain cleanup (rm -f)");
    let restore_idx = script.rfind("__sh2_status=\"${").expect("Generated script should restore __sh2_status");
    
    assert!(restore_idx > rm_idx, 
        "Target {:?}: __sh2_status restore (idx {}) must happen AFTER cleanup (idx {}) to avoid clobbering.\nScript snippet:\n{}", 
        target, restore_idx, rm_idx, &script[rm_idx..]
    );
}
