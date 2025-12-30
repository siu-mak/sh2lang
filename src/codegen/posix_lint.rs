/// POSIX compatibility linting for generated shell scripts
/// Detects bash-only constructs that would fail under strict POSIX shells

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosixLint {
    pub kind: PosixLintKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PosixLintKind {
    DoubleBracketTest,
    BashArithmetic,
    Arrays,
    AssociativeArrays,
    LocalOrDeclare,
    ProcessSubstitution,
    Pipefail,
    BraceExpansion,
    HereString,
    Other(String),
}

impl PosixLint {
    fn new(kind: PosixLintKind, message: String) -> Self {
        Self { kind, message }
    }
}

/// Scan generated shell script for bash-only constructs
pub fn lint_script(script: &str) -> Vec<PosixLint> {
    let mut lints = Vec::new();
    
    // Check for double-bracket test
    if script.contains("[[") || script.contains("]]") {
        lints.push(PosixLint::new(
            PosixLintKind::DoubleBracketTest,
            "posix-lint: bash-only construct detected: [[ ... ]]".to_string(),
        ));
    }
    
    // Check for bash arithmetic command (( )), but not $((  )) which is POSIX
    // Look for (( at start of line or after whitespace/semicolon, not preceded by $
    for line in script.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        // Check for standalone (( not preceded by $
        if trimmed.starts_with("((") || trimmed.contains(" ((") || trimmed.contains(";((") {
            // Make sure it's not $((
            if !trimmed.contains("$((") {
                lints.push(PosixLint::new(
                    PosixLintKind::BashArithmetic,
                    "posix-lint: bash-only construct detected: (( ... ))".to_string(),
                ));
                break;
            }
        }
    }
    
    // Check for declare/local (more precise: check for actual usage, not just substring)
    // Look for lines that start with declare or local after whitespace
    for line in script.lines() {
        let trimmed = line.trim();
        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with("declare ") || trimmed.starts_with("local ") {
            lints.push(PosixLint::new(
                PosixLintKind::LocalOrDeclare,
                "posix-lint: bash-only construct detected: declare/local".to_string(),
            ));
            break; // Only report once
        }
    }
    
    // Check for array syntax
    if script.contains("[@]") || script.contains("[*]") || script.contains("=(") {
        lints.push(PosixLint::new(
            PosixLintKind::Arrays,
            "posix-lint: bash-only construct detected: array syntax".to_string(),
        ));
    }
    
    // Check for associative arrays
    if script.contains("declare -A") {
        lints.push(PosixLint::new(
            PosixLintKind::AssociativeArrays,
            "posix-lint: bash-only construct detected: associative arrays".to_string(),
        ));
    }
    
    // Check for process substitution
    if script.contains("<(") || script.contains(">(") {
        lints.push(PosixLint::new(
            PosixLintKind::ProcessSubstitution,
            "posix-lint: bash-only construct detected: process substitution".to_string(),
        ));
    }
    
    // Check for pipefail
    if script.contains("set -o pipefail") {
        lints.push(PosixLint::new(
            PosixLintKind::Pipefail,
            "posix-lint: bash-only construct detected: set -o pipefail".to_string(),
        ));
    }
    
    // Check for here-string
    if script.contains("<<<") {
        lints.push(PosixLint::new(
            PosixLintKind::HereString,
            "posix-lint: bash-only construct detected: here-string <<<".to_string(),
        ));
    }
    
    lints
}

/// Render lint errors as a user-friendly error message
pub fn render_lints(lints: &[PosixLint]) -> String {
    let mut msg = String::from("error: POSIX target emitted bash-only shell constructs:\n");
    
    for lint in lints {
        msg.push_str("  - ");
        msg.push_str(&lint.message);
        msg.push('\n');
    }
    
    msg.push_str("hint: use --target bash, or avoid the feature that requires bash.");
    msg
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_double_bracket() {
        let script = r#"if [[ -n "$x" ]]; then echo "y"; fi"#;
        let lints = lint_script(script);
        assert!(!lints.is_empty());
        assert!(lints.iter().any(|l| matches!(l.kind, PosixLintKind::DoubleBracketTest)));
    }
    
    #[test]
    fn test_local_declare() {
        let script = "local x=1";
        let lints = lint_script(script);
        assert!(lints.iter().any(|l| matches!(l.kind, PosixLintKind::LocalOrDeclare)));
    }
    
    #[test]
    fn test_clean_posix() {
        let script = r#"x=1; [ -n "$x" ] && echo "$x""#;
        let lints = lint_script(script);
        assert!(lints.is_empty());
    }
}
