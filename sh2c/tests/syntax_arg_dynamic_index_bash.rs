use common::{assert_exec_matches_fixture_target, TargetShell, compile_path_to_shell, repo_root};

mod common;

#[test]
fn test_arg_dynamic_index_bash() {
    // 1. Run execution test with args (loaded from .args file)
    assert_exec_matches_fixture_target("arg_dynamic_index", TargetShell::Bash);

    // 2. Check compiled output for correct usage of __sh2_arg_by_index
    let fixture = repo_root().join("sh2c/tests/fixtures/arg_dynamic_index.sh2");
    let compiled = compile_path_to_shell(&fixture, TargetShell::Bash);
    
    // Positive assertions:
    // arg(i) -> "$i"
    if !compiled.contains("__sh2_arg_by_index \"$i\" \"$@\"") {
        panic!("Missing expected variable index emission: __sh2_arg_by_index \"$i\" \"$@\"");
    }
    if !compiled.contains("__sh2_arg_by_index \"$(( ( i + 2 ) ))\" \"$@\"") {
         panic!("Missing expected canonical arithmetic index emission for 'i + 2'. Compiled:\n{}", compiled);
    }
    // arg(4) -> "$4" (optimization)
    if !compiled.contains("\"$4\"") {
          panic!("Missing expected optimized literal index emission for '4' -> \"$4\"");
    }

    // Negative assertions (bad raw tokens):
    if compiled.contains("__sh2_arg_by_index i \"$@\"") {
        panic!("Found unsafe raw variable token as index");
    }
    if compiled.contains("__sh2_arg_by_index 4 \"$@\"") {
        // Technically "4" without quotes is okay for non-strict shell, but we want consistency.
        // Actually, "4" is safe. But "i" is not if i contains spaces (variable name does not, but value might be expanded if not careful context). Use quotes.
        // Wait, "4" literal is always safe.
        // But the plan preference was quotes. Let's enforce quotes for consistency.
         panic!("Found unquoted literal index '4' (prefer quoted)");
    }
    if compiled.contains("__sh2_arg_by_index (") {
         panic!("Found raw parenthesis start (likely unquoted arithmetic)");
    }
}
