//! Policy test: every `sh("...")` usage in `sh2` code blocks must be justified.
//!
//! Scans all markdown docs for `sh("` inside fenced ```sh2 blocks.
//! Each occurrence must have a `# sh(...) because:` or `// sh(...) because:`
//! comment within 3 lines above (same fence), OR be inside a section wrapped
//! with explicit sentinel markers:
//!
//!     <!-- sh2-docs:allow-sh-examples:start -->
//!     ...code here is exempt...
//!     <!-- sh2-docs:allow-sh-examples:end -->
//!
//! ## Allowlist strategy
//!
//! We allowlist by **HTML comment sentinels**, not heading text. This makes
//! the allowlist immune to heading wording changes. Only sections that are
//! reference documentation for `sh()` itself should be wrapped:
//!
//! - `docs/language.md`: The §6.3 "sh(expr)" reference section.
//!   The "Prefer structured primitives" subsection is deliberately
//!   left OUTSIDE the markers.
//!
//! - `README.md`: The `sh("...")` raw shell escape hatch section.
//!
//! ## Canonical justification marker
//!
//! The canonical marker is `sh(...) because:` (with parenthesized ellipsis).
//! Accepted in both `#` and `//` comment styles. The variant `sh() because:`
//! is NOT accepted to keep enforcement meaningful.
//!
//! ## Fence parsing
//!
//! Supports fences of 3+ backticks. Tracks opening delimiter length so
//! closing fences match correctly. Tolerates trailing whitespace.

use std::fs;
use std::path::{Path, PathBuf};

// ── Constants ──────────────────────────────────────────────────

const ALLOW_START: &str = "<!-- sh2-docs:allow-sh-examples:start -->";
const ALLOW_END: &str = "<!-- sh2-docs:allow-sh-examples:end -->";
const JUSTIFICATION_MARKER: &str = "sh(...) because:";
const LOOKBACK_LINES: usize = 3;

// ── File discovery ─────────────────────────────────────────────

/// Gather all markdown docs: README.md + docs/**/*.md
fn gather_doc_paths() -> Vec<PathBuf> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut docs = Vec::new();

    let readme = repo_root.join("README.md");
    if readme.exists() {
        docs.push(readme);
    }

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
            } else if path.extension().map_or(false, |e| e == "md") {
                out.push(path);
            }
        }
    }
}

// ── Fence detection ────────────────────────────────────────────

/// Parse the opening fence: returns (backtick_count, language) if this is
/// a valid opening fence. Supports 3+ backticks with optional language tag.
fn parse_fence_open(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with("```") {
        return None;
    }
    let backtick_count = trimmed.chars().take_while(|c| *c == '`').count();
    if backtick_count < 3 {
        return None;
    }
    let after = trimmed[backtick_count..].trim().to_string();
    // Opening fences have a language tag (possibly empty), no backticks after.
    // If it contains backticks, it's not a valid opening fence.
    if after.contains('`') {
        return None;
    }
    Some((backtick_count, after))
}

/// Check if a line closes a fence opened with `open_backtick_count` backticks.
/// A closing fence has at least that many backticks and no other content.
fn is_fence_close(line: &str, open_backtick_count: usize) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with("```") {
        return false;
    }
    let count = trimmed.chars().take_while(|c| *c == '`').count();
    if count < open_backtick_count {
        return false;
    }
    // After the backticks, only whitespace is allowed.
    trimmed[count..].trim().is_empty()
}

// ── Justification check ───────────────────────────────────────

/// Check if a `sh(...) because:` justification exists within lookback
/// lines above (within the same fence) or inline on the current line.
fn has_justification(
    lines: &[&str],
    current_idx: usize,
    fence_start_idx: usize,
) -> bool {
    // Check the current line (inline justification).
    if lines[current_idx].contains(JUSTIFICATION_MARKER) {
        return true;
    }
    // Check up to LOOKBACK_LINES above, but not before the fence start.
    let start = current_idx
        .saturating_sub(LOOKBACK_LINES)
        .max(fence_start_idx + 1);
    for i in start..current_idx {
        if lines[i].contains(JUSTIFICATION_MARKER) {
            return true;
        }
    }
    false
}

// ── Main scanner ──────────────────────────────────────────────

/// Violation found by the scanner.
#[derive(Debug)]
struct Violation {
    file: String,
    line_num: usize,
    line: String,
    context: Vec<(usize, String)>, // (line_num, content)
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}: sh(\"...\") without justification:\n  {:>4} | {}",
            self.file, self.line_num, self.line_num, self.line
        )?;
        if !self.context.is_empty() {
            write!(f, "\n  Context:")?;
            for (num, content) in &self.context {
                write!(f, "\n  {:>4} | {}", num, content)?;
            }
        }
        write!(
            f,
            "\n  Hint: Add `# sh(...) because: <reason>` within 3 lines above"
        )
    }
}

