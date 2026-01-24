mod common;

#[test]
fn test_cwd_non_literal_error() {
    common::assert_codegen_panics(
        "with_cwd_non_literal_is_error", 
        "cwd(...) currently requires a string literal"
    );
}

#[test]
fn test_cwd_number_error() {
    common::assert_codegen_panics(
        "with_cwd_non_string_literal_number_is_error",
        "cwd(...) currently requires a string literal"
    );
}

#[test]
fn test_cwd_bool_error() {
    common::assert_codegen_panics(
        "with_cwd_non_string_literal_bool_is_error",
        "cwd(...) currently requires a string literal"
    );
}

#[test]
fn test_cwd_literal_ok() {
    // Should compile fine (existing behavior)
    let src = r#"
        func main() {
            with cwd("/tmp") {
                run("ls")
            }
        }
    "#;
    let _ = common::compile_to_bash(src);
}
