mod common;
use common::assert_parse_error_matches_snapshot;

#[test]
fn parse_err_unexpected_token() {
    assert_parse_error_matches_snapshot("parse_err_unexpected_token");
}

#[test]
fn parse_err_missing_rbrace_try() {
    assert_parse_error_matches_snapshot("parse_err_missing_rbrace_try");
}

#[test]
fn parse_err_eof_after_keyword() {
    assert_parse_error_matches_snapshot("parse_err_eof_after_keyword");
}

#[test]
fn parse_err_unterminated_string() {
    assert_parse_error_matches_snapshot("parse_err_unterminated_string");
}
