mod common;
use common::assert_codegen_matches_snapshot;

#[test]
fn codegen_comments() {
    assert_codegen_matches_snapshot("comments");
}

#[test]
fn codegen_nothing_panics() {
    common::assert_codegen_panics(
        "nothing",
        "No entrypoint: define `func main()`.",
    );
}
