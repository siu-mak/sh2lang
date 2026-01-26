use sh2c::ast::{Stmt, StmtKind, PipeSegment};
mod common;
use common::*;

#[test]
fn parse_pipe_sudo_basic() {
    let program = parse_fixture("pipe_sudo_basic");
    let func = &program.functions[0];
    
    // 3 pipelines
    assert_eq!(func.body.len(), 3);

    // 1. run | sudo
    if let Stmt { node: StmtKind::Pipe(segments), .. } = &func.body[0] {
        assert_eq!(segments.len(), 2);
        assert!(matches!(segments[0].node, PipeSegment::Run(_)));
        assert!(matches!(segments[1].node, PipeSegment::Sudo(_)));
    } else {
        panic!("Expected Pipe for stmt 0");
    }

    // 2. sudo | run
    if let Stmt { node: StmtKind::Pipe(segments), .. } = &func.body[1] {
        assert_eq!(segments.len(), 2);
        assert!(matches!(segments[0].node, PipeSegment::Sudo(_)));
        assert!(matches!(segments[1].node, PipeSegment::Run(_)));
    } else {
        panic!("Expected Pipe for stmt 1");
    }

    // 3. block | sudo
    if let Stmt { node: StmtKind::Pipe(segments), .. } = &func.body[2] {
        assert_eq!(segments.len(), 2);
        assert!(matches!(segments[0].node, PipeSegment::Block(_)));
        assert!(matches!(segments[1].node, PipeSegment::Sudo(_)));
    } else {
        panic!("Expected Pipe for stmt 2");
    }
}

#[test]
fn codegen_pipe_sudo_basic() {
    assert_codegen_matches_snapshot("pipe_sudo_basic");
}

// No exec test - sudo requires privileges or interactive tty often unavailable in CI
