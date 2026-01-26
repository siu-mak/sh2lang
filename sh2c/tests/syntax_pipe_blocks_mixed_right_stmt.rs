use sh2c::ast::StmtKind;
mod common;
use common::*;
use sh2c::ast::Stmt;

#[test]
fn parse_pipe_blocks_mixed_right_stmt() {
    let program = parse_fixture("pipe_blocks_mixed_right_stmt");
    let func = &program.functions[0];

    // Check Stmt::Pipe { segments }
    if let Stmt {
        node: StmtKind::Pipe(segments),
        ..
    } = &func.body[0]
    {
        assert_eq!(segments.len(), 2);
        // Seg 0: { print("a") print("b") } (2 stmts) - Block
        if let sh2c::ast::PipeSegment::Block(stmts) = &segments[0].node {
             assert_eq!(stmts.len(), 2);
        } else {
            panic!("Expected Block for segment 0");
        }

        // Seg 1: run("grep", "b") - Run
        if let sh2c::ast::PipeSegment::Run(_) = &segments[1].node {
            // OK
        } else {
            panic!("Expected Run for segment 1");
        }
    } else {
        panic!("Expected Stmt::Pipe");
    }
}

#[test]
fn codegen_pipe_blocks_mixed_right_stmt() {
    assert_codegen_matches_snapshot("pipe_blocks_mixed_right_stmt");
}

#[test]
fn exec_pipe_blocks_mixed_right_stmt() {
    assert_exec_matches_fixture("pipe_blocks_mixed_right_stmt");
}
