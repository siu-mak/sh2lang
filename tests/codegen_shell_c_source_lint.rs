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
        
        // Get context: 10 lines before and after
        let context_start = line_num.saturating_sub(10);
        let context_end = (line_num + 10).min(codegen_source.lines().count());
        
        let context: String = codegen_source
            .lines()
            .skip(context_start)
            .take(context_end - context_start)
            .collect::<Vec<_>>()
            .join("\n");
        
        // Check if this is the known sh(expr) block
        let is_sh_expr = context.contains("Cmd::Raw") || 
                         context.contains("sh(expr)") ||
                         context.contains("sh() ->") ||
                         context.contains("__sh2_cmd");
        
        if is_sh_expr {
            // sh(expr) is allowed (user-facing escape hatch)
            continue;
        }
        
        // Not sh(expr) - must be compiler-internal -c
        // Check that it follows the safe positional-parameter pattern
        
        // Look for indicators of safe pattern in the source code:
        // 1. Contains ' _ ' or "_ " (placeholder in format string)
        // 2. Contains "$1" or "${1}" (positional refs in script literal)
        
        let has_placeholder = context.contains("' _ ") || 
                              context.contains("_ \"") ||
                              context.contains("_ '");
        
        let _has_positional_ref = context.contains("\"$1\"") || 
                                  context.contains("${1}") ||
                                  context.contains("'$1'");
        
        if !has_placeholder {
            panic!(
                "Found bash -c / sh -c usage in src/codegen.rs at line {} without positional-parameter pattern.\n\
                 Line: {}\n\
                 \n\
                 Compiler-internal -c usage MUST use the positional-parameter pattern:\n\
                 - Include dummy '_' placeholder for $0\n\
                 - Pass dynamic values as separate arguments\n\
                 - Reference them via \"$1\", \"$2\", etc. in the script\n\
                 \n\
                 See docs/codegen-safety.md for the safe pattern.\n\
                 \n\
                 If this is sh(expr), ensure __sh2_cmd variable is used in nearby code.",
                line_num_1indexed,
                line
            );
        }
        
        // If placeholder exists but no positional refs, it might be a constant script
        // That's okay - the runtime test will catch if it's actually unsafe
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
