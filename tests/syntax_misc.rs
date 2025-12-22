mod common;
use common::assert_codegen_matches_snapshot;

#[test]
fn codegen_comments() { assert_codegen_matches_snapshot("comments"); }

#[test]
fn codegen_nothing() { assert_codegen_matches_snapshot("nothing"); }
