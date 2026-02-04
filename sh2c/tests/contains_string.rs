mod common;
use common::{run_test_in_targets, run_test_bash_only};

#[test]
fn test_contains_string_substring() {
    let code = r#"
        func main() {
            if contains("bad host:5000", " ") {
                print("contains space")
            } else {
                print("no space")
            }
            
            if contains("goodhost:5000", " ") {
                print("contains space")
            } else {
                print("no space")
            }
        }
    "#;
    run_test_in_targets("substring_check", code, "contains space\nno space");
}

#[test]
fn test_contains_list_regression() {
    // List contains is Bash-only
    let code = r#"
        func main() {
            let xs = ["a", "b"]
            if contains(xs, "b") { print("found b") }
            if !contains(xs, "c") { print("not found c") }
        }
    "#;
    run_test_bash_only("list_check", code, "found b\nnot found c");
}

#[test]
fn test_contains_empty_needle() {
    // "contains(abc, "") == true"
    let code = r#"
        func main() {
            if contains("abc", "") { print("found empty") }
        }
    "#;
    run_test_in_targets("empty_needle", code, "found empty");
}

#[test]
fn test_contains_scalar_variable_regression() {
    let code = r#"
        func main() {
            let host = "bad host:5000"
            if contains(host, " ") {
                print("has space")
            } else {
                print("no space")
            }
        }
    "#;
    run_test_in_targets("scalar_var", code, "has space");
}

#[test]
fn test_contains_list_expression_temp() {
    // List contains is Bash-only
    let code = r#"
        func main() {
            if contains(split("a,b", ","), "b") {
                print("found b")
            } else {
                print("not found b")
            }
        }
    "#;
    run_test_bash_only("list_expr", code, "found b");
}

#[test]
fn test_contains_list_literal_temp() {
    // List contains is Bash-only
    let code = r#"
        func main() {
            if contains(["x", "y"], "y") {
                print("found y")
            } else {
                print("not found y")
            }
        }
    "#;
    run_test_bash_only("list_lit", code, "found y");
}

#[test]
fn test_contains_value_assignment() {
    let code = r#"
        func main() {
            let x = contains("abc", "b")
            print("x=" & x)
            let y = contains("abc", "z")
            print("y=" & y)
        }
    "#;
    run_test_in_targets("value_assign", code, "x=true\ny=false");
}

#[test]
fn test_contains_string_special_chars() {
    // Test special characters that could cause shell interpretation issues
    let code = r#"
        func main() {
            // Dollar sign
            if contains("a$b", "$") {
                print("found dollar")
            }
            
            // Brackets (glob chars)
            if contains("a[b]c", "[b]") {
                print("found brackets")
            }
            
            // Asterisk
            if contains("a*b*c", "*b*") {
                print("found asterisk")
            }
        }
    "#;
    run_test_in_targets(
        "contains_special",
        code,
        "found dollar\nfound brackets\nfound asterisk"
    );
}

#[test]
fn test_contains_string_needle_with_dash() {
    // Critical test: needles starting with - must work (POSIX -e flag fix)
    // Without -e flag, grep would interpret -b as a flag
    let code = r#"
        func main() {
            if contains("a-b-c", "-b") {
                print("found dash b")
            }
            
            if contains("test --flag", "--flag") {
                print("found double dash")
            }
            
            if contains("-start", "-s") {
                print("found dash start")
            }
        }
    "#;
    run_test_in_targets(
        "contains_dash_needle",
        code,
        "found dash b\nfound double dash\nfound dash start"
    );
}

#[test]
fn test_contains_string_backslash() {
    // Backslash handling
    let code = r#"
        func main() {
            if contains("a\\b", "\\") {
                print("found backslash")
            }
        }
    "#;
    run_test_in_targets("contains_backslash", code, "found backslash");
}
