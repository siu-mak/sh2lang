use sh2c::ast::{Stmt, StmtKind, PipeSegment};
mod common;
use common::*;

#[test]
fn parse_pipe_block_producer() {
    let program = parse_fixture("pipe_block_producer_to_run_consumer");
    if let Stmt { node: StmtKind::Pipe(segments), .. } = &program.functions[0].body[0] {
        assert_eq!(segments.len(), 2);
        assert!(matches!(segments[0].node, PipeSegment::Block(_)));
        assert!(matches!(segments[1].node, PipeSegment::Run(_)));
    } else {
        panic!("Expected Pipe");
    }
}

#[test]
fn parse_pipe_block_consumer() {
    let program = parse_fixture("pipe_run_producer_to_block_consumer");
    if let Stmt { node: StmtKind::Pipe(segments), .. } = &program.functions[0].body[0] {
        assert_eq!(segments.len(), 2);
        assert!(matches!(segments[0].node, PipeSegment::Run(_)));
        assert!(matches!(segments[1].node, PipeSegment::Block(_)));
    } else {
        panic!("Expected Pipe");
    }
}

#[test]
fn parse_pipe_block_multi() {
    let program = parse_fixture("pipe_block_multi");
    if let Stmt { node: StmtKind::Pipe(segments), .. } = &program.functions[0].body[0] {
        assert_eq!(segments.len(), 3);
        assert!(matches!(segments[0].node, PipeSegment::Block(_)));
        assert!(matches!(segments[1].node, PipeSegment::Block(_)));
        assert!(matches!(segments[2].node, PipeSegment::Block(_)));
    } else {
        panic!("Expected Pipe");
    }
}

#[test]
fn codegen_pipe_block_producer() {
    assert_codegen_matches_snapshot("pipe_block_producer_to_run_consumer");
}

#[test]
fn codegen_pipe_block_consumer() {
    assert_codegen_matches_snapshot("pipe_run_producer_to_block_consumer");
}

#[test]
fn codegen_pipe_block_multi() {
    assert_codegen_matches_snapshot("pipe_block_multi");
}

#[test]
fn exec_pipe_block_producer() {
    assert_exec_matches_fixture("pipe_block_producer_to_run_consumer");
}

#[test]
fn exec_pipe_block_consumer() {
    assert_exec_matches_fixture("pipe_run_producer_to_block_consumer");
}

#[test]
fn exec_pipe_block_multi() {
    assert_exec_matches_fixture("pipe_block_multi");
}

#[test]
fn parse_pipe_kw_mixed() {
    let code = r#"
        func main() {
            pipe run("echo", "a") | { run("cat") }
        }
    "#;
    use sh2c::span::SourceMap;
    use sh2c::lexer::lex;
    use sh2c::parser::parse;
    
    let sm = SourceMap::new(code.to_string());
    let tokens = lex(&sm, code).expect("Lexing failed");
    let program = parse(&tokens, &sm, "test_mixed.sh2").expect("Parsing failed");

    if let Stmt { node: StmtKind::Pipe(segments), .. } = &program.functions[0].body[0] {
        assert_eq!(segments.len(), 2);
        assert!(matches!(segments[0].node, PipeSegment::Run(_)));
        assert!(matches!(segments[1].node, PipeSegment::Block(_)));
    } else {
        panic!("Expected Pipe");
    }
}

#[test]
fn parse_pipe_kw_single_segment_fail() {
    let code = r#"
        func main() {
            pipe { print("fail") }
        }
    "#;
    use sh2c::span::SourceMap;
    use sh2c::lexer::lex;
    use sh2c::parser::parse;

    let sm = SourceMap::new(code.to_string());
    let tokens = lex(&sm, code).expect("Lexing failed");
    let result = parse(&tokens, &sm, "test");
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.msg.contains("pipe requires at least two segments"));
}
