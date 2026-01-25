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
}

use crate::span::Diagnostic;

pub type ParsResult<T> = Result<T, Diagnostic>;

impl<'a> Parser<'a> {
    pub fn error<T>(&self, msg: &str, span: Span) -> ParsResult<T> {
        Err(Diagnostic {
            msg: msg.to_string(),
            span,
            sm: Some(self.sm.clone()),
            file: Some(self.file.to_string()),
        })
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

    pub fn expect(&mut self, kind: TokenKind) -> ParsResult<()> {
        if let Some(t) = self.peek() {
            if t.kind == kind {
                self.advance();
                Ok(())
            } else {
                self.error(&format!("Expected {:?}, got {:?}", kind, t.kind), t.span)
            }
        } else {
            self.error(
                &format!("Expected {:?}, got EOF", kind),
                self.current_span(),
            )
        }
    }
// ...

    pub fn match_kind(&mut self, kind: TokenKind) -> bool {
        if let Some(t) = self.peek() {
            if t.kind == kind {
                self.advance();
                return true;
            }
        }
        false
    }

    /// Consumes explicit separators (semicolons). 
    /// Newlines are treated as whitespace by the lexer, so statements separated by newlines 
    /// are parsed sequentially without explicit separator tokens.
    pub fn consume_separators(&mut self) {
        while self.match_kind(TokenKind::Semi) {
            // Consume
        }
    }
}