/// Scan a single file (or in-memory content) for policy violations.
fn scan_content(file_label: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut in_sh2_block = false;
    let mut fence_backtick_count: usize = 0;
    let mut fence_start_idx: usize = 0;
    let mut in_allowed_section = false;
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Track sentinel markers (outside code blocks only).
        if !in_sh2_block {
            if trimmed == ALLOW_START {
                in_allowed_section = true;
                continue;
            }
            if trimmed == ALLOW_END {
                in_allowed_section = false;
                continue;
            }
        }

        // Detect start of code blocks.
        if !in_sh2_block {
            if let Some((count, lang)) = parse_fence_open(line) {
                if lang == "sh2" {
                    in_sh2_block = true;
                    fence_backtick_count = count;
                    fence_start_idx = i;
                }
                continue;
            }
        }

        // Detect end of code blocks.
        if in_sh2_block && is_fence_close(line, fence_backtick_count) {
            in_sh2_block = false;
            continue;
        }

        // Only check inside sh2 code blocks.
        if !in_sh2_block {
            continue;
        }

        // Skip if inside an allowlisted section.
        if in_allowed_section {
            continue;
        }

        // Skip pure comment lines (# or // at start of trimmed line).
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        // Check for sh(" usage.
        if !trimmed.contains("sh(\"") && !trimmed.contains("sh(r\"") {
            continue;
        }

        // Check for justification within lookback window (same fence) or inline.
        if !has_justification(&lines, i, fence_start_idx) {
            // Build context: up to 3 preceding lines for debugging.
            let ctx_start = i.saturating_sub(3).max(fence_start_idx + 1);
            let context: Vec<(usize, String)> = (ctx_start..i)
                .map(|j| (j + 1, lines[j].to_string()))
                .collect();

            violations.push(Violation {
                file: file_label.to_string(),
                line_num: i + 1,
                line: line.to_string(),
                context,
            });
        }
    }

    violations
}

fn check_sh_usage_policy(path: &Path) -> Vec<Violation> {
    let content = fs::read_to_string(path).expect("Failed to read doc file");
    scan_content(&path.display().to_string(), &content)
}

// ── Main test ─────────────────────────────────────────────────

#[test]
fn docs_sh_usage_policy() {
    let paths = gather_doc_paths();
    assert!(!paths.is_empty(), "No doc files found");

    let mut all_violations = Vec::new();
    for path in &paths {
        let violations = check_sh_usage_policy(path);
        all_violations.extend(violations);
    }

    if !all_violations.is_empty() {
        eprintln!(
            "\n=== sh() usage policy violations ({}) ===\n",
            all_violations.len()
        );
        for v in &all_violations {
            eprintln!("{}\n", v);
        }
        panic!(
            "{} unjustified sh(\"...\") usage(s) in docs. \
             Add `# sh(...) because: <reason>` comment within 3 lines above each usage, \
             or use a structured primitive (glob(), find0(), run() | run(), spawn()).",
            all_violations.len()
        );
    }
}

// ── Fixture-based unit tests ──────────────────────────────────
//
// These tests exercise the scanner over synthetic markdown content
// to verify edge cases without depending on doc headings or wording.

#[cfg(test)]
mod fixture_tests {
    use super::*;

    /// Load a fixture file from sh2c/tests/fixtures/docs_policy/
    fn fixture_path(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/docs_policy")
            .join(name)
    }

    fn fixture_content(name: &str) -> String {
        let path = fixture_path(name);
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path.display(), e))
    }

    #[test]
    fn fixture_unjustified_triggers_violation() {
        let content = fixture_content("unjustified.md");
        let violations = scan_content("fixtures/unjustified.md", &content);
        assert!(
            !violations.is_empty(),
            "Expected violations for unjustified sh() but found none"
        );
        // Should flag the specific unjustified line
        assert!(
            violations.iter().any(|v| v.line.contains("sh(\"echo bad\")")),
            "Should flag sh(\"echo bad\"), got: {:?}",
            violations
        );
    }

    #[test]
    fn fixture_justified_passes() {
        let content = fixture_content("justified.md");
        let violations = scan_content("fixtures/justified.md", &content);
        assert!(
            violations.is_empty(),
            "Expected no violations but got: {:?}",
            violations
        );
    }

    #[test]
    fn fixture_sentinel_markers_exempt() {
        let content = fixture_content("sentinel_allowed.md");
        let violations = scan_content("fixtures/sentinel_allowed.md", &content);
        assert!(
            violations.is_empty(),
            "Expected no violations inside sentinel markers but got: {:?}",
            violations
        );
    }

    #[test]
    fn fixture_sentinel_outside_not_exempt() {
        let content = fixture_content("sentinel_outside.md");
        let violations = scan_content("fixtures/sentinel_outside.md", &content);
        assert!(
            !violations.is_empty(),
            "Expected violations for sh() outside sentinel markers but found none"
        );
    }

    #[test]
    fn fixture_four_backtick_fence() {
        let content = fixture_content("four_backtick.md");
        let violations = scan_content("fixtures/four_backtick.md", &content);
        assert!(
            !violations.is_empty(),
            "Expected violations inside 4-backtick sh2 fence but found none"
        );
        // The justified one should pass, only unjustified should fail
        assert_eq!(
            violations.len(),
            1,
            "Expected exactly 1 violation, got {}",
            violations.len()
        );
    }

    #[test]
    fn fixture_trailing_spaces_fence() {
        let content = fixture_content("trailing_spaces.md");
        let violations = scan_content("fixtures/trailing_spaces.md", &content);
        assert!(
            !violations.is_empty(),
            "Expected violations inside fence with trailing spaces but found none"
        );
    }

    #[test]
    fn fixture_wrong_marker_rejected() {
        // Uses old-style sh() because: (without ...) — should NOT be accepted
        let content = fixture_content("wrong_marker.md");
        let violations = scan_content("fixtures/wrong_marker.md", &content);
        assert!(
            !violations.is_empty(),
            "Expected violations for sh() because: (wrong marker) but found none"
        );
    }
}
