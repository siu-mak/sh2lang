
use std::process::Command;

#[test]
fn test_sh2do_semicolon_oneliner() {
    let output = Command::new(env!("CARGO_BIN_EXE_sh2do"))
        .arg("print(\"a\"); print(\"b\")")
        .output()
        .expect("Failed to run sh2do");

    assert!(output.status.success(), "Execution failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("a"), "Missing 'a'");
    assert!(stdout.contains("b"), "Missing 'b'");
    
    // Check order roughly (output might vary by shell impl but "a" before "b" is expected)
    let a_pos = stdout.find("a").unwrap();
    let b_pos = stdout.find("b").unwrap();
    assert!(a_pos < b_pos, "Output out of order: {}", stdout);
}

#[test]
fn test_sh2do_semicolon_trailing() {
    let output = Command::new(env!("CARGO_BIN_EXE_sh2do"))
        .arg("print(\"ok\");")
        .output()
        .expect("Failed to run sh2do");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ok"));
}
