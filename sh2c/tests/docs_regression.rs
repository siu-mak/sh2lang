//! Regression tests to prevent docs from drifting to incorrect patterns.
//!
//! 1. `and`/`or` operators: The sh2 compiler uses `&&` / `||`, NOT `and` / `or`.
//! 2. Boolean materialization: Assigning boolean expressions to variables is not
//!    supported (e.g. `let ok = (sum == 42)`). Examples showing this must be
//!    explicitly labeled as unsupported with markers like `# ❌` or `# not supported`.

use std::fs;
use std::path::{Path, PathBuf};

/// Gather all markdown docs: README.md + docs/**/*.md
fn gather_doc_paths() -> Vec<PathBuf> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut docs = Vec::new();

    // Root README.md
    let readme = repo_root.join("README.md");
    if readme.exists() {
        docs.push(readme);
    }

    // All *.md under docs/
    let docs_dir = repo_root.join("docs");
    if docs_dir.is_dir() {
        gather_md_files_recursive(&docs_dir, &mut docs);
    }

    docs
}

fn gather_md_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                gather_md_files_recursive(&path, out);
            } else if path.extension().map(|e| e == "md").unwrap_or(false) {
                out.push(path);
            }
        }
    }
}

/// Check if a line or nearby context has an "unsupported" marker.
fn has_unsupported_marker(line: &str, prev_lines: &[&str]) -> bool {
    let markers = ["# ❌", "not supported", "unsupported", "limitation"];
    let line_lower = line.to_lowercase();

    // Check current line
    for marker in &markers {
        if line_lower.contains(&marker.to_lowercase()) {
            return true;
        }
    }

    // Check previous 1-2 non-empty lines (comment context)
    for prev in prev_lines.iter().rev().take(2) {
        let prev_lower = prev.to_lowercase();
        if prev.trim().starts_with('#') {
            for marker in &markers {
                if prev_lower.contains(&marker.to_lowercase()) {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if RHS of a `let` statement looks like a boolean expression.
fn is_likely_boolean_rhs(rhs: &str) -> bool {
    // Comparison operators
    let comparison_ops = ["==", "!=", "<=", ">=", "<", ">"];
    for op in &comparison_ops {
        if rhs.contains(op) {
            return true;
        }
    }

    // Logical operators
    if rhs.contains("&&") || rhs.contains("||") {
        return true;
    }

    // Leading ! (negation) - but not != which is already covered
    let trimmed = rhs.trim();
    if trimmed.starts_with('!') && !trimmed.starts_with("!=") {
        return true;
    }

    // Boolean literals (outside of strings)
    let parts: Vec<&str> = rhs.split('"').collect();
    for (i, part) in parts.iter().enumerate() {
        if i % 2 == 0 {
            // Outside quotes
            let words: Vec<&str> = part.split_whitespace().collect();
            if words.iter().any(|w| *w == "true" || *w == "false") {
                return true;
            }
        }
    }

    // Known predicate builtins
    let predicates = [
        "exists(",
        "is_dir(",
        "is_file(",
        "is_symlink(",
        "is_exec(",
        "is_readable(",
        "is_writable(",
        "is_non_empty(",
        "matches(",
        "contains(",
        "contains_line(",
    ];
    for pred in &predicates {
        if rhs.contains(pred) {
            return true;
        }
    }

    false
}

/// Scans markdown content for fenced ```sh2 code blocks and checks for:
/// 1. Forbidden `and`/`or` operator syntax
/// 2. Boolean materialization (let x = <boolean>) without unsupported marker
fn check_doc_for_all_patterns(path: &Path) -> Vec<String> {
    let content = fs::read_to_string(path).expect("Failed to read doc file");
    let mut errors = Vec::new();
    let mut in_sh2_block = false;
    let mut block_start_line = 0;
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
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

        let trimmed = line.trim();

        // Skip pure comment lines for operator check
        let is_comment = trimmed.starts_with('#');

        // === Check 1: Forbidden `and`/`or` operators ===
        if !is_comment {
            let parts: Vec<&str> = line.split('"').collect();
            for (j, part) in parts.iter().enumerate() {
                if j % 2 == 0 {
                    if part.contains(" and ") {
                        errors.push(format!(
                            "{}:{} (block@{}): ` and ` operator - should be `&&`:\n  {}",
                            path.display(),
                            line_num,
                            block_start_line,
                            line
                        ));
                    }
                    if part.contains(" or ") {
                        errors.push(format!(
                            "{}:{} (block@{}): ` or ` operator - should be `||`:\n  {}",
                            path.display(),
                            line_num,
                            block_start_line,
                            line
                        ));
                    }
                }
            }
        }

        // === Check 2: Boolean materialization ===
        // Look for: let <ident> = <boolean-expr>
        if !is_comment && trimmed.starts_with("let ") {
            if let Some(eq_pos) = trimmed.find('=') {
                // Make sure it's not == (comparison)
                let after_eq = &trimmed[eq_pos..];
                if !after_eq.starts_with("==") {
                    let rhs = &trimmed[eq_pos + 1..];
                    if is_likely_boolean_rhs(rhs) {
                        // Check for unsupported marker
                        let prev_lines: Vec<&str> = if i >= 2 {
                            lines[i.saturating_sub(2)..i].to_vec()
                        } else {
                            lines[..i].to_vec()
                        };

                        if !has_unsupported_marker(line, &prev_lines) {
                            errors.push(format!(
                                "{}:{} (block@{}): Boolean materialization without unsupported marker:\n  {}\n  Hint: Add `# ❌ Not supported:` comment or use inline condition/bool_str()",
                                path.display(),
                                line_num,
                                block_start_line,
                                line
                            ));
                        }
                    }
                }
            }
        }
    }

    errors
}

#[test]
fn docs_no_and_or_operators_in_sh2_blocks() {
    let docs = gather_doc_paths();
    let mut all_errors = Vec::new();

    for doc in &docs {
        let errors = check_doc_for_all_patterns(doc);
        // Filter to only and/or errors for this test
        let filtered: Vec<_> = errors
            .into_iter()
            .filter(|e| e.contains("` and `") || e.contains("` or `"))
            .collect();
        all_errors.extend(filtered);
    }

    if !all_errors.is_empty() {
        panic!(
            "Found {} forbidden `and`/`or` operator(s) in sh2 code blocks:\n\n{}",
            all_errors.len(),
            all_errors.join("\n\n")
        );
    }
}

// NOTE: docs_no_unmarked_boolean_materialization test was removed because
// boolean materialization is now supported (let ok = (x == 42) is valid).
// The test was designed to catch the old limitation, which no longer applies.

#[test]
fn feature_matrix_exists_if_referenced() {
    let readme = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("README.md");
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
