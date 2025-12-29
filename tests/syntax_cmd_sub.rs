mod common;
use common::assert_codegen_matches_snapshot;

// Note: Parsing of capture(...) is covered by Expr tests usually, or implicitly here if we add specific parser tests.

#[test]
fn codegen_cmd_sub() {
    assert_codegen_matches_snapshot("cmd_sub");
}
#[test]
fn codegen_cmd_sub_args() {
    assert_codegen_matches_snapshot("cmd_sub_args");
}
#[test]
fn codegen_cmd_sub_call() {
    assert_codegen_matches_snapshot("cmd_sub_call");
}
#[test]
fn codegen_cmd_sub_call_args() {
    assert_codegen_matches_snapshot("cmd_sub_call_args");
}
#[test]
fn codegen_cmd_sub_pipe() {
    assert_codegen_matches_snapshot("cmd_sub_pipe");
}
#[test]
fn codegen_capture_basic() {
    assert_codegen_matches_snapshot("capture_basic");
}
#[test]
fn codegen_capture_args() {
    assert_codegen_matches_snapshot("capture_args");
}
