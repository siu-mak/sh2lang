use sh2c::ast::StmtKind;
mod common;
use common::*;
use sh2c::ast::Stmt;

#[test]
fn parse_pipe_blocks_basic() {
    let program = parse_fixture("pipe_blocks_basic");
    let func = &program.functions[0];

    // Check Stmt::Pipe(segments)
    if let Stmt {
        node: StmtKind::Pipe(segments),
        ..
    } = &func.body[0]
    {
        assert_eq!(segments.len(), 2);
        // Seg 0: print("a"), print("b") - Block
        if let sh2c::ast::PipeSegment::Block(stmts) = &segments[0].node {
             assert_eq!(stmts.len(), 2);
        } else {
             panic!("Expected Block for segment 0");
        }
        
        // Seg 1: run("grep", "b") - Block (due to {})
        if let sh2c::ast::PipeSegment::Block(stmts) = &segments[1].node {
             assert_eq!(stmts.len(), 1);
        } else {
             panic!("Expected Block for segment 1");
        }
    } else {
        panic!("Expected Stmt::Pipe");
    }
}

#[test]
fn codegen_pipe_blocks_basic() {
    assert_codegen_matches_snapshot("pipe_blocks_basic");
}

#[test]
fn exec_pipe_blocks_basic() {
    assert_exec_matches_fixture("pipe_blocks_basic");
}
