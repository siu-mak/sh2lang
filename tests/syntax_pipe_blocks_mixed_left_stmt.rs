mod common;
use common::*;

#[test]
fn parse_pipe_blocks_mixed_left_stmt_fail() {
    // This syntax `run | { ... }` is currently not supported and should strictly fail.
    let result = std::panic::catch_unwind(|| {
        parse_fixture("pipe_blocks_mixed_left_stmt")
    });
    assert!(result.is_err(), "Parser should panic on mixed pipe/block syntax");
    
    let err = result.err().unwrap();
    let msg = if let Some(s) = err.downcast_ref::<&str>() {
        *s
    } else if let Some(s) = err.downcast_ref::<String>() {
        s.as_str()
    } else {
        "Unknown panic message"
    };
    
    assert_eq!(msg, "expected run(...) after '|' in pipeline");
}

// Codegen and Exec tests removed because parsing is expected to fail.