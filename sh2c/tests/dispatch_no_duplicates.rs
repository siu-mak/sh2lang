use common::{assert_exec_matches_fixture_target, compile_path_to_shell, repo_root, TargetShell};

mod common;

#[test]
fn test_dispatch_no_duplicates() {
    // 1. Run execution test to confirm runtime behavior
    assert_exec_matches_fixture_target("dispatch_no_duplicates", TargetShell::Bash);
    assert_exec_matches_fixture_target("dispatch_no_duplicates", TargetShell::Posix);

    let targets = [
        (TargetShell::Bash, "Bash"),
        (TargetShell::Posix, "Posix")
    ];
    
    let fixture = repo_root().join("sh2c/tests/fixtures/dispatch_no_duplicates.sh2");

    for (target, target_name) in targets {
        let compiled = compile_path_to_shell(&fixture, target);
        
        for term in ["install", "dirs", "registry"] {
            // Count lines that look like a conditional check for this term.
            // Robust logic:
            // - Must contain 'if' or 'elif'
            // - Must contain sub variable ($sub or ${sub})
            // - Must contain the term string literal ("term" or 'term')
            // - Must contain a test operator ([ or [[ or test)
            
            let count = compiled.lines()
                .filter(|line| {
                    let l = line.trim();
                    let has_ctrl = l.starts_with("if") || l.starts_with("elif");
                    let has_var = l.contains("$sub") || l.contains("${sub}");
                    let has_term = l.contains(&format!("'{}'", term)) || l.contains(&format!("\"{}\"", term));
                    let has_op = l.contains("[") || l.contains("test ");
                    
                    has_ctrl && has_var && has_term && has_op
                })
                .count();

            if count != 1 {
                 // Context for failure
                 let context: String = compiled.lines()
                    .filter(|l| l.contains(term) || l.contains("if") || l.contains("elif"))
                    .take(20)
                    .collect::<Vec<_>>()
                    .join("\n");

                panic!(
                    "{}: Expected exactly 1 conditional check for '{}', found {}.\n(This fails if duplicate 'if' blocks are emitted for the same source condition)\nContext:\n{}\n", 
                    target_name, term, count, context
                );
            }
        }
    }
}
