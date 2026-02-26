use std::process::Command;
use std::path::PathBuf;

fn sh2c_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sh2c"))
}

fn assert_diag_output(fixture: &str) {
    let fixture_path = PathBuf::from("tests/fixtures").join(fixture);
    let expected_path = fixture_path.with_extension("stderr.expected");
    
    let output = Command::new(sh2c_path())
        .arg(&fixture_path)
        .output()
        .expect("Failed to run sh2c");
        
    assert!(!output.status.success(), "sh2c should fail");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.lines()
        .map(|l| { eprintln!("LINE: {:?}", l); l }) 
        .filter(|l| !l.contains("thread '") && !l.contains("panicked at") && !l.contains("note: run with"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();
    
    if std::env::var("SH2C_UPDATE_SNAPSHOTS").is_ok() {
        std::fs::write(&expected_path, &stderr).expect("Failed to update snapshot");
    }
    
    let expected = std::fs::read_to_string(&expected_path).expect("Missing expected file");
    let expected = expected.replace("\r\n", "\n").trim().to_string();
    
    assert_eq!(stderr, expected, "Stderr mismatch for {}", fixture);
}

#[test]
fn diag_span_try_run_expr_invalid() {
    assert_diag_output("diag_span_try_run_expr_invalid.sh2");
}

#[test]
fn diag_span_bad_option_loc() {
    assert_diag_output("diag_span_bad_option_loc.sh2");
}

#[test]
fn diag_span_multiline_expr() {
    assert_diag_output("diag_span_multiline_expr.sh2");
}

#[test]
fn unknown_function_is_error() {
    assert_diag_output("unknown_function_is_error.sh2");
}
#[test]
fn diag_span_let_redecl() {
    assert_diag_output("diag_span_let_redecl.sh2");
}

#[test]
fn diag_span_set_undecl() {
    assert_diag_output("diag_span_set_undecl.sh2");
}

#[test]
fn diag_span_for_redecl() {
    assert_diag_output("diag_span_for_redecl.sh2");
}

#[test]
fn diag_span_each_line_redecl() {
    assert_diag_output("diag_span_each_line_redecl.sh2");
}

#[test]
fn diag_span_namespaced_call_err_unknown_alias() {
    assert_diag_output("namespaced_call/err_unknown_alias.sh2");
}

#[test]
fn diag_span_namespaced_call_err_unknown_func() {
    assert_diag_output("namespaced_call/err_unknown_func.sh2");
}

#[test]
fn diag_span_namespaced_call_err_missing_paren() {
    assert_diag_output("namespaced_call/err_missing_paren.sh2");
}

#[test]
fn diag_span_namespaced_call_suggest_alias() {
    assert_diag_output("namespaced_call/err_suggest_alias.sh2");
}

#[test]
fn diag_span_namespaced_call_suggest_func() {
    assert_diag_output("namespaced_call/err_suggest_func.sh2");
}
