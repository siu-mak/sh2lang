mod common;
mod expr;
mod stmt;

use self::common::{ParsResult, Parser};
use crate::ast::*;
use crate::lexer::{Token, TokenKind};
use crate::span::SourceMap;
use std::collections::HashMap;

pub fn parse(tokens: &[Token], sm: &SourceMap, file: &str) -> ParsResult<Program> {
    let mut parser = Parser::new(tokens, sm, file);
    let mut imports = Vec::new();
    let mut functions = Vec::new();


    let start_span = parser.current_span();

    while parser.peek().is_some() {
        parser.consume_separators();
        if parser.peek().is_none() {
            break;
        }

        if parser.match_kind(TokenKind::Import) {
            if let Some(TokenKind::String(path)) = parser.peek_kind() {
                imports.push(path.clone());
                parser.advance();
            } else {
                parser.error(
                    "Expected string literal after import",
                    parser.current_span(),
                )?;
            }
        } else if parser.match_kind(TokenKind::Func) {
            let start = parser.previous_span(); // 'func' span
            let name = if let Some(TokenKind::Ident(s)) = parser.peek_kind() {
                s.clone()
            } else {
                parser.error("Expected function name", parser.current_span())?
            };
            parser.advance();

            parser.expect(TokenKind::LParen)?;
            let mut params = Vec::new();
            if !parser.match_kind(TokenKind::RParen) {
                loop {
                    if let Some(TokenKind::Ident(p)) = parser.peek_kind() {
                        params.push(p.clone());
                        parser.advance();
                    } else {
                        parser.error("Expected parameter name", parser.current_span())?;
                    }
                    if !parser.match_kind(TokenKind::Comma) {
                        break;
                    }
                }
                parser.expect(TokenKind::RParen)?;
            }

            let body = parser.parse_brace_stmt_block()?;
            // RBrace consumed
            let end = parser.previous_span();
            let span = start.merge(end);

            functions.push(Function {
                name,
                params,
                body,
                span,
                file: file.to_string(),
            });
        } else {
            return parser.error(
                "Top-level statements are not allowed. Move code into func main() { ... }.",
                parser.current_span(),
            );
        }
    }

    let end_span = parser.previous_span(); // Last token span
    let span = if start_span.end <= end_span.end {
        start_span.merge(end_span)
    } else {
        start_span // Empty file?
    };

    Ok(Program {
        imports,
        functions,
        span,
        source_maps: HashMap::new(),  // Filled by loader later
        entry_file: file.to_string(), // Initial parse sets this, loader might override or correct it
    })
}
