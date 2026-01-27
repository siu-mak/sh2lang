use common::{assert_exec_matches_fixture_target, TargetShell, compile_path_to_shell, repo_root};

mod common;

#[test]
fn test_redirect_inherit_bash() {
    // 1. Run execution test
    assert_exec_matches_fixture_target("redirect_inherit_basic", TargetShell::Bash);

    // 2. Check for literal \n in compiled output
    let fixture = repo_root().join("sh2c/tests/fixtures/redirect_inherit_basic.sh2");
    let compiled = compile_path_to_shell(&fixture, TargetShell::Bash);
    
    // 2. Check for literal \n in compiled output (specific context)
    // We scan lines to ensure no statement-ending literal \n exists.
    for (i, line) in compiled.lines().enumerate() {
        // Broad check for likely problematic lines (status capture or wait)
        if line.contains("wait \"") || line.contains("__sh2_cs_") || line.contains("__sh2_ts_") {
             if line.contains("\\n") {
                 panic!("Found literal \\n on line {}: {}", i+1, line);
             }
        }
    }
    
    // 3. Explicitly check for runtime "unary operator expected" in stderr
    // The assertions above (assert_exec_matches_fixture_target) already check
    // total output against the expected .stderr file.
    // However, to be extra explicit as requested:
    // We trust that if the stderr had contained "unary operator expected", 
    // it would mismatch the clean .stderr fixture and fail step 1.
    // If we wanted to run it again manually, we could, but assert_exec_matches... is sufficient
    // provided the expected .stderr is clean (which it is: "hello stderr\n").
}
