use crate::common::*;
use sh2c::formatter;
use sh2c::lexer;
use sh2c::parser;
use sh2c::span::SourceMap;

mod common;

#[test]
fn test_formatter_suites() {
    let cases = [
        "fmt_basic", 
        "fmt_strings_and_interp", 
        "fmt_redirect_heredoc",
        "fmt_chain_indent",
        "fmt_capture_pipe",
        "fmt_misc_exprs"
    ];
    for case in cases {
        eprintln!("Testing formatter case: {}", case);
        verify_formatting(case);
    }
}

fn verify_formatting(fixture_name: &str) {
    let sh2_path = format!("tests/fixtures/{}.sh2", fixture_name);
    let expected_path = format!("tests/fixtures/{}.formatted.sh2.expected", fixture_name);
    
    let src = std::fs::read_to_string(&sh2_path).expect("Failed to read fixture");

    // 1. Parse Original
    let sm = SourceMap::new(src.clone());
    let tokens = lexer::lex(&sm, &src).expect("Lexer failed");
    let mut original_ast = parser::parse(&tokens, &sm, "test").expect("Parser failed");
    
    // 2. Format
    let formatted = formatter::format_program(&original_ast);
    assert!(!formatted.contains("<<UNIMPLEMENTED"), "Formatter emitted UNIMPLEMENTED placeholder");

    // 3. Check Snapshot
    if std::env::var("SH2C_UPDATE_SNAPSHOTS").is_ok() {
        std::fs::write(&expected_path, &formatted).expect("Failed to write snapshot");
    }
    let expected = std::fs::read_to_string(&expected_path).unwrap_or_default();
    assert_eq!(formatted.trim(), expected.trim(), "Formatted output mismatch in {}", fixture_name);

    // 4. Parse Formatted
    let sm_fmt = SourceMap::new(formatted.clone());
    let tokens_fmt = lexer::lex(&sm_fmt, &formatted).expect("Lexer failed on formatted output");
    let mut formatted_ast = parser::parse(&tokens_fmt, &sm_fmt, &format!("{}.formatted", fixture_name)).expect("Parser failed on formatted output");

    // 5. Compare ASTs (ignoring spans)
    strip_spans_program(&mut original_ast);
    strip_spans_program(&mut formatted_ast);
    
    assert_eq!(original_ast, formatted_ast, "AST mismatch after round-trip");
}
