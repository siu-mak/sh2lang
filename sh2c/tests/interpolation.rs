mod common;
use common::run_test_in_targets;

#[test]
fn test_interp_ident() {
    // Hardening tests
    
    // Escaped braces \{ \} -> Literal { }
    let code = r#"
        func main() {
            print($"Val: \{ \}")
        }
    "#;
    run_test_in_targets("interp_esc_braces", code, "Val: { }");

    // Literal backslash \\ -> \
    let code = r#"
        func main() {
            print($"Backslash: \\")
        }
    "#;
    run_test_in_targets("interp_esc_backslash", code, "Backslash: \\");
    
    // Valid expression
    let code = r#"
        func main() {
            let x = 10
            print($"Val: {x}")
        }
    "#;
    run_test_in_targets("interp_valid", code, "Val: 10");
}

#[test]
fn test_interp_arith() {
    let code = r#"
        func main() {
            print($"Sum: {1 + 2}")
        }
    "#;
    run_test_in_targets("interp_arith", code, "Sum: 3");
}

#[test]
fn test_interp_call() {
    let code = r#"
        func greet(name) {
            return $"Hello, {name}!"
        }
        func main() {
            print(greet("World"))
        }
    "#;
    run_test_in_targets("interp_call", code, "Hello, World!");
}

#[test]
fn test_interp_multiple_holes() {
    let code = r#"
        func main() {
            let a = 1
            let b = 2
            print($"{a} + {b} = {a + b}")
        }
    "#;
    run_test_in_targets("interp_multiple", code, "1 + 2 = 3");
}

#[test]
fn test_interp_complex_whitespace() {
    let code = r#"
        func main() {
            print($"  Spaces { 10 }  ")
        }
    "#;
    run_test_in_targets("interp_ws", code, "  Spaces 10  ");
}

#[test]
fn test_interp_concat_variables() {
    // Test string concatenation with variables (workaround for quotes in holes)
    let code = r#"
        func main() {
            let a = "Hello"
            let b = "World"
            print($"Concat: {a & b}")
        }
    "#;
    run_test_in_targets("interp_concat_vars", code, "Concat: HelloWorld");
}

#[test]
fn test_interp_escaped_braces() {
    let code = r#"
        func main() {
            print($"Literal \{ and \}")
        }
    "#;
    run_test_in_targets("interp_escape", code, "Literal { and }");
}

#[test]
fn test_interp_boolean_implicit() {
    let code = r#"
        func main() {
            print($"Bool: {true}")
        }
    "#;
    run_test_in_targets("interp_bool", code, "Bool: true");
}

#[test]
fn test_interp_escape_and_hole_mix() {
    // Mix of escaped braces and actual holes
    let code = r#"
        func main() {
            print($"A: \{ B: {1} C: \}")
        }
    "#;
    run_test_in_targets("escape_mix", code, "A: { B: 1 C: }");
}


// NEGATIVE TEST: Quotes inside holes are not supported
#[test]
fn test_interp_quote_in_hole_unsupported() {
    // This test verifies that quotes inside holes fail at compile time
    // with a clear, stable diagnostic message about the lexer limitation
    common::assert_parse_error_matches_snapshot("interp_quote_in_hole_unsupported");
}

// NEGATIVE TEST: Real missing brace should show original diagnostic
#[test]
fn test_interp_missing_brace() {
    // This test ensures that a genuinely missing `}` shows the correct
    // "Unterminated interpolation hole" diagnostic, not the lexer limitation message
    common::assert_parse_error_matches_snapshot("interp_missing_brace");
}

