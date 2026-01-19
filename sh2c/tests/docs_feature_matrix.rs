//! Regression tests to ensure docs/feature_matrix.md stays in sync with tests.
//!
//! This test enforces:
//! 1. All evidence references in feature_matrix.md must exist (tests or fixtures).
//! 2. Globs must match at least one file.
//! 3. Every `syntax_*.rs` test must be covered by the matrix (directly or via glob).
//!
//! If exclusions are needed, they must:
//! - Be listed in EXCLUDED_SYNTAX_TESTS with an inline comment
//! - Reference a file that actually exists

use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Excluded syntax tests: each MUST exist and have an explanation.
/// These are internal/infrastructure tests that don't represent user-facing features.
const EXCLUDED_SYNTAX_TESTS: &[(&str, &str)] = &[
    // Boolean variable internals
    ("syntax_bool_var.rs", "implementation detail: boolean variable assignment internals"),
    ("syntax_bool_compare_literals.rs", "implementation detail: bool comparison internals"),
    ("syntax_bool_str_as_arg.rs", "implementation detail: bool_str() argument handling"),
    ("syntax_bool_str_basic.rs", "implementation detail: bool_str() basic behavior"),
    // Quoting/escaping internals
    ("syntax_string_quote_escape.rs", "implementation detail: shell escaping internals"),
    ("syntax_string_split_default.rs", "implementation detail: default IFS behavior"),
    ("syntax_quoting.rs", "implementation detail: quoting contract tests"),
    // Return semantics internals
    ("syntax_return_fs_predicate.rs", "implementation detail: predicate return semantics"),
    ("syntax_return_not.rs", "implementation detail: negation return semantics"),
    ("syntax_return_semantics.rs", "implementation detail: function return semantics"),
    // Pipe parser edge cases
    ("syntax_pipe_parser_errors.rs", "infrastructure: parser error handling tests"),
    // Scope internals
    ("syntax_subshell_group_scope.rs", "implementation detail: variable scope in subshells"),
    // Target internals
    ("syntax_target.rs", "infrastructure: multi-target codegen validation"),
    ("syntax_target_posix.rs", "infrastructure: POSIX-specific codegen validation"),
    // System builtins internals
    ("syntax_system.rs", "implementation detail: system builtin internals"),
    ("syntax_system_vars.rs", "implementation detail: system variable internals"),
    // Truthiness semantics
    ("syntax_truthiness.rs", "implementation detail: condition truthiness rules"),
    // Generic value handling
    ("syntax_values.rs", "implementation detail: value type handling"),
    // Misc internals
    ("syntax_misc.rs", "infrastructure: miscellaneous edge case tests"),
    // Concat behavior internals
    ("syntax_concat_amp.rs", "implementation detail: `&` concat operator internals"),
    ("syntax_concat_amp_precedence.rs", "implementation detail: concat precedence"),
    // Dollar pipe internals
    ("syntax_dollar_pipe.rs", "implementation detail: $() pipe handling"),
];

/// Extract all evidence references from feature_matrix.md.
/// Matches any backtick-wrapped token containing `.rs` or `.sh2`.
/// Supports globs (e.g., `syntax_fs_*.rs`).
fn extract_evidence_refs(content: &str) -> HashSet<String> {
    let mut refs = HashSet::new();
    let mut in_backtick = false;
    let mut current = String::new();
    
    for ch in content.chars() {
        if ch == '`' {
            if in_backtick {
                // Check if it contains .rs or .sh2 (handles globs like *.rs)
                if current.contains(".rs") || current.contains(".sh2") {
                    refs.insert(current.clone());
                }
                current.clear();
            }
            in_backtick = !in_backtick;
        } else if in_backtick {
            current.push(ch);
        }
    }
    refs
}

/// Gather all files in a directory matching a suffix
fn gather_files_with_suffix(dir: &Path, suffix: &str) -> HashSet<String> {
    let mut files = HashSet::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(suffix) {
                files.insert(name);
            }
        }
    }
    files
}

/// Check if a pattern (with single `*`) matches a filename.
/// Only supports single `*` wildcard.
fn pattern_matches(pattern: &str, filename: &str) -> bool {
    if !pattern.contains('*') {
        return pattern == filename;
    }
    
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() != 2 {
        // Multi-* not supported
        return false;
    }
    filename.starts_with(parts[0]) && filename.ends_with(parts[1])
}

/// Expand a pattern to matching files
fn expand_pattern(pattern: &str, files: &HashSet<String>) -> Vec<String> {
    files.iter()
        .filter(|f| pattern_matches(pattern, f))
        .cloned()
        .collect()
}

