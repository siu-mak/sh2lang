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

/// Test that safe -c patterns are correctly identified
#[test]
fn test_safe_shell_c_patterns() {
    // Safe: single-quoted script with positional params
    assert_shell_c_is_safe("bash -c 'printf \"%s\\n\" \"$1\" | grep -Fxq -- \"$2\"' _ \"$text\" \"$needle\"");
    assert_shell_c_is_safe("sh -c 'test -n \"$1\"' _ \"$val\"");
    assert_shell_c_is_safe("bash -c 'echo \"$1\" \"$2\"' _ \"hello\" \"world\"");
    
    // Safe: no dynamic data (no args after _)
    assert_shell_c_is_safe("bash -c 'echo hello' _");
}

/// Test that unsafe -c patterns are correctly detected
#[test]
fn test_unsafe_shell_c_patterns() {
    // Unsafe: double-quoted script with variable
    assert_shell_c_is_unsafe("bash -c \"printf %s $text\"");
    
    // Unsafe: quote concatenation
    assert_shell_c_is_unsafe("bash -c 'echo '\"$x\"");
    
    // Unsafe: script uses $x instead of $1
    assert_shell_c_is_unsafe("bash -c 'echo $x' _ \"$x\"");
    
    // Unsafe: script uses ${var} instead of ${1}
    assert_shell_c_is_unsafe("bash -c 'test -n ${myvar}' _ \"$val\"");
}

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
    
    for (i, line) in lines.iter().enumerate() {
        if !line.contains("bash -c") && !line.contains("sh -c") {
            continue;
        }
        
        // Found a -c usage - check if it's sh(expr) or compiler-internal
        
        // Detect sh(expr) by signature: look for __sh2_cmd= in nearby lines
        let is_sh_expr = is_sh_expr_usage(&lines, i);
        
        if is_sh_expr {
            // sh(expr) is allowed to use -c unsafely (user-facing escape hatch)
            continue;
        }
        
        // Not sh(expr) - must follow safe positional-parameter pattern
        check_line_is_safe_shell_c(line, fixture_name, target, i + 1);
    }
}

/// Detect if a -c usage is from sh(expr) by looking for the signature
fn is_sh_expr_usage(lines: &[&str], line_idx: usize) -> bool {
    // sh(expr) implementation uses __sh2_cmd variable
    // Look in a window around the -c line
    let start = line_idx.saturating_sub(5);
    let end = (line_idx + 5).min(lines.len());
    
    for i in start..end {
        if lines[i].contains("__sh2_cmd=") {
            return true;
        }
    }
    
    false
}

/// Check that a single line with -c follows the safe pattern
fn check_line_is_safe_shell_c(line: &str, fixture_name: &str, target: &str, line_num: usize) {
    // Safe pattern requirements:
    // 1. Single-quoted script: -c '...'
    // 2. Dummy _ placeholder after script
    // 3. If args present, script uses $1, $2, etc. (not $var, ${var})
    
    // Check 1: Single-quoted script (not double-quoted)
    if line.contains("-c \"") {
        panic!(
            "Fixture {} ({} target) line {}: Unsafe -c pattern detected.\n\
             Line: {}\n\
             \n\
             Compiler-internal -c must use single-quoted script, not double-quoted.\n\
             Double quotes allow variable interpolation which violates S2.\n\
             See docs/codegen-safety.md for the safe pattern.",
            fixture_name, target, line_num, line
        );
    }
    
    // Check 2: Must have dummy _ placeholder
    // Look for ' _ (after closing quote) or " _ (after closing double quote)
    // The placeholder comes after the script literal
    if !line.contains("' _") && !line.contains("\" _") {
        panic!(
            "Fixture {} ({} target) line {}: Unsafe -c pattern detected.\n\
             Line: {}\n\
             \n\
             Compiler-internal -c must include dummy '_' placeholder for $0.\n\
             Pattern should be: -c '...' _ or -c \"...\" _\n\
             See docs/codegen-safety.md for the safe pattern.",
            fixture_name, target, line_num, line
        );
    }
    
    // Check 3: Script must not use non-positional variables
    // Extract the script portion (between -c ' and ')
    if let Some(script) = extract_script_from_line(line) {
        // Check for $var or ${var} patterns (non-positional)
        // Allow $1, $2, ${1}, ${2} (positional)
        
        // Simple heuristic: look for $ followed by letter or underscore
        // This catches $var, ${var}, $HOME, etc.
        // But allows $1, $2, $?, $$, etc.
        
        let has_non_positional_var = script.chars()
            .zip(script.chars().skip(1))
            .any(|(c1, c2)| c1 == '$' && (c2.is_alphabetic() || c2 == '_' || c2 == '{'));
        
        if has_non_positional_var {
            // Check if it's actually ${1}, ${2}, etc. (positional)
            let is_positional_brace = script.contains("${1") || 
                                       script.contains("${2") ||
                                       script.contains("${3");
            
            if !is_positional_brace {
                panic!(
                    "Fixture {} ({} target) line {}: Unsafe -c pattern detected.\n\
                     Line: {}\n\
                     Script: {}\n\
                     \n\
                     Script uses non-positional variable ($var or ${{var}}).\n\
                     Compiler-internal -c must use positional parameters ($1, $2, etc.).\n\
                     See docs/codegen-safety.md for the safe pattern.",
                    fixture_name, target, line_num, line, script
                );
            }
        }
    }
}

/// Extract the script literal from a -c line
fn extract_script_from_line(line: &str) -> Option<String> {
    // Look for -c ' and extract until matching '
    if let Some(start_idx) = line.find("-c '") {
        let script_start = start_idx + 4; // Skip "-c '"
        let remaining = &line[script_start..];
        
        // Find the closing '
        if let Some(end_idx) = remaining.find("' ") {
            return Some(remaining[..end_idx].to_string());
        }
    }
    
    None
}

/// Assert that a line follows the safe -c pattern
fn assert_shell_c_is_safe(line: &str) {
    // Use the same check as the main test
    check_line_is_safe_shell_c(line, "unit_test", "test", 1);
}

/// Assert that a line violates the safe -c pattern
fn assert_shell_c_is_unsafe(line: &str) {
    let result = std::panic::catch_unwind(|| {
        check_line_is_safe_shell_c(line, "unit_test", "test", 1);
    });
    
    assert!(
        result.is_err(),
        "Expected line to be detected as unsafe, but it passed: {}",
        line
    );
}
