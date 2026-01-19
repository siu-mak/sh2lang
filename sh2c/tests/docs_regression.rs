//! Regression test to prevent docs from drifting back to incorrect `and`/`or` operator syntax.
//!
//! The sh2 compiler uses `&&` / `||` for logical operators, NOT `and` / `or`.
//! This test scans fenced ```sh2 code blocks in documentation to ensure they 
//! don't use the incorrect textual operator syntax.

use std::fs;
use std::path::Path;

/// Scans markdown content for fenced ```sh2 code blocks and checks for forbidden patterns.
fn check_doc_for_forbidden_patterns(path: &Path) -> Vec<String> {
    let content = fs::read_to_string(path).expect("Failed to read doc file");
    let mut errors = Vec::new();
    let mut in_sh2_block = false;
    let mut block_start_line = 0;

    for (i, line) in content.lines().enumerate() {
        let line_num = i + 1;
        
        // Detect start/end of sh2 code blocks
        if line.trim().starts_with("```sh2") {
            in_sh2_block = true;
            block_start_line = line_num;
            continue;
        }
        if in_sh2_block && line.trim().starts_with("```") {
            in_sh2_block = false;
            continue;
        }

        // Only check inside sh2 code blocks
        if !in_sh2_block {
            continue;
        }

        // Skip comments
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }

        // Check for forbidden patterns:
        // " and " or " or " used as operators (not inside strings)
        // Simple heuristic: look for ` and ` or ` or ` surrounded by non-quote context
        // This is imperfect but catches obvious cases like:
        //   run("x") and run("y")
        //   exists("a") or exists("b")
        
        // Split on quotes to get non-string parts
        let parts: Vec<&str> = line.split('"').collect();
        for (j, part) in parts.iter().enumerate() {
            // Even indices are outside quotes, odd indices are inside
            if j % 2 == 0 {
                if part.contains(" and ") {
                    errors.push(format!(
                        "{}:{} (block starting at {}): Found ` and ` operator in sh2 code block - should be `&&`:\n  {}",
                        path.display(), line_num, block_start_line, line
                    ));
                }
                if part.contains(" or ") {
                    errors.push(format!(
                        "{}:{} (block starting at {}): Found ` or ` operator in sh2 code block - should be `||`:\n  {}",
                        path.display(), line_num, block_start_line, line
                    ));
                }
            }
        }
    }

    errors
}

#[test]
fn docs_no_and_or_operators_in_sh2_blocks() {
    let docs = [
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("README.md"),
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("docs/language.md"),
    ];

    let mut all_errors = Vec::new();

    for doc in &docs {
        if doc.exists() {
            let errors = check_doc_for_forbidden_patterns(doc);
            all_errors.extend(errors);
        }
    }

    if !all_errors.is_empty() {
        panic!(
            "Found {} forbidden `and`/`or` operator(s) in sh2 code blocks:\n\n{}",
            all_errors.len(),
            all_errors.join("\n\n")
        );
    }
}

#[test]
fn feature_matrix_exists_if_referenced() {
    // If README.md references feature_matrix.md, it should exist
    let readme = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("README.md");
    let content = fs::read_to_string(&readme).unwrap_or_default();
    
    if content.contains("feature_matrix.md") {
        let feature_matrix = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("docs/feature_matrix.md");
        assert!(
            feature_matrix.exists(),
            "README.md references feature_matrix.md but it doesn't exist at {}",
            feature_matrix.display()
        );
    }
}
