mod common;
use common::*;
use sh2c::lexer::lex;

#[test]
fn exec_multiline_cooked_basic() {
    assert_exec_matches_fixture_target("multiline_cooked_basic", TargetShell::Bash);
    assert_exec_matches_fixture_target("multiline_cooked_basic", TargetShell::Posix);
}

#[test]
fn exec_multiline_raw_basic() {
    assert_exec_matches_fixture_target("multiline_raw_basic", TargetShell::Bash);
    assert_exec_matches_fixture_target("multiline_raw_basic", TargetShell::Posix);
}

#[test]
fn exec_multiline_mixed_quotes() {
    assert_exec_matches_fixture_target("multiline_mixed_quotes", TargetShell::Bash);
    assert_exec_matches_fixture_target("multiline_mixed_quotes", TargetShell::Posix);
}

#[test]
#[should_panic(expected = "Unterminated triple-quoted string")]
fn parse_unterminated_cooked() {
    // Just lexing should panic
    let src = std::fs::read_to_string("tests/fixtures/multiline_unterminated.sh2").unwrap();
    use sh2c::span::SourceMap;
    let sm = SourceMap::new(src.clone());
    let _ = lex(&sm, &src);
}

#[test]
fn codegen_multiline_cooked_basic() {
    assert_codegen_matches_snapshot("multiline_cooked_basic");
}

#[test]
fn codegen_multiline_raw_basic() {
    assert_codegen_matches_snapshot("multiline_raw_basic");
}

#[test]
fn codegen_multiline_mixed_quotes() {
    assert_codegen_matches_snapshot("multiline_mixed_quotes");
}
