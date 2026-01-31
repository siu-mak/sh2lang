use super::common::{ParsResult, Parser};
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::span::Span;
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
            TokenKind::String(s) => Ok(Expr {
                node: ExprKind::Literal(s.clone()),
                span,
            }),
            TokenKind::Sh => {
                self.expect(TokenKind::LParen)?;
                let cmd = self.parse_expr()?;
                let mut options = Vec::new();
                
                while self.match_kind(TokenKind::Comma) {
                    let opt_start = self.current_span();
                    if let Some(TokenKind::Ident(opt_name)) = self.peek_kind() {
                        let opt_name = opt_name.clone();
                        let name_span = self.advance().unwrap().span;
                        self.expect(TokenKind::Equals)?;
                        let value = self.parse_expr()?;
                        
                        match opt_name.as_str() {
                            "shell" => {
                                options.push(CallOption {
                                    name: opt_name,
                                    value,
                                    span: name_span,
                                });
                            }
                            "allow_fail" => {
                                return self.error(
                                    "allow_fail is only valid on statement-form sh(...); use capture(sh(...), allow_fail=true) for expression capture",
                                    opt_start,
                                );
                            }
                            _ => {
                                return self.error(
                                    &format!("unknown sh() option '{}'; supported: shell", opt_name),
                                    opt_start,
                                );
                            }
                        }
                    } else {
                        return self.error("expected option name", self.current_span());
                    }
                }
                
                self.expect(TokenKind::RParen)?;
                Ok(Expr {
                    node: ExprKind::Sh { cmd: Box::new(cmd), options },
                    span: span.merge(self.previous_span()),
                })
            }
            TokenKind::Dollar => {
                if let Some(TokenKind::String(s)) = self.peek_kind() {
                    let s = s.clone();
                    let str_span = self.advance().unwrap().span;
                    let full_span = span.merge(str_span);
                    self.parse_interpolated_string(&s, full_span)
                } else {
                    self.parse_command_substitution(span, false)
                }
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
                    if self.match_kind(TokenKind::LParen) {
                        if !self.match_kind(TokenKind::RParen) {
                            loop {
                                // Lookahead for named argument key=value
                                if let Some(TokenKind::Ident(_)) = self.peek_kind() {
                                    if let Some(TokenKind::Equals) = self.tokens.get(self.pos + 1).map(|t| &t.kind) {
                                        return self.error(
                                            "Named arguments are only supported for builtins: run, sudo, sh, capture, confirm",
                                            self.current_span()
                                        );
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
                        node: ExprKind::Call { name: s, args },
                        span: full_span,
                    })
                } else {
                    Ok(Expr {
                        node: ExprKind::Var(s),
                        span,
                    })
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
            TokenKind::Args => Ok(Expr {
                node: ExprKind::Args,
                span,
            }),
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
                     args.push(Expr { node: ExprKind::Literal("sh".to_string()), span: s_span });
                     
                     // Check for sh(...) shorthand
                     if self.peek_kind() == Some(&TokenKind::LParen) {
                          self.expect(TokenKind::LParen)?;
                          if !self.match_kind(TokenKind::RParen) {
                              loop {
                                   // Named args in shorthand
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
        let mut parts: Vec<Expr> = Vec::new();
        let mut i = 0;
        let mut buf = String::new();

        while i < raw.len() {
            if raw[i..].starts_with("\\$") {
                buf.push('$');
                i += 2;
                continue;
            }
            if raw[i..].starts_with("${") {
                if !buf.is_empty() {
                    parts.push(Expr {
                        node: ExprKind::Literal(std::mem::take(&mut buf)),
                        span,
                    });
                }
                let start = i + 2;
                if let Some(end_rel) = raw[start..].find('}') {
                    let end = start + end_rel;
                    let ident = &raw[start..end];
                    if is_valid_ident(ident) {
                        parts.push(Expr {
                            node: ExprKind::Var(ident.to_string()),
                            span,
                        });
                        i = end + 1;
                        continue;
                    }
                }
                buf.push('$');
                i += 1;
                continue;
            }
            if raw[i..].starts_with('$') {
                if !buf.is_empty() {
                    parts.push(Expr {
                        node: ExprKind::Literal(std::mem::take(&mut buf)),
                        span,
                    });
                }
                let start = i + 1;
                // find end of ident
                let len = raw[start..]
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(raw.len() - start);
                if len > 0 {
                    let ident = &raw[start..start + len];
                    parts.push(Expr {
                        node: ExprKind::Var(ident.to_string()),
                        span,
                    });
                    i = start + len;
                    continue;
                }
            }
            let ch = raw[i..].chars().next().unwrap();
            buf.push(ch);
            i += ch.len_utf8();
        }

        if !buf.is_empty() {
            parts.push(Expr {
                node: ExprKind::Literal(buf),
                span,
            });
        }
        if parts.is_empty() {
            return Ok(Expr {
                node: ExprKind::Literal(String::new()),
                span,
            });
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
}

fn is_valid_ident(s: &str) -> bool {
    let mut chars = s.chars();
    if let Some(first) = chars.next() {
        if !first.is_ascii_alphabetic() && first != '_' {
            return false;
        }
        for c in chars {
            if !c.is_ascii_alphanumeric() && c != '_' {
                return false;
            }
        }
        true
    } else {
        false
    }
}
