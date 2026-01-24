mod common;

#[test]
fn test_arg_dynamic_loop() {
    common::assert_exec_matches_fixture("arg_dynamic_loop");
}

#[test]
fn test_arg_variable_index() {
    let src = r#"
        func main() {
            let i = 1
            print(arg(i))
        }
    "#;
    let result = common::try_compile_to_shell(src, common::TargetShell::Bash);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}

#[test]
fn test_arg_expression_index() {
    let src = r#"
        func main() {
            let idx = 1 + 1
            print(arg(idx))
        }
    "#;
    let result = common::try_compile_to_shell(src, common::TargetShell::Bash);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}

#[test]
fn test_arg_literal_still_works() {
    let src = r#"
        func main() {
            print(arg(1))
        }
    "#;
    let result = common::try_compile_to_shell(src, common::TargetShell::Bash);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}
