use crate::ir::Val;

/// Escape single quotes within a string literal for safe shell quoting.
/// Replaces ' with '\'' and wraps in '...'
pub(super) fn sh_single_quote(s: &str) -> String {
    let mut out = String::from("'");
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

/// Check if a Val represents a boolean expression (comparison, logical op, predicate, etc.)
/// These require special handling when assigned to variables.
pub(super) fn is_boolean_val(v: &Val) -> bool {
    matches!(
        v,
        Val::Bool(_)
            | Val::Compare { .. }
            | Val::And(_, _)
            | Val::Or(_, _)
            | Val::Not(_)
            | Val::Exists(_)
            | Val::IsDir(_)
            | Val::IsFile(_)
            | Val::IsSymlink(_)
            | Val::IsExec(_)
            | Val::IsReadable(_)
            | Val::IsWritable(_)
            | Val::IsNonEmpty(_)
            | Val::Matches(_, _)
            | Val::StartsWith { .. }
            | Val::ContainsList { .. }
            | Val::ContainsSubstring { .. }
            | Val::ContainsLine { .. }
            | Val::Confirm { .. }
    )
}

pub(super) fn is_boolean_expr(v: &Val) -> bool {
    matches!(
        v,
        Val::Compare { .. }
            | Val::And(..)
            | Val::Or(..)
            | Val::Not(..)
            | Val::Exists(..)
            | Val::IsDir(..)
            | Val::IsFile(..)
            | Val::IsSymlink(..)
            | Val::IsExec(..)
            | Val::IsReadable(..)
            | Val::IsWritable(..)
            | Val::IsNonEmpty(..)
            | Val::Bool(..)
            | Val::Matches(..)
    )
}

pub(super) fn emit_case_glob_pattern(glob: &str) -> String {
    let mut out = String::new();
    let mut literal_buf = String::new();

    for c in glob.chars() {
        if c == '*' || c == '?' {
            if !literal_buf.is_empty() {
                out.push_str(&sh_single_quote(&literal_buf));
                literal_buf.clear();
            }
            out.push(c);
        } else {
            literal_buf.push(c);
        }
    }
    if !literal_buf.is_empty() {
        out.push_str(&sh_single_quote(&literal_buf));
    }

    if out.is_empty() {
        return "''".to_string();
    }
    out
}
