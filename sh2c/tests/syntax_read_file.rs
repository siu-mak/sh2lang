mod common;

#[test]
fn test_read_file_basic() {
    common::assert_exec_matches_fixture("read_file_basic");
}

#[test]
fn test_read_file_multiline() {
    common::assert_exec_matches_fixture("read_file_multiline");
}

#[test]
fn test_read_file_trailing_newline() {
    common::assert_exec_matches_fixture("read_file_trailing_newline");
}

#[test]
fn test_read_file_missing_error() {
    common::assert_exec_matches_fixture("read_file_missing_error");
}
