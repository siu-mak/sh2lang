mod common;
use common::run_test_in_targets;
use std::io::Write;

#[test]
fn test_contains_line_needle_starts_with_dash() {
    let mut tf = tempfile::NamedTempFile::new().expect("create tempfile");
    write!(tf, "-option\nregular line\n--double-dash\n").expect("write tempfile");
    let path = tf.path().to_str().expect("path utf8");

    let code = format!(r#"
        func main() {{
            let tf = r"{}"
            if contains_line(tf, "-option") {{
                print("found dash")
            }} else {{
                print("not found dash")
            }}
            
            if contains_line(tf, "--double-dash") {{
                print("found double") 
            }} else {{
                print("not found double")
            }}
        }}
    "#, path);

    run_test_in_targets("dash_needle", &code, "found dash\nfound double");
}
