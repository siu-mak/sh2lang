mod common;
use common::run_test_in_targets;

#[test]
fn test_bool_encoding_output() {
    let code = r#"
        func main() {
            let t = true
            let f = false
            print(t)
            print(f)
        }
    "#;
    // We expect explicit "true" and "false" strings
    run_test_in_targets("bool_print", code, "true\nfalse");
}

#[test]
fn test_bool_var_condition() {
    let code = r#"
        func main() {
            let t = true
            let f = false
            if t {
                print("t is true")
            }
            if !f {
                print("f is false")
            }
        }
    "#;
    run_test_in_targets("bool_cond", code, "t is true\nf is false");
}

#[test]
fn test_bool_from_expression() {
    let code = r#"
        func main() {
            let is_match = contains("abc", "b")
            if is_match {
                print("match")
            }
        }
    "#;
    run_test_in_targets("bool_expr", code, "match");
}
