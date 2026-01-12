use std::fs;

/// Source code lint to detect unsafe bash -c / sh -c patterns in codegen.
///
/// This is a "cheap early warning" lint that scans src/codegen.rs source text
/// for -c usage and verifies it follows S2 safety rules.

#[test]
fn test_codegen_source_only_has_safe_shell_c() {
    let codegen_source = fs::read_to_string("src/codegen.rs")
        .expect("Failed to read src/codegen.rs");
    
    // Find all occurrences of bash -c or sh -c in the source
    let lines: Vec<(usize, &str)> = codegen_source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("bash -c") || line.contains("sh -c"))
        .collect();
    
    if lines.is_empty() {
        // No -c usage in source - safe
        return;
    }
    
    // Check each occurrence
    for (line_num, line) in &lines {
        let line_num_1indexed = line_num + 1;
        
        // Skip comments
        if line.trim_start().starts_with("//") {
            continue;
        }

        // Strict Allowlist:
        // The ONLY allowed usage of `bash -c` / `sh -c` is within the literal string emission
        // for the `__sh2_sh_probe` helper function.
        // There are no other valid uses (compiler-internal -c is banned).
        
        let is_probe_helper_emission = line.contains("s.push_str(\"__sh2_sh_probe() {");
        
        if is_probe_helper_emission {
            continue;
        }

        // Failed strict allowlist
        panic!(
            "Found banned bash -c / sh -c usage in src/codegen.rs at line {}.\n\
             Line: {}\n\
             \n\
             Ticket S2/S5 Restriction:\n\
             Usage of `bash -c` or `sh -c` is STRICTLY PROHIBITED in the compiler,\n\
             except for the single definition of `__sh2_sh_probe` in `emit_prelude`.\n\
             \n\
             If you are trying to use `-c` for something else, YOU MUST NOT.\n\
             Use direct emission or a different approach.",
            line_num_1indexed,
            line
        );
    }
}

/// Verify the sh(expr) block uses the expected pattern
#[test]
fn test_sh_expr_block_uses_expected_pattern() {
    let codegen_source = fs::read_to_string("src/codegen.rs")
        .expect("Failed to read src/codegen.rs");
    
    // The sh(expr) implementation should:
    // 1. Store command in __sh2_cmd variable
    // 2. Execute via bash -c "$__sh2_cmd" or sh -c "$__sh2_cmd"
    
    assert!(
        codegen_source.contains("Cmd::Raw"),
        "Could not find Cmd::Raw in src/codegen.rs"
    );
    
    assert!(
        codegen_source.contains("__sh2_cmd"),
        "sh(expr) implementation should use __sh2_cmd variable"
    );
    
    assert!(
        codegen_source.contains("bash -c") || codegen_source.contains("sh -c"),
        "sh(expr) implementation should use bash -c or sh -c"
    );
}
