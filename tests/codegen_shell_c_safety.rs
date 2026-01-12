use std::path::Path;

mod common;
use common::*;

/// Regression test to enforce S2 safety property: compiler-generated `-c`
/// must use positional parameters for dynamic data, never interpolation.
///
/// Exception: user-facing `sh(expr)` is intentionally unsafe (see S1 docs).
///
/// See docs/codegen-safety.md for the full specification.

//
// UNIT-STYLE VALIDATION TESTS
//

//
// OBSOLETE UNIT TESTS REMOVED
// Pattern checks are no longer needed as we now strictly enforce
// helper-only usage for -c.


//
// FIXTURE-BASED INTEGRATION TESTS
//

#[test]
fn test_hello_no_unsafe_shell_c() {
    verify_shell_c_safety("hello");
}

#[test]
fn test_compare_no_unsafe_shell_c() {
    verify_shell_c_safety("compare");
}

#[test]
fn test_for_basic_no_unsafe_shell_c() {
    verify_shell_c_safety("for_basic");
}

#[test]
fn test_concat_amp_basic_no_unsafe_shell_c() {
    verify_shell_c_safety("concat_amp_basic");
}

#[test]
fn test_guardrail_hostile_strings_no_unsafe_shell_c() {
    verify_shell_c_safety("guardrail_hostile_strings");
}

/// Positive test: sh(expr) fixtures are allowed to use -c unsafely
#[test]
fn test_sh_expr_fixtures_allowed_to_use_shell_c() {
    let fixtures_with_sh = vec![
        "sh_expr_var_probe",
        "sh_expr_concat_probe",
        "sh_probe_no_fail_fast",
    ];
    
    for fixture_name in fixtures_with_sh {
        let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
        
        if !Path::new(&sh2_path).exists() {
            continue;
        }
        
        let bash_output = compile_path_to_shell(Path::new(&sh2_path), TargetShell::Bash);
        
        // sh(expr) fixtures are EXPECTED to generate bash -c (documented as unsafe)
        assert!(
            bash_output.contains("bash -c"),
            "Fixture {} should generate 'bash -c' for sh(expr) feature",
            fixture_name
        );
        
        // Also verify strict safety (helper isolation) for these fixtures
        check_shell_c_safety(&bash_output, fixture_name, "bash");
    }
}

//
// HELPER FUNCTIONS
//

/// Verify a fixture follows S2 safety rules
fn verify_shell_c_safety(fixture_name: &str) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    
    assert!(
        Path::new(&sh2_path).exists(),
        "Fixture {} does not exist",
        sh2_path
    );
    
    // Compile for both targets
    let bash_output = compile_path_to_shell(Path::new(&sh2_path), TargetShell::Bash);
    let posix_output = compile_path_to_shell(Path::new(&sh2_path), TargetShell::Posix);
    
    check_shell_c_safety(&bash_output, fixture_name, "bash");
    check_shell_c_safety(&posix_output, fixture_name, "posix");
}

/// Check that generated code follows S2 safety rules
///
/// This is a lint/guardrail using pattern matching, not a full shell parser.
fn check_shell_c_safety(output: &str, fixture_name: &str, target: &str) {
    let lines: Vec<&str> = output.lines().collect();
    
    // Strict enforcement:
    // bash -c / sh -c is ONLY allowed inside the `__sh2_sh_probe() { ... }` definition.
    // It is NOT allowed anywhere else (not even at call sites).
    
    let mut inside_helper = false;
    
    for (i, line) in lines.iter().enumerate() {
        // State tracking for helper definition
        if line.contains("__sh2_sh_probe() {") {
            inside_helper = true;
        }
        
        // Check for banned usage
        if line.contains("bash -c") || line.contains("sh -c") {
            if !inside_helper {
                panic!(
                    "Fixture {} ({} target) line {}: Found banned `bash -c` / `sh -c` outside of probe helper.\n\
                     Line: {}\n\
                     \n\
                     Ticket S5 Strict Safety:\n\
                     Direct execution of `bash -c` or `sh -c` is FORBIDDEN in generated code.\n\
                     All dynamic shell execution must go through `__sh2_sh_probe` helper.",
                    fixture_name, target, i + 1, line
                );
            }
        }
        
        // Helper definition ends with '}'
        // NOTE: This assumes the helper is emitted on a single line or robustly closed.
        // Current emission is single-line or block. If single line:
        if inside_helper && line.contains("}") {
             inside_helper = false;
        }
    }
}
