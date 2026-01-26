mod common;

#[test]
fn test_cwd_non_literal_error() {
    common::check_err_contains(
        "with_cwd_non_literal_is_error",
        "cwd(...) requires a string literal path. Computed expressions are not allowed."
    );
}

#[test]
fn test_cwd_number_error() {
    common::check_err_contains(
        "with_cwd_non_string_literal_number_is_error",
        "cwd(...) requires a string literal path. Computed expressions are not allowed."
    );
}

#[test]
fn test_cwd_var_error() {
    common::check_err_contains(
        "with_cwd_var_is_error",
        "cwd(...) requires a string literal path. Computed expressions are not allowed."
    );
}

#[test]
fn test_cwd_bool_error() {
    common::check_err_contains(
        "with_cwd_non_string_literal_bool_is_error",
        "cwd(...) requires a string literal path. Computed expressions are not allowed."
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
