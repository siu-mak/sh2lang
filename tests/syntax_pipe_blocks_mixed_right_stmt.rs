use sh2c::ast::StmtKind;
mod common;
use common::*;
use sh2c::ast::Stmt;

#[test]
fn parse_pipe_blocks_mixed_right_stmt() {
    let program = parse_fixture("pipe_blocks_mixed_right_stmt");
    let func = &program.functions[0];

    // Check Stmt::PipeBlocks { segments }
    if let Stmt {
        kind: StmtKind::PipeBlocks { segments },
        ..
    } = &func.body[0]
    {
        assert_eq!(segments.len(), 2);
        // Seg 0: { print("a") print("b") } (2 stmts)
        assert_eq!(segments[0].len(), 2);
        // Seg 1: run("grep", "b") (1 stmt)
        assert_eq!(segments[1].len(), 1);
        if let Stmt {
            kind: StmtKind::Run(..),
            ..
        } = &segments[1][0]
        {
            // OK
        } else {
            panic!("Expected Run in segment 1");
        }
    } else {
        panic!("Expected Stmt::PipeBlocks");
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
