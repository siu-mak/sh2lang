mod common;
use common::*;
use sh2c::ast::{Stmt};

#[test]
fn parse_pipe_blocks_mixed_left_stmt() {
    let program = parse_fixture("pipe_blocks_mixed_left_stmt");
    let func = &program.functions[0];
    
    // Check Stmt::PipeBlocks { segments }
    if let Stmt::PipeBlocks { segments } = &func.body[0] {
        assert_eq!(segments.len(), 2);
        // Seg 0: run("printf", ...) (1 stmt)
        assert_eq!(segments[0].len(), 1);
        if let Stmt::Run(..) = &segments[0][0] {
             // OK
        } else {
             panic!("Expected Run in segment 0");
        }
        // Seg 1: { run("grep", "b") } (1 stmt)
        assert_eq!(segments[1].len(), 1);
        if let Stmt::Run(..) = &segments[1][0] {
              // OK
        } else {
             panic!("Expected Run in segment 1");
        }
    } else {
        panic!("Expected Stmt::PipeBlocks");
    }
}

#[test]
fn codegen_pipe_blocks_mixed_left_stmt() {
    assert_codegen_matches_snapshot("pipe_blocks_mixed_left_stmt");
}

#[test]
fn exec_pipe_blocks_mixed_left_stmt() {
    assert_exec_matches_fixture("pipe_blocks_mixed_left_stmt");
}