/// Check if evidence is a glob pattern
fn is_glob(s: &str) -> bool {
    s.contains('*')
}

#[test]
fn excluded_syntax_tests_exist() {
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let existing = gather_files_with_suffix(&tests_dir, ".rs");
    
    let mut missing: Vec<&str> = Vec::new();
    for (file, _reason) in EXCLUDED_SYNTAX_TESTS {
        if !existing.contains(*file) {
            missing.push(file);
        }
    }
    
    if !missing.is_empty() {
        panic!(
            "EXCLUDED_SYNTAX_TESTS references {} non-existent file(s):\n  - {}\n\n\
            Remove them from the exclusion list or fix the filename.",
            missing.len(),
            missing.join("\n  - ")
        );
    }
}

#[test]
fn feature_matrix_evidence_exists() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent().unwrap();
    let matrix_path = repo_root.join("docs/feature_matrix.md");
    
    let content = fs::read_to_string(&matrix_path)
        .expect("Could not read docs/feature_matrix.md");
    
    let evidence_refs = extract_evidence_refs(&content);
    
    // Gather all test files
    let sh2c_tests = manifest_dir.join("tests");
    let sh2do_tests = repo_root.join("sh2do/tests");
    let fixtures_dir = manifest_dir.join("tests/fixtures");
    
    let mut all_rs: HashSet<String> = HashSet::new();
    all_rs.extend(gather_files_with_suffix(&sh2c_tests, ".rs"));
    all_rs.extend(gather_files_with_suffix(&sh2do_tests, ".rs"));
    
    let all_sh2 = gather_files_with_suffix(&fixtures_dir, ".sh2");
    
    let mut missing = Vec::new();
    
    for evidence in &evidence_refs {
        if evidence.contains(".rs") {
            if is_glob(evidence) {
                let expanded = expand_pattern(evidence, &all_rs);
                if expanded.is_empty() {
                    missing.push(format!("{} (glob matched no .rs files)", evidence));
                }
            } else {
                if !all_rs.contains(evidence) {
                    missing.push(evidence.clone());
                }
            }
        } else if evidence.contains(".sh2") {
            let filename = evidence.rsplit('/').next().unwrap_or(evidence);
            if is_glob(filename) {
                let expanded = expand_pattern(filename, &all_sh2);
                if expanded.is_empty() {
                    missing.push(format!("{} (glob matched no .sh2 files)", evidence));
                }
            } else if !all_sh2.contains(filename) {
                missing.push(evidence.clone());
            }
        }
    }
    
    if !missing.is_empty() {
        panic!(
            "feature_matrix.md references {} non-existent file(s):\n  - {}\n\n\
            Fix the reference or remove it from the matrix.",
            missing.len(),
            missing.join("\n  - ")
        );
    }
}

#[test]
fn feature_matrix_covers_syntax_tests() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent().unwrap();
    let matrix_path = repo_root.join("docs/feature_matrix.md");
    
    let content = fs::read_to_string(&matrix_path)
        .expect("Could not read docs/feature_matrix.md");
    
    let sh2c_tests = manifest_dir.join("tests");
    let syntax_files: HashSet<String> = gather_files_with_suffix(&sh2c_tests, ".rs")
        .into_iter()
        .filter(|f| f.starts_with("syntax_"))
        .collect();
    
    let evidence_refs = extract_evidence_refs(&content);
    
    // Expand all evidence to get covered files
    let mut covered: HashSet<String> = HashSet::new();
    for evidence in &evidence_refs {
        // Only process syntax_* references (direct or glob)
        if evidence.starts_with("syntax_") && evidence.contains(".rs") {
            if is_glob(evidence) {
                for expanded in expand_pattern(evidence, &syntax_files) {
                    covered.insert(expanded);
                }
            } else {
                covered.insert(evidence.clone());
            }
        }
    }
    
    // Get excluded set
    let excluded: HashSet<String> = EXCLUDED_SYNTAX_TESTS
        .iter()
        .map(|(f, _)| f.to_string())
        .collect();
    
    // Find uncovered
    let mut uncovered: Vec<String> = syntax_files
        .iter()
        .filter(|f| !covered.contains(*f) && !excluded.contains(*f))
        .cloned()
        .collect();
    uncovered.sort();
    
    if !uncovered.is_empty() {
        panic!(
            "feature_matrix.md is missing {} syntax test(s):\n  - {}\n\n\
            Either:\n\
            1. Add the test to feature_matrix.md, OR\n\
            2. Add it to EXCLUDED_SYNTAX_TESTS with a reason.",
            uncovered.len(),
            uncovered.join("\n  - ")
        );
    }
}
