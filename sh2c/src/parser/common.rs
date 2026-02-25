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

    /// Parse a qualified call after the caller has consumed `ns_ident` and the `.` token.
    /// Always succeeds with (func_name, func_span, args, full_span) or returns Err.
    /// At entry, self.pos points to the token AFTER the dot.
    pub fn parse_qualified_call_or_err(
        &mut self, ns: &str, ns_span: Span,
    ) -> ParsResult<(String, Span, Vec<crate::ast::Expr>, Span)> {
        // Current token is what follows the dot.
        // Check for double-dot: fs..x  (current token would be Dot or DotDot)
        if matches!(self.peek_kind(), Some(TokenKind::Dot) | Some(TokenKind::DotDot)) {
            return self.error("Expected identifier after '.'", self.current_span());
        }

        // Must be an identifier
        let func_name = match self.peek_kind() {
            Some(TokenKind::Ident(f)) => { let f = f.clone(); self.advance(); f }
            _ => return self.error(
                &format!("Expected identifier after '{}.', e.g. {}.func()", ns, ns),
                self.current_span(),
            ),
        };
        // Capture func_span right after consuming the ident — used for targeted error spans
        let func_span = self.previous_span();

        // Must be followed by '('
        if self.peek_kind() != Some(&TokenKind::LParen) {
            return self.error(
                &format!(
                    "Expected '(' after '{}.{}' — qualified names can only be used as function calls",
                    ns, func_name
                ),
                func_span, // point at the func ident, not the next token
            );
        }
        self.expect(TokenKind::LParen)?;

        // Parse positional args (reject named args)
        let mut args = Vec::new();
        if !self.match_kind(TokenKind::RParen) {
            loop {
                if let Some(TokenKind::Ident(_)) = self.peek_kind() {
                    if self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::Equals) {
                        return self.error(
                            "Named arguments are not supported for qualified calls",
                            self.current_span(),
                        );
                    }
                }
                args.push(self.parse_expr()?);
                if !self.match_kind(TokenKind::Comma) { break; }
            }
            self.expect(TokenKind::RParen)?;
        }

        let full_span = ns_span.merge(self.previous_span());

        // Reject chained dots: a.b().c or a.b()..x
        if matches!(self.peek_kind(), Some(TokenKind::Dot) | Some(TokenKind::DotDot)) {
            return self.error(
                "chained qualified calls are not supported (a.b.c())",
                self.current_span(),
            );
        }

        Ok((func_name, func_span, args, full_span))
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

    /// Consumes explicit separators (semicolons). 
    /// Newlines are treated as whitespace by the lexer, so statements separated by newlines 
    /// are parsed sequentially without explicit separator tokens.
    pub fn consume_separators(&mut self) {
        while self.match_kind(TokenKind::Semi) {
            // Consume
        }
    }
}
