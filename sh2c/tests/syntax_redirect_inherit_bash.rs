use common::{assert_exec_matches_fixture_target, TargetShell, compile_path_to_shell, repo_root};

mod common;

#[test]
fn test_redirect_inherit_bash() {
    // 1. Run execution test to confirm runtime behavior (and see if it fails due to syntax or "unary operator expected")
    assert_exec_matches_fixture_target("redirect_inherit_basic", TargetShell::Bash);

    // 2. Check compiled output for literal \n
    let fixture = repo_root().join("sh2c/tests/fixtures/redirect_inherit_basic.sh2");
    let compiled = compile_path_to_shell(&fixture, TargetShell::Bash);
    
    // Precise Scanning:
    // Only fail if literal "\\n" appears in generated glue code lines.
    // We identify glue code by variables/constructs used in the redirect/inherit transformation.
    let glue_indicators = [
        "__sh2_cs_", 
        "__sh2_ts_", 
        "__sh2_fifo_", 
        "wait ", 
        "__sh2_final_"
    ];

    for (i, line) in compiled.lines().enumerate() {
        let is_glue_line = glue_indicators.iter().any(|ind| line.contains(ind));
        
        if is_glue_line {
            if line.contains("\\n") {
                 let start = i.saturating_sub(2);
                 let end = (i + 3).min(compiled.lines().count());
                 let context = compiled.lines().skip(start).take(end - start).collect::<Vec<_>>().join("\n");
                 
                 panic!(
                     "Found disallowed literal '\\n' in generated glue code on line {}:\n\n> {}\n\nContext:\n{}", 
                     i + 1, 
                     line.trim(),
                     context
                 );
            }
        }
    }
}
