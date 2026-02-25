/// Mangle a qualified call `alias.func` into a wrapper function name.
/// Used by resolver (to fill resolved_mangled) and loader (at D1 registration time).
pub(crate) fn mangle(alias: &str, func: &str) -> String {
    debug_assert!(alias.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
    debug_assert!(func.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
    format!("__imp_{}__{}", alias, func)
}
