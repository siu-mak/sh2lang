//! Canonical registry of sh2 builtin functions.
//!
//! This module provides the single source of truth for which function names
//! are handled specially by the compiler (in lowering or codegen).
//!
//! - EXPR_BUILTINS: lowered to specialized IR nodes in lower_expr (never reach fallback)
//! - PRELUDE_HELPERS: pass through to ir::Val::Call, handled by codegen with __sh2_ prefix

use std::collections::HashSet;
use std::sync::LazyLock;

/// Expression-level builtins handled specially in lower_expr.
/// These become specialized IR nodes (not ir::Val::Call).
/// If any of these reach the fallback branch in lower_expr, it's a compiler bug.
pub const EXPR_BUILTINS: &[&str] = &[
    "argv",
    "matches",
    "contains",
    "contains_line",
    "starts_with",
    "parse_args",
    "load_envfile",
    "json_kv",
    "which",
    "try_run",
    "require",
    "read_file",
    "write_file",
    "append_file",
    "log_info",
    "log_warn",
    "log_error",
    "home",
    "path_join",
    "lines",
    "split", // lowers to ir::Val::Split
    "save_envfile",
];

/// Prelude helper functions that pass through to ir::Val::Call.
/// Codegen handles these by prefixing with __sh2_.
pub const PRELUDE_HELPERS: &[&str] = &[
    "trim",
    "before",
    "after",
    "replace",
    "coalesce",
    "default", // alias for coalesce
];

/// All valid callable builtin names (union of EXPR_BUILTINS and PRELUDE_HELPERS).
pub static ALL_BUILTINS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    EXPR_BUILTINS
        .iter()
        .chain(PRELUDE_HELPERS.iter())
        .copied()
        .collect()
});

/// Check if a name is a valid builtin function (any category).
pub fn is_builtin(name: &str) -> bool {
    ALL_BUILTINS.contains(name)
}

/// Check if a name is an expression-level builtin (should be handled specially in lower_expr).
pub fn is_expr_builtin(name: &str) -> bool {
    EXPR_BUILTINS.contains(&name)
}

/// Check if a name is a prelude helper (allowed to pass through to ir::Val::Call).
pub fn is_prelude_helper(name: &str) -> bool {
    PRELUDE_HELPERS.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_overlap_between_builtin_categories() {
        let expr_set: HashSet<_> = EXPR_BUILTINS.iter().copied().collect();
        let prelude_set: HashSet<_> = PRELUDE_HELPERS.iter().copied().collect();
        let overlap: Vec<_> = expr_set.intersection(&prelude_set).collect();
        assert!(
            overlap.is_empty(),
            "Builtin categories must not overlap: {:?}",
            overlap
        );
    }
}
