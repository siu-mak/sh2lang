use crate::lexer::{Token, TokenKind};
use crate::span::{SourceMap, Span};

pub(crate) struct Parser<'a> {
    pub tokens: &'a [Token],
    pub pos: usize,
    pub sm: &'a SourceMap,
    pub file: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token], sm: &'a SourceMap, file: &'a str) -> Self {
        Parser {
            tokens,
            pos: 0,
            sm,
            file,
        }
    }

    pub fn peek(&self) -> Option<&'a Token> {
        self.tokens.get(self.pos)
    }

    pub fn peek_kind(&self) -> Option<&'a TokenKind> {
        self.peek().map(|t| &t.kind)
    }

    pub fn advance(&mut self) -> Option<&'a Token> {
        let t = self.tokens.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    pub fn error(&self, msg: &str, span: Span) -> ! {
        panic!("{}", self.sm.format_diagnostic(self.file, msg, span));
    }

    pub fn previous_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            // Should not happen if called correctly, but fallback to 0..0
            Span::new(0, 0)
        }
    }

    pub fn current_span(&self) -> Span {
        if let Some(t) = self.peek() {
            t.span
        } else {
            // EOF span: one char past last token or 0
            if self.tokens.is_empty() {
                Span::new(0, 0)
            } else {
                let last = self.tokens.last().unwrap().span;
                Span::new(last.end, last.end + 1)
            }
        }
    }

    pub fn expect(&mut self, kind: TokenKind) {
        if let Some(t) = self.peek() {
            if t.kind == kind {
                self.advance();
            } else {
                self.error(&format!("Expected {:?}, got {:?}", kind, t.kind), t.span);
            }
        } else {
            self.error(
                &format!("Expected {:?}, got EOF", kind),
                self.current_span(),
            );
        }
    }

    pub fn match_kind(&mut self, kind: TokenKind) -> bool {
        if let Some(t) = self.peek() {
            if t.kind == kind {
                self.advance();
                return true;
            }
        }
        false
    }
}
