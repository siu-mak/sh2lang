mod common;
use common::run_test_in_targets;
use std::io::Write;

#[test]
fn test_contains_line_file_has_exact_line() {
    let mut tf = tempfile::NamedTempFile::new().expect("create tempfile");
    write!(tf, "registry.example.com\nother.example.com\n").expect("write tempfile");
    let path = tf.path().to_str().expect("path utf8");
    
    // Inject the absolute tempfile path into the sh2 source safe logic
    let code = format!(r#"
        func main() {{
            let tf = r"{}"
            if contains_line(tf, "registry.example.com") {{
                print("found")
            }} else {{
                print("not found")
            }}
        }}
    "#, path);

    run_test_in_targets("exact_match", &code, "found");
    // tf is dropped here, file deleted
}

#[test]
fn test_contains_line_file_missing_line() {
    let mut tf = tempfile::NamedTempFile::new().expect("create tempfile");
    write!(tf, "registry.example.com\nother.example.com\n").expect("write tempfile");
    let path = tf.path().to_str().expect("path utf8");

    let code = format!(r#"
        func main() {{
            let tf = r"{}"
            if contains_line(tf, "missing.example.com") {{
                print("found")
            }} else {{
                print("not found")
            }}
        }}
    "#, path);

    run_test_in_targets("missing_line", &code, "not found");
}

#[test]
fn test_contains_line_exact_not_substring() {
    let mut tf = tempfile::NamedTempFile::new().expect("create tempfile");
    write!(tf, "registry.example.com\nother.example.com\n").expect("write tempfile");
    let path = tf.path().to_str().expect("path utf8");

    let code = format!(r#"
        func main() {{
            let tf = r"{}"
            # "registry" is a substring but not a full line
            if contains_line(tf, "registry") {{
                print("found")
            }} else {{
                print("not found")
            }}
        }}
    "#, path);

    run_test_in_targets("substring_check", &code, "not found");
}

#[test]
fn test_contains_line_empty_file() {
    let tf = tempfile::NamedTempFile::new().expect("create tempfile");
    // empty file
    let path = tf.path().to_str().expect("path utf8");

    let code = format!(r#"
        func main() {{
            let tf = r"{}"
            if contains_line(tf, "anything") {{
                print("found")
            }} else {{
                print("not found")
            }}
        }}
    "#, path);

    run_test_in_targets("empty_file", &code, "not found");
}

