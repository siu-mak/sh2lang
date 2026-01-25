



mod common;

#[test]
fn test_missing_whitespace_around_concat() {
    // Repro case: env.HOME&"/x"
    let src = r#"
        func main() {
            let p = env.HOME&"/x"
        }
    "#;
    
    let res = common::try_compile_to_shell(src, common::TargetShell::Bash);
    
    // We expect this to fail because we are enforcing whitespace around &
    match res {
        Err(msg) => {
            assert!(msg.contains("requires whitespace"), "Error should contain whitespace hint, got: {}", msg);
        }
        Ok(_) => {
            panic!("Expected compilation to fail due to missing whitespace around &");
        }
    }
}
