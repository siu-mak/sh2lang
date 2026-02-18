use super::common::{ParsResult, Parser};
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::span::{Diagnostic, Span};
use crate::sudo::SudoSpec;

impl<'a> Parser<'a> {
    pub fn parse_expr(&mut self) -> ParsResult<Expr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> ParsResult<Expr> {
        let mut left = self.parse_and()?;

        while self.match_kind(TokenKind::OrOr) {
            let right = self.parse_and()?;
            let span = left.span.merge(right.span);
            left = Expr {
                node: ExprKind::Or(Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> ParsResult<Expr> {
        let mut left = self.parse_comparison()?;

        while self.match_kind(TokenKind::AndAnd) {
            let right = self.parse_comparison()?;
            let span = left.span.merge(right.span);
            left = Expr {
                node: ExprKind::And(Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> ParsResult<Expr> {
        let left = self.parse_concat()?;

        if let Some(kind) = self.peek_kind() {
            let op = match kind {
                TokenKind::EqEq => Some(CompareOp::Eq),
                TokenKind::NotEq => Some(CompareOp::NotEq),
                TokenKind::Lt => Some(CompareOp::Lt),
                TokenKind::Le => Some(CompareOp::Le),
                TokenKind::Gt => Some(CompareOp::Gt),
                TokenKind::Ge => Some(CompareOp::Ge),
                _ => None,
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_concat()?;
                let span = left.span.merge(right.span);
                return Ok(Expr {
                    node: ExprKind::Compare {
                        left: Box::new(left),
                        op,
                        right: Box::new(right),
                    },
                    span,
                });
            }
        }
        Ok(left)
    }

    fn parse_concat(&mut self) -> ParsResult<Expr> {
        let mut left = self.parse_sum()?;
        while self.peek_kind() == Some(&TokenKind::Amp) {
            let amp_token = self.advance().unwrap();
            let amp_span = amp_token.span;
            
            // Ticket 10: Enforce whitespace around &
            // If the previous expression ends exactly where & starts, there is no whitespace.
            if left.span.end == amp_span.start { 
                return self.error("The & operator requires whitespace: env.HOME & \"/x\"", amp_span);
            }

            let right = self.parse_sum()?;
            
            // Check whitespace after &
            if amp_span.end == right.span.start {
                 return self.error("The & operator requires whitespace: env.HOME & \"/x\"", amp_span);
            }

            let span = left.span.merge(right.span);
            left = Expr {
                node: ExprKind::Concat(Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn parse_sum(&mut self) -> ParsResult<Expr> {
        let mut left = self.parse_term()?;

        loop {
            let op = match self.peek_kind() {
                Some(TokenKind::Plus) => ArithOp::Add,
                Some(TokenKind::Minus) => ArithOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_term()?;
            let span = left.span.merge(right.span);
            left = Expr {
                node: ExprKind::Arith {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            };
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> ParsResult<Expr> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.peek_kind() {
                Some(TokenKind::Star) => ArithOp::Mul,
                Some(TokenKind::Slash) => ArithOp::Div,
                Some(TokenKind::Percent) => ArithOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            let span = left.span.merge(right.span);
            left = Expr {
                node: ExprKind::Arith {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> ParsResult<Expr> {
        let start = self.current_span();
        if self.match_kind(TokenKind::Bang) {
            let expr = self.parse_unary()?;
            let span = start.merge(expr.span);
            Ok(Expr {
                node: ExprKind::Not(Box::new(expr)),
                span,
            })
        } else if self.match_kind(TokenKind::Minus) {
            let right = self.parse_unary()?;
            let span = start.merge(right.span);
            Ok(Expr {
                node: ExprKind::Arith {
                    left: Box::new(Expr {
                        node: ExprKind::Number(0),
                        span: start,
                    }),
                    op: ArithOp::Sub,
                    right: Box::new(right),
                },
                span,
            })
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> ParsResult<Expr> {
        let mut expr = self.parse_atom()?;

        loop {
            let start = expr.span;
            if self.match_kind(TokenKind::Dot) {
                if let Some(TokenKind::Ident(name)) = self.peek_kind() {
                    let name = name.clone();
                    let end_span = self.advance().unwrap().span;
                    let span = start.merge(end_span);
                    expr = Expr {
                        node: ExprKind::Field {
                            base: Box::new(expr),
                            name,
                        },
                        span,
                    };
                } else if self.match_kind(TokenKind::Status) {
                    let end_span = self.previous_span();
                    let span = start.merge(end_span);
                    expr = Expr {
                        node: ExprKind::Field {
                            base: Box::new(expr),
                            name: "status".to_string(),
                        },
                        span,
                    };
                } else if self.match_kind(TokenKind::Stdout) {
                    let end_span = self.previous_span();
                    let span = start.merge(end_span);
                    expr = Expr {
                        node: ExprKind::Field {
                            base: Box::new(expr),
                            name: "stdout".to_string(),
                        },
                        span,
                    };
                } else if self.match_kind(TokenKind::Stderr) {
                    let end_span = self.previous_span();
                    let span = start.merge(end_span);
                    expr = Expr {
                        node: ExprKind::Field {
                            base: Box::new(expr),
                            name: "stderr".to_string(),
                        },
                        span,
                    };
                } else {
                    self.error("Expected identifier after dot", self.current_span())?;
                    unreachable!() // or let it propagate, but ? already returns
                }
            } else if self.match_kind(TokenKind::LBracket) {
                // Map indexing check: var["key"]
                let mut is_map = false;
                if let ExprKind::Var(ref name) = expr.node {
                    if let Some(TokenKind::String(key)) = self.peek_kind() {
                        // Need to check if next is RBracket without consuming
                        if self.pos + 1 < self.tokens.len()
                            && self.tokens[self.pos + 1].kind == TokenKind::RBracket
                        {
                            self.advance(); // String
                            let key = key.clone();
                            self.expect(TokenKind::RBracket)?;
                            let end = self.previous_span();
                            let span = start.merge(end);
                            expr = Expr {
                                node: ExprKind::MapIndex {
                                    map: name.clone(),
                                    key,
                                },
                                span,
                            };
                            is_map = true;
                        }
                    }
                }

                if !is_map {
                    let index = self.parse_expr()?;
                    self.expect(TokenKind::RBracket)?;
                    let end = self.previous_span();
                    let span = start.merge(end);
                    expr = Expr {
                        node: ExprKind::Index {
                            list: Box::new(expr),
                            index: Box::new(index),
                        },
                        span,
                    };
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_atom(&mut self) -> ParsResult<Expr> {
        let t = self.advance();
        if t.is_none() {
            self.error("Unexpected EOF", self.current_span())?;
        }
        let t = t.unwrap();
        let span = t.span;

        match &t.kind {
            TokenKind::Input => {
                self.expect(TokenKind::LParen)?;
                let prompt = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                let full_span = span.merge(self.previous_span());
                Ok(Expr {
                    node: ExprKind::Input(Box::new(prompt)),
                    span: full_span,
                })
            }
            TokenKind::Confirm => {
                self.expect(TokenKind::LParen)?;
                let prompt = self.parse_expr()?;
                
                // Parse optional default= option
                let mut default_expr = None;
                let mut seen_default = false;
                
                while self.match_kind(TokenKind::Comma) {
                    let opt_start = self.current_span();
                    if let Some(TokenKind::Ident(opt_name)) = self.peek_kind() {
                        let opt_name = opt_name.clone();
                        self.advance();
                        self.expect(TokenKind::Equals)?;
                        let value = self.parse_expr()?;
                        
                        match opt_name.as_str() {
                            "default" => {
                                if seen_default {
                                    return self.error("default specified more than once", opt_start);
                                }
                                seen_default = true;
                                default_expr = Some(Box::new(value));
                            }
                            _ => {
                                return self.error(
                                    &format!("unknown confirm() option '{}'; supported: default", opt_name),
                                    opt_start,
                                );
                            }
                        }
                    } else {
                        return self.error("expected option name", self.current_span());
                    }
                }
                
                self.expect(TokenKind::RParen)?;
                let full_span = span.merge(self.previous_span());
                Ok(Expr {
                    node: ExprKind::Confirm { prompt: Box::new(prompt), default: default_expr },
                    span: full_span,
                })
            }

            TokenKind::LBrace => {
                let mut entries = Vec::new();
                while !self.match_kind(TokenKind::RBrace) {
                    let key = if let Some(TokenKind::String(s)) = self.peek_kind() {
                        s.clone()
                    } else {
                        self.error(
                            "Expected string literal key in map literal",
                            self.current_span(),
                        )?
                    };
                    self.advance();
                    self.expect(TokenKind::Colon)?;
                    let val = self.parse_expr()?;
                    entries.push((key, val));
                    if !self.match_kind(TokenKind::Comma) {
                        if self.peek_kind() != Some(&TokenKind::RBrace) {
                            self.error("Expected comma or closing brace", self.current_span())?;
                        }
                    }
                }
                // RBrace consumed by while loop condition? No, matched_kind consumed it if true?
                // Wait, match_kind consumes if true.
                // Loop check: !match_kind(RBrace) -> if RBrace, consumes and breaks.
                let full_span = span.merge(self.previous_span());
                Ok(Expr {
                    node: ExprKind::MapLiteral(entries),
                    span: full_span,
                })
            }
            TokenKind::LParen => {
                let e = self.parse_expr()?;
                if self.peek_kind() == Some(&TokenKind::Semi) {
                    return self.error("Unexpected statement separator ';' inside expression. Use ';' only between statements.", self.current_span());
                }
                self.expect(TokenKind::RParen)?;
                // Parenthesized expression span shouldn't be extended? Or should it?
                // Using 'e.span' discards parens. Usually we want inner span or outer?
                // Let's keep inner span for now or extend?
                // Most parsers keep parens in span.
                let full_span = span.merge(self.previous_span());
                Ok(Expr {
                    node: e.node,
                    span: full_span,
                })
            }
            TokenKind::String(s) => self.parse_interpolated_string(s, span),
            TokenKind::Sh => {
                self.expect(TokenKind::LParen)?;
                let cmd = self.parse_expr()?;
                let options = self.parse_sh_options(false)?;
                
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::Sh { cmd: Box::new(cmd), options },
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::Run => {
                let call = self.parse_call_args_and_options()?;
                // options are not allowed for run() when used directly as an expression (non-capture)
                if let Some(opt) = call.options.first() {
                     return self.error("run() options are only allowed inside capture(...); use capture(run(...), allow_fail=true)", opt.span);
                }
                Ok(Expr {
                    node: ExprKind::Run(call),
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::Ident(s) => {
                let s = s.clone();
                let is_call = self.peek_kind() == Some(&TokenKind::LParen);
                if is_call {
                    if s == "sudo" {
                        self.expect(TokenKind::LParen)?;

                        let mut args = Vec::new();
                        let mut options = Vec::new();

                        if !self.match_kind(TokenKind::RParen) {
                            loop {
                                // Check for named arg: IDENT = ...
                                // Allow mixed positional/named options.
                                let mut is_named = false;
                                if let Some(TokenKind::Ident(_)) = self.peek_kind() {
                                    if self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::Equals) {
                                       is_named = true;
                                    }
                                }

                                if is_named {
                                    let name = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                                        s.clone()
                                    } else {
                                        unreachable!()
                                    };
                                    let name_span = self.advance().unwrap().span;
                                    self.expect(TokenKind::Equals)?;
                                    let value = self.parse_expr()?;

                                    options.push(CallOption {
                                        name,
                                        value,
                                        span: name_span,
                                    });
                                } else {
                                    args.push(self.parse_expr()?);
                                }

                                if !self.match_kind(TokenKind::Comma) {
                                    break;
                                }
                            }
                            self.expect(TokenKind::RParen)?;
                        }

                        // Validate options
                        // Map lightweight error to full Diagnostic for parser
                        let spec = match SudoSpec::from_options(&options) {
                             Ok(s) => s,
                             Err((msg, span)) => return self.error(&msg, span),
                        };

                        if spec.allow_fail.is_some() {
                             // Correctly identifying the span for the error
                             let opt_span = options.iter().find(|o| o.name == "allow_fail").unwrap().span;
                             return self.error(
                                 "allow_fail is only valid on statement-form sudo(...); use capture(sudo(...), allow_fail=true) to allow failure during capture",
                                 opt_span
                             );
                        }

                        if args.is_empty() {
                            return self.error(
                                "sudo() requires at least one positional argument (the command)",
                                span,
                            );
                        }

                        let full_span = span.merge(self.previous_span());
                        return Ok(Expr {
                            node: ExprKind::Sudo { args, options },
                            span: full_span,
                        });
                    }


                    
                    let mut args = Vec::new();
                    let mut options = Vec::new();
                    
                    if self.match_kind(TokenKind::LParen) {
                        if !self.match_kind(TokenKind::RParen) {
                            loop {
                                // Check for named argument: IDENT = ...
                                // We allow mixing for now, but builtins usually enforce one or the other.
                                let mut is_named = false;
                                if let Some(TokenKind::Ident(_)) = self.peek_kind() {
                                    if self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::Equals) {
                                       is_named = true;
                                    }
                                }

                                if is_named {
                                    // Named argument - only allowed for specific builtins
                                    let allowed_builtins = ["run", "sudo", "sh", "capture", "confirm", "find_files", "find0", "wait", "wait_all"];
                                    if !allowed_builtins.contains(&s.as_str()) {
                                        return self.error(
                                            "Named arguments are only supported for builtins: run, sudo, sh, capture, confirm, find_files, find0, wait, wait_all",
                                            self.current_span()
                                        );
                                    }
                                    
                                    let name = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                                        s.clone()
                                    } else {
                                        unreachable!()
                                    };
                                    let name_span = self.advance().unwrap().span;
                                    self.expect(TokenKind::Equals)?;
                                    let value = self.parse_expr()?;

                                    options.push(CallOption {
                                        name,
                                        value,
                                        span: name_span,
                                    });
                                } else {
                                    // Positional argument
                                    args.push(self.parse_expr()?);
                                }

                                if !self.match_kind(TokenKind::Comma) {
                                    break;
                                }
                            }
                            self.expect(TokenKind::RParen)?;
                        }
                    }

                    let full_span = span.merge(self.previous_span());

                    // Arity validation
                    match s.as_str() {
                        "before" | "after" | "coalesce" | "default" | "split" => {
                            if args.len() != 2 {
                                // We can error here with the call span
                                self.error(
                                    &format!("{} requires exactly 2 arguments", s),
                                    full_span,
                                )?;
                            }
                        }
                        "replace" => {
                            if args.len() != 3 {
                                self.error(
                                    &format!("{} requires exactly 3 arguments", s),
                                    full_span,
                                )?;
                            }
                        }
                        "trim" => {
                            if args.len() != 1 {
                                self.error(
                                    &format!("{} requires exactly 1 argument", s),
                                    full_span,
                                )?;
                            }
                        }
                        _ => {}
                    }

                    Ok(Expr {
                        node: ExprKind::Call { name: s, args, options },
                        span: full_span,
                    })
                } else {
                    Ok(Expr {
                        node: ExprKind::Var(s),
                        span,
                    })
                }
            }
            TokenKind::Dollar => {
                if let Some(TokenKind::String(s)) = self.peek_kind() {
                    let s = s.clone();
                    let str_span = self.advance().unwrap().span;
                    let full_span = span.merge(str_span);
                    
                    // Attempt to parse brace interpolated string, and translate known failure modes
                    match self.parse_brace_interpolated_string(&s, full_span) {
                        Ok(expr) => Ok(expr),
                        Err(diagnostic) => {
                            // Detect the known limitation: quotes inside holes cause lexer truncation.
                            // The lexer terminates the string token early at the first `"` inside a hole.
                            //
                            // Use robust source-based detection to distinguish truncation from real errors.
                            let is_lexer_truncation = diagnostic.msg.contains("Unterminated interpolation hole")
                                && self.interpolated_string_looks_truncated(span, str_span);
                            
                            if is_lexer_truncation {
                                // Emit improved diagnostic for lexer limitation
                                let primary = "String literals inside interpolation holes are not supported yet (lexer limitation).";
                                let help = "help: workaround: assign to a variable first (e.g., let v = \"value\"; print($\"X: {v}\"))";
                                let combined_msg = format!("{}\n{}", primary, help);
                                
                                Err(self.make_error(&combined_msg, diagnostic.span))
                            } else {
                                // Real missing `}` or other error - keep original diagnostic
                                Err(diagnostic)
                            }
                        }
                    }
                } else {
                    self.parse_command_substitution(span, false)
                }
            }
            TokenKind::LBracket => {
                let mut exprs = Vec::new();
                if !self.match_kind(TokenKind::RBracket) {
                    loop {
                        exprs.push(self.parse_expr()?);
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                    if self.peek_kind() == Some(&TokenKind::Semi) {
                         return self.error("Unexpected statement separator ';' inside expression. Use ';' only between statements.", self.current_span());
                    }
                    self.expect(TokenKind::RBracket)?;
                }
                let full_span = span.merge(self.previous_span());
                Ok(Expr {
                    node: ExprKind::List(exprs),
                    span: full_span,
                })
            }
            TokenKind::Args => {
                let mut full_span = span;
                if self.match_kind(TokenKind::LParen) {
                    self.expect(TokenKind::RParen)?;
                    full_span = full_span.merge(self.previous_span());
                }
                Ok(Expr {
                    node: ExprKind::Args,
                    span: full_span,
                })
            },
            TokenKind::Capture => self.parse_command_substitution(span, true),
            TokenKind::Exists => {
                self.expect(TokenKind::LParen)?;
                let path = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::Exists(Box::new(path)),
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::IsDir => {
                self.expect(TokenKind::LParen)?;
                let path = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::IsDir(Box::new(path)),
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::IsFile => {
                self.expect(TokenKind::LParen)?;
                let path = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::IsFile(Box::new(path)),
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::IsSymlink => {
                self.expect(TokenKind::LParen)?;
                let path = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::IsSymlink(Box::new(path)),
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::IsExec => {
                self.expect(TokenKind::LParen)?;
                let path = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::IsExec(Box::new(path)),
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::IsReadable => {
                self.expect(TokenKind::LParen)?;
                let path = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::IsReadable(Box::new(path)),
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::IsWritable => {
                self.expect(TokenKind::LParen)?;
                let path = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::IsWritable(Box::new(path)),
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::IsNonEmpty => {
                self.expect(TokenKind::LParen)?;
                let path = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::IsNonEmpty(Box::new(path)),
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::BoolStr => {
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::BoolStr(Box::new(expr)),
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::Len => {
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::Len(Box::new(expr)),
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::Arg => {
                self.expect(TokenKind::LParen)?;
                let index_expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::Arg(Box::new(index_expr)),
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::Index => {
                self.expect(TokenKind::LParen)?;
                let list = self.parse_expr()?;
                self.expect(TokenKind::Comma)?;
                let index = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::Index {
                        list: Box::new(list),
                        index: Box::new(index),
                    },
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::Join => {
                self.expect(TokenKind::LParen)?;
                let list = self.parse_expr()?;
                self.expect(TokenKind::Comma)?;
                let sep = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::Join {
                        list: Box::new(list),
                        sep: Box::new(sep),
                    },
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::Status => {
                self.expect(TokenKind::LParen)?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::Status,
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::Pid => {
                self.expect(TokenKind::LParen)?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::Pid,
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::Env => {
                // Check if dot
                if self.peek_kind() == Some(&TokenKind::Dot) {
                    self.advance(); // Dot
                    if let Some(TokenKind::Ident(name)) = self.peek_kind() {
                        let name = name.clone();
                        self.advance();
                        Ok(Expr {
                            node: ExprKind::EnvDot(name),
                            span: span.merge(self.previous_span()),
                        })
                    } else {
                        self.error("Expected identifier after env.", self.current_span())?
                    }
                } else {
                    self.expect(TokenKind::LParen)?;
                    let name_expr = self.parse_expr()?;
                    self.expect(TokenKind::RParen)?;
                    Ok(Expr {
                        node: ExprKind::Env(Box::new(name_expr)),
                        span: span.merge(self.previous_span()),
                    })
                }
            }
            TokenKind::Uid => {
                self.expect(TokenKind::LParen)?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::Uid,
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::Ppid => {
                self.expect(TokenKind::LParen)?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::Ppid,
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::Pwd => {
                self.expect(TokenKind::LParen)?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::Pwd,
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::SelfPid => {
                self.expect(TokenKind::LParen)?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::SelfPid,
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::Argv0 => {
                self.expect(TokenKind::LParen)?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::Argv0,
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::Argc => {
                self.expect(TokenKind::LParen)?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::Argc,
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::Count => {
                self.expect(TokenKind::LParen)?;
                let inner = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::Count(Box::new(inner)),
                    span: span.merge(self.previous_span()),
                })
            }

            TokenKind::True => Ok(Expr {
                node: ExprKind::Bool(true),
                span,
            }),
            TokenKind::False => Ok(Expr {
                node: ExprKind::Bool(false),
                span,
            }),
            TokenKind::Number(n) => Ok(Expr {
                node: ExprKind::Number(*n),
                span,
            }),
            TokenKind::Semi => {
                 return self.error("Unexpected statement separator ';' inside expression. Use ';' only between statements.", span);
             }
            // Spawn and wait are EXPR_BUILTINS, parse as calls
            TokenKind::Spawn | TokenKind::Wait => {
                // These are builtins that look like function calls: spawn(...) or wait(...)
                let name = match t.kind {
                    TokenKind::Spawn => "spawn".to_string(),
                    TokenKind::Wait => "wait".to_string(),
                    _ => unreachable!(),
                };
                
                let mut args = Vec::new();
                let mut options = Vec::new();
                
                self.expect(TokenKind::LParen)?;
                
                if !self.match_kind(TokenKind::RParen) {
                    loop {
                        // Check for named argument: IDENT = ...
                        let mut is_named = false;
                        if let Some(TokenKind::Ident(_)) = self.peek_kind() {
                            if self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::Equals) {
                                is_named = true;
                            }
                        }
                        
                        if is_named {
                            // Named argument
                            let opt_name = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                                s.clone()
                            } else {
                                unreachable!()
                            };
                            let name_span = self.advance().unwrap().span;
                            self.expect(TokenKind::Equals)?;
                            let value = self.parse_expr()?;
                            
                            options.push(CallOption {
                                name: opt_name,
                                value,
                                span: name_span,
                            });
                        } else {
                            // Positional argument
                            args.push(self.parse_expr()?);
                        }
                        
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                }
                
                let full_span = span.merge(self.previous_span());
                Ok(Expr {
                    node: ExprKind::Call { name, args, options },
                    span: full_span,
                })
            }
            _ => {
                self.error(&format!("Expected expression, got {:?}", t.kind), span)
            }
        }
    }

    fn parse_command_substitution(&mut self, start_span: Span, is_capture: bool) -> ParsResult<Expr> {
        self.expect(TokenKind::LParen)?;
        let mut segments = Vec::new();
        let mut options = Vec::new();
        let mut allow_fail_span: Option<Span> = None;

        loop {
            let mut args = Vec::new();

            loop {
                // Check end of args list or pipe
                let next_kind = self.peek_kind();
                if next_kind == Some(&TokenKind::RParen) || next_kind == Some(&TokenKind::Pipe) {
                    break;
                }

                if self.match_kind(TokenKind::Run) {
                    // Path C: Special case $( run(...) ) - Flatten args & extract options
                    let call = self.parse_call_args_and_options()?;
                    
                    if is_capture {
                        // Strict validation and hoisting for capture(run(...))
                        for opt in call.options.iter() {
                            if opt.name == "allow_fail" {
                                if let Some(_) = allow_fail_span {
                                     return self.error("allow_fail specified more than once", opt.span);
                                }
                                allow_fail_span = Some(opt.span);
                                if !matches!(opt.value.node, ExprKind::Bool(_)) {
                                     return self.error("allow_fail must be a boolean literal", opt.value.span);
                                }
                            } else {
                                return self.error(&format!("Unknown option '{}' for run()", opt.name), opt.span);
                            }
                        }

                        // Hoist valid options (currently only allow_fail)
                        options.extend(call.options);
                    } else {
                         // Reject ANY option in non-capture context
                         if let Some(opt) = call.options.first() {
                             return self.error("run() options are not allowed in command substitution $(...); use capture(run(...), allow_fail=true)", opt.span);
                         }
                    }

                    // Flatten args into current segment (command-word model)
                    args.extend(call.args);

                } else if self.match_kind(TokenKind::Sh) {
                     let s_span = self.previous_span();
                     
                     // Check for sh("cmd") shorthand vs bare `sh` command word
                     if self.peek_kind() == Some(&TokenKind::LParen) {
                          // sh("cmd") shorthand: emit ExprKind::Sh so lowering
                          // correctly injects -c (Ticket 10).
                          self.expect(TokenKind::LParen)?;
                          let cmd = self.parse_expr()?;
                          let options = self.parse_sh_options(false)?;
                          self.expect(TokenKind::RParen)?;
                          args.push(Expr {
                              node: ExprKind::Sh { cmd: Box::new(cmd), options },
                              span: s_span,
                          });
                     } else {
                          // Bare `sh` as a command word (no parens = file runner)
                          args.push(Expr { node: ExprKind::Literal("sh".to_string()), span: s_span });
                     }
                } else if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                     if s == "sudo" {
                         let s_span = self.advance().unwrap().span;
                         // Preserve existing sudo behavior
                         if self.peek_kind() == Some(&TokenKind::LParen) {
                             let call = self.parse_call_args_and_options()?;
                             
                             // Validate options using SudoSpec logic
                             // This ensures policy compliance and valid option combinations
                             match SudoSpec::from_options(&call.options) {
                                 Ok(spec) => {
                                      if let Some((_, span)) = spec.allow_fail {
                                           return self.error("allow_fail is only valid on statement-form sudo(...); use capture(sudo(...), allow_fail=true) to allow failure during capture", span);
                                      }

                                      if call.args.is_empty() {
                                          return self.error("sudo() requires at least one positional argument (the command)", s_span);
                                      }

                                      // FLATTEN to avoid double-quoting in $(...)
                                      // Replicate lowering logic: sudo + flags + -- + args
                                      
                                      args.push(Expr { node: ExprKind::Literal("sudo".to_string()), span: s_span });
                                      
                                      for flag in spec.to_flags_argv() {
                                          args.push(Expr { 
                                              node: ExprKind::Literal(flag), 
                                              span: s_span 
                                          });
                                      }
                                      
                                      // Mandatory separator before command args (aligns with lower.rs)
                                      args.push(Expr { 
                                          node: ExprKind::Literal("--".to_string()), 
                                          span: s_span 
                                      });
                                      
                                      args.extend(call.args);
                                 },
                                 Err((msg, span)) => return self.error(&msg, span),
                             }

                         } else {
                             // Just "sudo" word
                             args.push(Expr { node: ExprKind::Literal("sudo".to_string()), span: s_span });
                         }

                     } else {
                        // Check for named arg option (allow_fail=...) - Reject in generic calls
                        let is_named = self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::Equals);

                        if is_named {
                            let name = s.clone();
                            let name_span = self.advance().unwrap().span; // consume ident
                            self.expect(TokenKind::Equals)?; // consume =
                            let value = self.parse_expr()?; // consume value
                            
                            if is_capture {
                                // Outer capture option validation
                                if name == "allow_fail" {
                                    if let Some(_) = allow_fail_span {
                                         return self.error("allow_fail specified more than once", name_span);
                                    }
                                    allow_fail_span = Some(name_span);
                                    if !matches!(value.node, ExprKind::Bool(_)) {
                                         return self.error("allow_fail must be a boolean literal", value.span);
                                    }
                                } else {
                                    return self.error(&format!("Unknown option '{}' for capture(). Supported options: allow_fail", name), name_span);
                                }

                                options.push(CallOption { name, value, span: name_span });
                                continue;
                            } else {
                                // Policy Enforcement: Generic calls cannot have named args
                                return self.error("Named arguments are only supported for builtins: run, sudo, sh, capture, confirm", name_span);
                            }
                        }

                        // Regular Ident: Treat as Literal Command Word
                        let name = s.clone();
                        let span = self.advance().unwrap().span;
                        
                        args.push(Expr {
                            node: ExprKind::Literal(name),
                            span,
                        });
                        
                        // Generic call shorthand `$(func(arg))` -> flatten to `func arg`
                        // Strict positional-only parsing to avoid named-arg leak
                        if self.peek_kind() == Some(&TokenKind::LParen) {
                            self.expect(TokenKind::LParen)?;
                            if !self.match_kind(TokenKind::RParen) {
                                loop {
                                     // Check for named arg leakage in shorthand
                                     if let Some(TokenKind::Ident(_)) = self.peek_kind() {
                                         if self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::Equals) {
                                              return self.error("Named arguments are only supported for builtins: run, sudo, sh, capture, confirm", self.current_span());
                                         }
                                     }

                                     args.push(self.parse_expr()?);

                                     if !self.match_kind(TokenKind::Comma) {
                                         break;
                                     }
                                }
                                self.expect(TokenKind::RParen)?;
                            }
                        }
                     }
                } else if let Some(TokenKind::String(_s)) = self.peek_kind() {
                     args.push(self.parse_expr()?);
                } else if self.match_kind(TokenKind::Dollar) {
                     self.pos -= 1; // Backtrack for parse_expr
                     args.push(self.parse_expr()?);
                } else {
                    // Allow other literals (numbers, etc) via parse_expr
                    args.push(self.parse_expr()?);
                }

                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
            }

            if args.is_empty() {
                 return self.error("Unexpected empty command", self.current_span());
            }

            segments.push(args);
            if !self.match_kind(TokenKind::Pipe) {
                break;
            }
        }

        self.expect(TokenKind::RParen)?;
        let full_span = start_span.merge(self.previous_span());

        // AST Compatibility Rule:
        // Always return Command/CommandPipe.
        // If capture options are present, wrap in Capture.
        
        let command_expr = if segments.len() == 1 {
            Expr {
                node: ExprKind::Command(segments.into_iter().next().unwrap()),
                span: full_span,
            }
        } else {
            Expr {
                node: ExprKind::CommandPipe(segments),
                span: full_span,
            }
        };

        if is_capture && !options.is_empty() {
             Ok(Expr {
                node: ExprKind::Capture {
                    expr: Box::new(command_expr),
                    options,
                },
                span: full_span,
            })
        } else {
            Ok(command_expr)
        }
    }
                    
    fn parse_interpolated_string(&mut self, raw: &str, span: Span) -> ParsResult<Expr> {
        // P0 Fix: Treat all strings as literals (no implicit expansion of $ or ${}).
        // We simply return the raw content as a single Literal expr.
        // We retain \$ -> $ unescaping for backward compatibility if desired, 
        // OR we treat \$ as literal \$ if strictness implies raw?
        // Plan says: "Preserve \$ unescaping (or treat as literal $)".
        // If I want print("\$FOO") to be literal, then \$ must be $.
        // BUT if "..." is STRICT literal, then $FOO is literal $FOO.
        // So escaping is NOT needed for $FOO.
        // But escaping might be needed for " (quote).
        // The lexer handles " escaping inside string token.
        // Here we receive raw string content (already stripped of surrounding quotes?).
        // No, `raw` here is content.
        // Lexer provides raw string content?
        // Wait, `parse_interpolated_string` is called by `parse_atom` on `TokenKind::String(raw)`.
        // The `raw` string from lexer usually handles escapes?
        // Let's assume `raw` is the literal content characters.
        // If so, we just wrap it.
        // However, existing code had a loop for `\$` unescaping.
        // "We retain \$ -> $ unescaping for backward compatibility".
        
        // Actually, if strict literal `"$FOO"` means literal `$FOO`.
        // Then `"\$FOO"` means literal `\$FOO`?
        // Or does `\` escape `$`?
        // In bash `'...'`, `\` is literal.
        // In sh2 `strict literal`, maybe `\` escapes? 
        // I will stick to "Treat all strings as literals".
        // But to support `"` inside string, lexer handled `\"`.
        // Does lexer handle `\$`? Probably not.
        // I will implementation simple pass-through OR basic unescape if needed.
        // Previous correct implementation (before I reverted it):
        
        let mut buf = String::new();
        let mut i = 0;
        while i < raw.len() {
             if raw[i..].starts_with("\\$") {
                 buf.push('$');
                 i += 2;
                 continue;
             }
             let ch = raw[i..].chars().next().unwrap();
             buf.push(ch);
             i += ch.len_utf8();
        }
        
        Ok(Expr {
            node: ExprKind::Literal(buf),
            span,
        })
    }

    /// Detect if an interpolated string token was truncated by the lexer.
    ///
    /// The lexer terminates string tokens at the first unescaped `"`, even if
    /// that quote appears inside an interpolation hole `{...}`. This helper
    /// scans the raw source to find where the interpolated string *should* end
    /// (the true closing quote) and compares it to where the lexer thinks it ends.
    ///
    /// Returns `true` if the lexer truncated the string early (quote-in-hole case).
    fn interpolated_string_looks_truncated(&self, dollar_span: Span, str_span: Span) -> bool {
        let src = self.sm.src();
        let full_span = dollar_span.merge(str_span);
        
        // Verify this is $"..."
        if full_span.start >= src.len() || src.as_bytes()[full_span.start] != b'$' {
            return false;
        }
        
        // Find the opening quote after $
        let mut pos = full_span.start + 1;
        if pos >= src.len() || src.as_bytes()[pos] != b'"' {
            return false;
        }
        
        // The lexer believes the string ends at str_span.end - 1 (the closing quote)
        let assumed_close_quote_abs = str_span.end.saturating_sub(1);
        
        // Scan forward to find the true closing quote, tracking brace depth
        pos += 1; // Start after opening quote
        let mut brace_depth: i32 = 0;
        let mut escaped = false;
        
        let bytes = src.as_bytes();
        while pos < bytes.len() {
            let ch = bytes[pos];
            
            if escaped {
                escaped = false;
                pos += 1;
                continue;
            }
            
            match ch {
                b'\\' => {
                    escaped = true;
                }
                b'{' => {
                    brace_depth += 1;
                }
                b'}' => {
                    brace_depth = brace_depth.saturating_sub(1);
                }
                b'"' if brace_depth == 0 => {
                    // Found the true closing quote
                    let true_close_quote_abs = pos;
                    // If true close is after the assumed close, lexer truncated
                    return true_close_quote_abs > assumed_close_quote_abs;
                }
                _ => {}
            }
            
            pos += 1;
        }
        
        // EOF reached without finding closing quote - not truncation, just unterminated
        false
    }

    fn parse_brace_interpolated_string(&mut self, _unused_decoded: &str, span: Span) -> ParsResult<Expr> {
        // Expected span: $"..." where span.start is '$' and span.end is after closing '"'
        // Use raw source for precise offsets and to bypass Lexer's escape decoding
        let raw_src = self.sm.src().get(span.start..span.end).ok_or_else(|| {
            self.make_error("Failed to retrieve source for interpolated string", span)
        })?;

        if raw_src.len() < 3 {
             return self.error("Interpolated string span too short (expected $\"...\")", span);
        }
        
        // Find the start of the string (opening quote)
        // full_span includes the leading '$'
        let quote_start_idx = match raw_src.find('"') {
            Some(i) => i,
            None => return self.error("Interpolated string missing opening quote", span),
        };
        
        // inner content starts after the quote and ends before the closing quote
        // The last char of raw_src MUST be the closing quote
        if !raw_src.ends_with('"') {
             return self.error("Interpolated string missing closing quote", span);
        }
        
        let inner_start_offset = span.start + quote_start_idx + 1;
        let inner_src = &raw_src[quote_start_idx+1..raw_src.len()-1];
        

        
        let mut parts: Vec<Expr> = Vec::new();
        let mut buf = String::new();
        let mut chars = inner_src.char_indices().peekable();
        
        // Track the start position of the current literal chunk (relative to inner_src)
        let mut lit_start_rel = 0;
        
        while let Some((i, c)) = chars.next() {
            if c == '\\' {
                if let Some((_, next)) = chars.peek() {
                    let next_char = *next;

                    // Explicit escape rule: \{ and \} become literal { and }
                    if next_char == '{' || next_char == '}' {
                        buf.push(next_char);
                        chars.next(); // Consume the peeked character
                        continue; // Loop will advance to next char
                    }
                    // Standard escapes
                    match next_char {
                        'n' => { buf.push('\n'); chars.next(); continue; }
                        't' => { buf.push('\t'); chars.next(); continue; }
                        'r' => { buf.push('\r'); chars.next(); continue; }
                        '\\' => { buf.push('\\'); chars.next(); continue; }
                        '"' => { buf.push('"'); chars.next(); continue; }
                        '$' => { buf.push('$'); chars.next(); continue; }
                         _ => {
                             // Unknown escape: preserve both backslash and next char
                             buf.push('\\');
                             buf.push(next_char);
                             chars.next();
                             continue;
                         }
                    }
                } else {
                    // Trailing backslash
                    buf.push('\\');
                    continue;
                }
            }
            
            if c == '{' {

                // Determine absolute start pos of the hole's content (after '{')
                // i is offset in inner_src.
                let hole_content_start_abs = inner_start_offset + i + 1;
                
                // Flush buffer
                if !buf.is_empty() {
                    parts.push(Expr {
                        node: ExprKind::Literal(std::mem::take(&mut buf)),
                        span: Span::new(inner_start_offset + lit_start_rel, inner_start_offset + i),
                    });
                }

                // robust scan for matching '}'
                // We must respect nested strings inside the hole to avoid stopping at '}' inside a string
                // We do NOT support nested braces `{ { } }` per instructions (unless part of expr?)
                // User said "Sub-lexer... full consumption".
                // If we slice until `}`, `parse_expr` handles the content. 
                // BUT if `}` is inside a string in the expr, we must skip it.
                // So we need a "balanced scan" respecting quotes.
                
                let mut nesting = 1; // We are inside first {
                let mut content_end_rel = i + 1; // relative to inner_src start
                let mut found_end = false;
                
                // Temp iterator for lookahead scanning
                let mut scanner = inner_src[i+1..].char_indices().peekable();
                
                while let Some((j, sc)) = scanner.next() {
                     // j is relative to the slice starting after '{'
                     
                     // Handle backslash escapes first - in raw source, \" is an escaped quote
                     if sc == '\\' {
                         scanner.next(); // Skip the escaped character
                         continue;
                     }
                     
                     // Now handle quoted strings (only unescaped quotes start strings)
                     if sc == '"' {
                         // Skip string content until closing quote
                         while let Some((_, qc)) = scanner.next() {
                             if qc == '\\' { 
                                 scanner.next(); // Skip escaped char inside string
                                 continue; 
                             }
                             if qc == '"' { break; }
                         }
                         continue;
                     }
                     
                     if sc == '{' {
                         nesting += 1;
                     }
                     if sc == '}' {
                         nesting -= 1;
                         if nesting == 0 {
                             content_end_rel = i + 1 + j;
                             found_end = true;
                             break;
                         }
                     }
                }
                
                if !found_end {
                     return self.error("Unterminated interpolation hole; missing '}'", Span::new(inner_start_offset + i, inner_start_offset + i + 1));
                }
                
                // Extract content
                let content_chars = &inner_src[i+1..content_end_rel]; // Chars inside { }
                
                // Unescape content: \" -> ", \\ -> \
                // We must process this because the extracted content is still "inside" the string literal form
                // so valid expr code like "foo" appears as \"foo\"
                let mut content = String::with_capacity(content_chars.len());
                let mut esc_iter = content_chars.chars();
                while let Some(ch) = esc_iter.next() {
                    if ch == '\\' {
                        if let Some(next) = esc_iter.next() {
                            if next == '"' {
                                content.push('"');
                            } else if next == '\\' {
                                content.push('\\');
                            } else {
                                // Other escapes inside hole?
                                // If user wrote $" { \n } ", raw is \n. 
                                // sh2 expression parser handles \n as char.
                                // If user wrote $" { \x } " (where x is not " or \).
                                // Valid sh2 code? 
                                // \x is just \x.
                                content.push('\\');
                                content.push(next);
                            }
                        } else {
                            // Trailing backslash
                            content.push('\\');
                        }
                    } else {
                        content.push(ch);
                    }
                }
                
                // Parse expression from content
                let sub_sm = crate::span::SourceMap::new(content.clone());
                let tokens = crate::lexer::lex(&sub_sm, "interpolation").map_err(|d| {
                     let mut d = d;
                     d.msg = format!("Lexer error inside interpolation: {}", d.msg);
                     // Remap span: base + local_span
                     let local_start = d.span.start;
                     let local_end = d.span.end;
                     d.span = Span::new(hole_content_start_abs + local_start, hole_content_start_abs + local_end);
                     d
                })?;
                
                let mut sub_parser = Parser::new(&tokens, &sub_sm, "interpolation");
                let expr = sub_parser.parse_expr().map_err(|mut d| {
                    d.msg = format!("Parse error inside interpolation: {}", d.msg);
                     let local_start = d.span.start;
                     let local_end = d.span.end;
                     d.span = Span::new(hole_content_start_abs + local_start, hole_content_start_abs + local_end);
                    d
                })?;
                
                if sub_parser.peek().is_some() {
                     // Report "Extra tokens" at the location of the next token
                     let next_tok = sub_parser.peek().unwrap();
                     let local_start = next_tok.span.start;
                     // local_end = next_tok.span.end? Or just point to start?
                     let err_pos = hole_content_start_abs + local_start;
                     return self.error("Extra tokens after expression in interpolation hole", Span::new(err_pos, err_pos + 1));
                }

                parts.push(expr);
                
                // Update lit_start_rel to point to the character after the closing '}'
                lit_start_rel = content_end_rel + 1;
                
                // Advance main loop
                // "chars" iterator needs to skip validly.
                // We manually advanced `scanner`, but `chars` iterator is independent and behind.
                // We calculate how many chars we consumed.
                // We consumed `content_end_rel - i` chars (including the closing }).
                // But `chars` next() yields byte indices.
                // Safest way: Fast-forward `chars`
                while let Some((idx, _)) = chars.peek() {
                    if *idx <= content_end_rel { 
                        chars.next(); 
                    } else {
                        break;
                    }
                }
                // Need to ensure we consumed the closing '}' too.
                // content_end_rel is index of '}'.
                // Loop above stops when peeked index > content_end_rel
                // So we correctly consumed '}'.
                continue;
            }
            
            buf.push(c);
        }

        if !buf.is_empty() {
             parts.push(Expr {
                 node: ExprKind::Literal(buf),
                 span: Span::new(inner_start_offset + lit_start_rel, inner_start_offset + inner_src.len()),
             });
        }
        
        if parts.is_empty() {
            return Ok(Expr { node: ExprKind::Literal(String::new()), span });
        }

        let mut expr = parts[0].clone();
        for p in parts.into_iter().skip(1) {
            expr = Expr {
                node: ExprKind::Concat(Box::new(expr), Box::new(p)),
                span,
            };
        }
        Ok(expr)
    }

    fn make_error(&self, msg: &str, span: Span) -> Diagnostic {
         Diagnostic {
            msg: msg.to_string(),
            span,
            sm: Some(self.sm.clone()),
            file: Some(self.file.to_string()),
        }
    }
    pub(crate) fn parse_sh_options(&mut self, allow_allow_fail: bool) -> Result<Vec<CallOption>, Diagnostic> {
        let mut options = Vec::new();
        let mut seen_shell = false;
        let mut seen_args = false;
        let mut seen_allow_fail = false;

        while self.match_kind(TokenKind::Comma) {
            let opt_start = self.current_span();
            let (opt_name, name_span) = if let Some(TokenKind::Ident(opt_name)) = self.peek_kind() {
                let name = opt_name.clone();
                let span = self.advance().unwrap().span;
                (name, span)
            } else if self.peek_kind() == Some(&TokenKind::Args) {
                // `args` is lexed as a keyword (TokenKind::Args), not Ident.
                // Accept it as a valid option name in named-option position.
                let span = self.advance().unwrap().span;
                ("args".to_string(), span)
            } else {
                return Err(self.make_error("expected option name", self.current_span()));
            };

            self.expect(TokenKind::Equals)?;
            let value = self.parse_expr()?;

            match opt_name.as_str() {
                    "shell" => {
                        if seen_shell {
                             return Err(self.make_error("shell specified more than once", opt_start));
                        }
                        seen_shell = true;
                        options.push(CallOption {
                            name: opt_name,
                            value,
                            span: name_span,
                        });
                    }
                    "args" => {
                        if seen_args {
                             return Err(self.make_error("args specified more than once", opt_start));
                        }
                        seen_args = true;
                         options.push(CallOption {
                            name: opt_name,
                            value,
                            span: name_span,
                        });
                    }
                    "allow_fail" => {
                        if !allow_allow_fail {
                            return Err(self.make_error(
                                "allow_fail is only valid on statement-form sh(...); use capture(sh(...), allow_fail=true) for expression capture",
                                opt_start,
                            ));
                        }
                        if seen_allow_fail {
                             return Err(self.make_error("allow_fail specified more than once", opt_start));
                        }
                        seen_allow_fail = true;

                        if let ExprKind::Bool(_) = value.node {
                            // OK
                        } else {
                             return Err(self.make_error("allow_fail must be a boolean", value.span));
                        }
                        options.push(CallOption {
                            name: opt_name,
                            value,
                            span: name_span,
                        });
                    }
                    _ => {
                         let msg = if allow_allow_fail {
                            format!("unknown sh() option '{}'; supported: shell, args, allow_fail", opt_name)
                        } else {
                             format!("unknown sh() option '{}'; supported: shell, args", opt_name)
                        };
                        return Err(self.make_error(&msg, opt_start));
                    }
                }
            
        }
        Ok(options)
    }
}


