use std::fs;

/// Source code lint to detect unsafe bash -c / sh -c patterns in codegen.
///
/// This is a "cheap early warning" lint that scans src/codegen.rs source text
/// for -c usage and verifies it follows S2 safety rules.

#[test]
fn test_codegen_source_only_has_safe_shell_c() {
    let mut files: Vec<_> = fs::read_dir("src/codegen")
        .expect("Failed to read src/codegen directory")
        .flatten()
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        .collect();
    files.sort_by_key(|e| e.path());

    for entry in files {
        let path = entry.path().to_string_lossy().into_owned();
        let codegen_source = fs::read_to_string(entry.path())
            .unwrap_or_else(|_| panic!("Failed to read {}", path));
        
        let lines: Vec<(usize, &str)> = codegen_source
            .lines()
            .enumerate()
            .filter(|(_, line)| line.contains("bash -c") || line.contains("sh -c"))
            .collect();
            
        if lines.is_empty() {
            continue;
        }

        for (line_num, line) in &lines {
            let line_num_1indexed = line_num + 1;
            
            if line.trim_start().starts_with("//") {
                continue;
            }

            let is_probe_helper_emission = line.contains("s.push_str(\"__sh2_sh_probe() {");
            let is_probe_helper_args_emission = line.contains("s.push_str(\"__sh2_sh_probe_args() {");
            
            if is_probe_helper_emission || is_probe_helper_args_emission {
                continue;
            }

            panic!(
                "Found banned bash -c / sh -c usage in {} at line {}.\n\
                 Line: {}\n\
                 \n\
                 Ticket S2/S5 Restriction:\n\
                 Usage of `bash -c` or `sh -c` is STRICTLY PROHIBITED in the compiler,\n\
                 except for the single definition of `__sh2_sh_probe` in `emit_prelude`.\n\
                 \n\
                 If you are trying to use `-c` for something else, YOU MUST NOT.\n\
                 Use direct emission or a different approach.",
                 path,
                 line_num_1indexed,
                 line
            );
        }
    }
}

/// Verify the sh(expr) block uses the expected pattern
#[test]
fn test_sh_expr_block_uses_expected_pattern() {
    let mut files: Vec<_> = fs::read_dir("src/codegen")
        .expect("Failed to read src/codegen directory")
        .flatten()
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        .collect();
    files.sort_by_key(|e| e.path());

    let mut codegen_source = String::new();
    for entry in files {
        codegen_source.push_str("\n/* <");
        codegen_source.push_str(&entry.path().to_string_lossy());
        codegen_source.push_str("> */\n");
        codegen_source.push_str(&fs::read_to_string(entry.path()).unwrap());
    }
    
    // The sh(expr) implementation should:
    // 1. Store command in __sh2_cmd variable
    // 2. Execute via bash -c "$__sh2_cmd" or sh -c "$__sh2_cmd"
    
    assert!(
        codegen_source.contains("Cmd::Raw"),
        "Could not find Cmd::Raw in src/codegen/"
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
