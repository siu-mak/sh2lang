use super::common::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;

impl<'a> Parser<'a> {
    pub fn parse_stmt(&mut self) -> Stmt {
        let mut left = self.parse_stmt_atom();

        loop {
            if self.match_kind(TokenKind::AndAnd) {
                let right = self.parse_stmt_atom();
                let span = left.span.merge(right.span);
                left = Stmt {
                    node: StmtKind::AndThen {
                        left: vec![left],
                        right: vec![right],
                    },
                    span,
                };
            } else if self.match_kind(TokenKind::OrOr) {
                let right = self.parse_stmt_atom();
                let span = left.span.merge(right.span);
                left = Stmt {
                    node: StmtKind::OrElse {
                        left: vec![left],
                        right: vec![right],
                    },
                    span,
                };
            } else {
                break;
            }
        }
        left
    }

    fn parse_stmt_atom(&mut self) -> Stmt {
        let start_span = self.current_span();
        let kind = self.peek_kind().cloned();

        if kind.is_none() {
            self.error("Expected statement, got EOF", start_span);
        }
        let kind = kind.unwrap();

        let stmt_kind = match kind {
            TokenKind::Let => {
                self.advance();
                let name = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                    s.clone()
                } else {
                    self.error("Expected variable name after let", self.current_span());
                };
                self.advance();
                self.expect(TokenKind::Equals);
                let value = self.parse_expr();
                StmtKind::Let { name, value }
            }
            TokenKind::Run => {
                // We dispatch to helper that does NOT consume Run token?
                // Old parser: `parse_run_call` expected Run.
                // So we can call it.

                let mut segments = Vec::new();
                segments.push(self.parse_run_call());

                while self.match_kind(TokenKind::Pipe) {
                    if self.peek_kind() == Some(&TokenKind::Run) {
                        segments.push(self.parse_run_call());
                    } else {
                        self.error(
                            "expected run(...) after '|' in pipeline",
                            self.current_span(),
                        );
                    }
                }

                if segments.len() == 1 {
                    StmtKind::Run(segments.pop().unwrap())
                } else {
                    StmtKind::Pipe(segments)
                }
            }
            TokenKind::Exec => {
                self.advance();
                self.expect(TokenKind::LParen);
                let mut args = Vec::new();
                if !self.match_kind(TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr());
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen);
                }
                if args.is_empty() {
                    self.error("exec requires at least one argument", self.current_span());
                }
                StmtKind::Exec(args)
            }
            TokenKind::Print => {
                self.advance();
                self.expect(TokenKind::LParen);
                let expr = self.parse_expr();
                self.expect(TokenKind::RParen);
                StmtKind::Print(expr)
            }
            TokenKind::PrintErr => {
                self.advance();
                self.expect(TokenKind::LParen);
                let expr = self.parse_expr();
                self.expect(TokenKind::RParen);
                StmtKind::PrintErr(expr)
            }
            TokenKind::If => {
                self.advance();
                let cond = self.parse_expr();
                self.expect(TokenKind::LBrace);
                let mut then_body = Vec::new();
                while !self.match_kind(TokenKind::RBrace) {
                    then_body.push(self.parse_stmt());
                }

                let mut elifs = Vec::new();
                loop {
                    if self.match_kind(TokenKind::Elif) {
                        let cond = self.parse_expr();
                        self.expect(TokenKind::LBrace);
                        let mut body = Vec::new();
                        while !self.match_kind(TokenKind::RBrace) {
                            body.push(self.parse_stmt());
                        }
                        elifs.push(Elif { cond, body });
                    } else if self.peek_kind() == Some(&TokenKind::Else) {
                        // Check if `else if` (legacy/compat?)
                        // "else if" logic:
                        if self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::If) {
                            self.advance(); // else
                            self.advance(); // if
                            let cond = self.parse_expr();
                            self.expect(TokenKind::LBrace);
                            let mut body = Vec::new();
                            while !self.match_kind(TokenKind::RBrace) {
                                body.push(self.parse_stmt());
                            }
                            elifs.push(Elif { cond, body });
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                let else_body = if self.match_kind(TokenKind::Else) {
                    self.expect(TokenKind::LBrace);
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt());
                    }
                    Some(body)
                } else {
                    None
                };

                StmtKind::If {
                    cond,
                    then_body,
                    elifs,
                    else_body,
                }
            }
            TokenKind::Case => {
                self.advance();
                let expr = self.parse_expr();
                self.expect(TokenKind::LBrace);
                let mut arms = Vec::new();
                while !self.match_kind(TokenKind::RBrace) {
                    let mut patterns = Vec::new();
                    loop {
                        if let Some(TokenKind::String(s)) = self.peek_kind() {
                            let s = s.clone();
                            self.advance();
                            patterns.push(Pattern::Literal(s));
                        } else if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                            if s == "glob" {
                                self.advance();
                                self.expect(TokenKind::LParen);
                                if let Some(TokenKind::String(p)) = self.peek_kind() {
                                    patterns.push(Pattern::Glob(p.clone()));
                                    self.advance();
                                } else {
                                    self.error(
                                        "Expected string literal for glob",
                                        self.current_span(),
                                    );
                                }
                                self.expect(TokenKind::RParen);
                            } else {
                                self.error(
                                    "Expected string, glob(\"...\"), or _",
                                    self.current_span(),
                                );
                            }
                        } else if self.match_kind(TokenKind::Underscore) {
                            patterns.push(Pattern::Wildcard);
                        } else {
                            self.error("Expected pattern", self.current_span());
                        }

                        if !self.match_kind(TokenKind::Pipe) {
                            break;
                        }
                    }
                    self.expect(TokenKind::Arrow);
                    self.expect(TokenKind::LBrace);
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt());
                    }
                    arms.push(CaseArm { patterns, body });
                }
                StmtKind::Case { expr, arms }
            }
            TokenKind::While => {
                self.advance();
                let cond = self.parse_expr();
                self.expect(TokenKind::LBrace);
                let mut body = Vec::new();
                while !self.match_kind(TokenKind::RBrace) {
                    body.push(self.parse_stmt());
                }
                StmtKind::While { cond, body }
            }
            TokenKind::For => {
                self.advance();
                if self.match_kind(TokenKind::LParen) {
                    let key_var = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                        s.clone()
                    } else {
                        self.error("Expected ident", self.current_span())
                    };
                    self.advance();
                    self.expect(TokenKind::Comma);
                    let val_var = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                        s.clone()
                    } else {
                        self.error("Expected ident", self.current_span())
                    };
                    self.advance();
                    self.expect(TokenKind::RParen);
                    self.expect(TokenKind::In);

                    let map = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                        s.clone()
                    } else {
                        self.error("Expected map ident", self.current_span())
                    };
                    self.advance();

                    self.expect(TokenKind::LBrace);
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt());
                    }
                    StmtKind::ForMap {
                        key_var,
                        val_var,
                        map,
                        body,
                    }
                } else {
                    let var = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                        s.clone()
                    } else {
                        self.error("Expected ident", self.current_span())
                    };
                    self.advance();
                    self.expect(TokenKind::In);

                    let items = if self.match_kind(TokenKind::LParen) {
                        let mut items = Vec::new();
                        if !self.match_kind(TokenKind::RParen) {
                            loop {
                                items.push(self.parse_expr());
                                if !self.match_kind(TokenKind::Comma) {
                                    break;
                                }
                            }
                            self.expect(TokenKind::RParen);
                        }
                        items
                    } else {
                        vec![self.parse_expr()]
                    };

                    self.expect(TokenKind::LBrace);
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt());
                    }
                    StmtKind::For { var, items, body }
                }
            }
            TokenKind::Break => {
                self.advance();
                StmtKind::Break
            }
            TokenKind::Continue => {
                self.advance();
                StmtKind::Continue
            }
            TokenKind::Return => {
                self.advance();
                let val = if is_expr_start(self.peek_kind()) {
                    Some(self.parse_expr())
                } else {
                    None
                };
                StmtKind::Return(val)
            }
            TokenKind::Exit => {
                self.advance();
                let code = if is_expr_start(self.peek_kind()) {
                    Some(self.parse_expr())
                } else {
                    None
                };
                StmtKind::Exit(code)
            }
            TokenKind::With => {
                self.advance();
                if self.match_kind(TokenKind::Env) {
                    self.expect(TokenKind::LBrace);
                    let mut bindings = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        let name = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                            s.clone()
                        } else {
                            self.error("Expected ident", self.current_span())
                        };
                        self.advance();
                        if self.match_kind(TokenKind::Colon) || self.match_kind(TokenKind::Equals) {
                            // consumed
                        } else {
                            self.error("Expected : or = after env key", self.current_span());
                        }
                        let val = self.parse_expr();
                        bindings.push((name, val));
                        self.match_kind(TokenKind::Comma);
                    }
                    self.expect(TokenKind::LBrace);
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt());
                    }
                    StmtKind::WithEnv { bindings, body }
                } else if self.match_kind(TokenKind::Cwd) {
                    let path = self.parse_expr();
                    self.expect(TokenKind::LBrace);
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt());
                    }
                    StmtKind::WithCwd { path, body }
                } else if self.match_kind(TokenKind::Redirect) {
                    self.expect(TokenKind::LBrace);
                    let mut stdout = None;
                    let mut stderr = None;
                    let mut stdin = None;
                    while !self.match_kind(TokenKind::RBrace) {
                        if self.match_kind(TokenKind::Stdout) {
                            self.expect(TokenKind::Colon);
                            let t = self.parse_redirect_target();
                            if let RedirectTarget::HereDoc { .. } = t {
                                self.error("heredoc only allowed for stdin", self.previous_span());
                            }
                            stdout = Some(t);
                        } else if self.match_kind(TokenKind::Stderr) {
                            self.expect(TokenKind::Colon);
                            let t = self.parse_redirect_target();
                            if let RedirectTarget::HereDoc { .. } = t {
                                self.error("heredoc only allowed for stdin", self.previous_span());
                            }
                            stderr = Some(t);
                        } else if self.match_kind(TokenKind::Stdin) {
                            self.expect(TokenKind::Colon);
                            let t = self.parse_redirect_target();
                            match t {
                                RedirectTarget::HereDoc { .. } => {}
                                RedirectTarget::File { append, .. } => {
                                    if append {
                                        self.error("Cannot append to stdin", self.previous_span());
                                    }
                                }
                                _ => self.error(
                                    "stdin can only be redirected from a file or heredoc",
                                    self.previous_span(),
                                ),
                            }
                            stdin = Some(t);
                        } else {
                            self.error("Expected stdout, stderr, or stdin", self.current_span());
                        }
                        self.match_kind(TokenKind::Comma);
                    }
                    self.expect(TokenKind::LBrace);
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt());
                    }
                    StmtKind::WithRedirect {
                        stdout,
                        stderr,
                        stdin,
                        body,
                    }
                } else if self.match_kind(TokenKind::Log) {
                    self.expect(TokenKind::LParen);
                    let path = self.parse_expr();
                    let mut append = false;
                    while self.match_kind(TokenKind::Comma) {
                        if self.match_kind(TokenKind::Append) {
                            self.expect(TokenKind::Equals);
                            if self.match_kind(TokenKind::True) {
                                append = true;
                            } else if self.match_kind(TokenKind::False) {
                                append = false;
                            } else {
                                self.error("append must be true/false", self.current_span());
                            }
                        } else {
                            self.error("Expected option name (e.g. append)", self.current_span());
                        }
                    }
                    self.expect(TokenKind::RParen);
                    self.expect(TokenKind::LBrace);
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt());
                    }
                    StmtKind::WithLog { path, append, body }
                } else {
                    self.error(
                        "Expected 'env', 'cwd', 'redirect', or 'log' after 'with'",
                        self.current_span(),
                    );
                }
            }
            TokenKind::Spawn => {
                self.advance();
                if self.match_kind(TokenKind::LBrace) {
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt());
                    }
                    StmtKind::Spawn {
                        stmt: Box::new(Stmt {
                            node: StmtKind::Group { body },
                            span: start_span.merge(self.previous_span()),
                        }),
                    }
                } else {
                    let stmt = self.parse_stmt();
                    StmtKind::Spawn {
                        stmt: Box::new(stmt),
                    }
                }
            }
            TokenKind::Wait => {
                self.advance();
                self.expect(TokenKind::LParen);
                let expr = if is_expr_start(self.peek_kind()) {
                    Some(self.parse_expr())
                } else {
                    None
                };
                self.expect(TokenKind::RParen);
                StmtKind::Wait(expr)
            }
            TokenKind::Try => {
                self.advance();
                self.expect(TokenKind::LBrace);
                let mut try_body = Vec::new();
                while !self.match_kind(TokenKind::RBrace) {
                    try_body.push(self.parse_stmt());
                }
                self.expect(TokenKind::Catch);
                self.expect(TokenKind::LBrace);
                let mut catch_body = Vec::new();
                while !self.match_kind(TokenKind::RBrace) {
                    catch_body.push(self.parse_stmt());
                }
                StmtKind::TryCatch {
                    try_body,
                    catch_body,
                }
            }
            TokenKind::Cd => {
                self.advance();
                self.expect(TokenKind::LParen);
                let path = self.parse_expr();
                self.expect(TokenKind::RParen);
                StmtKind::Cd { path }
            }
            TokenKind::Export => {
                self.advance();
                self.expect(TokenKind::LParen);
                let name = if let Some(TokenKind::String(s)) = self.peek_kind() {
                    s.clone()
                } else {
                    self.error("Expected string", self.current_span())
                };
                self.advance();
                let value = if self.match_kind(TokenKind::Comma) {
                    Some(self.parse_expr())
                } else {
                    None
                };
                self.expect(TokenKind::RParen);
                StmtKind::Export { name, value }
            }
            TokenKind::Unset => {
                self.advance();
                self.expect(TokenKind::LParen);
                let name = if let Some(TokenKind::String(s)) = self.peek_kind() {
                    s.clone()
                } else {
                    self.error("Expected string", self.current_span())
                };
                self.advance();
                self.expect(TokenKind::RParen);
                StmtKind::Unset { name }
            }
            TokenKind::Source => {
                self.advance();
                self.expect(TokenKind::LParen);
                let path = self.parse_expr();
                self.expect(TokenKind::RParen);
                StmtKind::Source { path }
            }
            TokenKind::Subshell => {
                self.advance();
                self.expect(TokenKind::LBrace);
                let mut body = Vec::new();
                while !self.match_kind(TokenKind::RBrace) {
                    body.push(self.parse_stmt());
                }
                StmtKind::Subshell { body }
            }
            TokenKind::Group => {
                self.advance();
                self.expect(TokenKind::LBrace);
                let mut body = Vec::new();
                while !self.match_kind(TokenKind::RBrace) {
                    body.push(self.parse_stmt());
                }
                StmtKind::Group { body }
            }
            TokenKind::Sh => {
                self.advance();
                if self.match_kind(TokenKind::LParen) {
                    let s = if let Some(TokenKind::String(s)) = self.peek_kind() {
                        s.clone()
                    } else {
                        self.error("Expected string", self.current_span())
                    };
                    self.advance();
                    self.expect(TokenKind::RParen);
                    StmtKind::Sh(s)
                } else if self.match_kind(TokenKind::LBrace) {
                    let mut lines = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        if let Some(TokenKind::String(s)) = self.peek_kind() {
                            lines.push(s.clone());
                            self.advance();
                        } else {
                            self.error(
                                "Expected string literal in sh {{ ... }}",
                                self.current_span(),
                            );
                        }
                        if !self.match_kind(TokenKind::Comma) {
                            if self.peek_kind() != Some(&TokenKind::RBrace) {
                                self.error("Expected comma or closing brace", self.current_span());
                            }
                        }
                    }
                    StmtKind::ShBlock(lines)
                } else {
                    self.error("Expected ( or {{ after sh", self.current_span());
                }
            }
            TokenKind::Set => {
                self.advance();
                let target = if let Some(TokenKind::Ident(name)) = self.peek_kind() {
                    let name = name.clone();
                    self.advance();
                    LValue::Var(name)
                } else if self.match_kind(TokenKind::Env) {
                    self.expect(TokenKind::Dot);
                    let name = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                        s.clone()
                    } else {
                        self.error("Expected ident", self.current_span())
                    };
                    self.advance();
                    LValue::Env(name)
                } else {
                    self.error("Expected ident or env.VAR", self.current_span());
                };
                self.expect(TokenKind::Equals);
                let value = self.parse_expr();
                StmtKind::Set { target, value }
            }
            TokenKind::PipeKw => {
                self.advance();
                let mut segments = Vec::new();

                segments.push(self.parse_pipe_segment());
                if !self.match_kind(TokenKind::Pipe) {
                    self.error("pipe requires at least two segments", self.current_span());
                }
                segments.push(self.parse_pipe_segment());

                while self.match_kind(TokenKind::Pipe) {
                    segments.push(self.parse_pipe_segment());
                }
                StmtKind::PipeBlocks { segments }
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                self.expect(TokenKind::LParen);
                let mut args = Vec::new();
                if !self.match_kind(TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr());
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen);
                }
                StmtKind::Call { name, args }
            }
            _ => self.error(&format!("Expected statement, got {:?}", kind), start_span),
        };

        Stmt {
            node: stmt_kind,
            span: start_span.merge(self.previous_span()),
        }
    }

    fn parse_run_call(&mut self) -> RunCall {
        self.expect(TokenKind::Run);
        self.expect(TokenKind::LParen);
        let mut args = Vec::new();
        let mut options = Vec::new();

        while !self.match_kind(TokenKind::RParen) {
            // Check for named arg: IDENT = ...
            let is_named_arg = if let Some(TokenKind::Ident(_)) = self.peek_kind() {
                self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::Equals)
            } else {
                false
            };

            if is_named_arg {
                let start_span = self.current_span();
                let name = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                    s.clone()
                } else {
                    unreachable!()
                };
                self.advance(); // consume ident
                let name_span = start_span.merge(self.previous_span());
                self.expect(TokenKind::Equals);
                let value = self.parse_expr();
                
                options.push(RunOption {
                    name,
                    value,
                    span: name_span,
                });
            } else {
                args.push(self.parse_expr());
            }

            self.match_kind(TokenKind::Comma);
        }
        RunCall { args, options }
    }

    fn parse_pipe_segment(&mut self) -> Vec<Stmt> {
        if self.match_kind(TokenKind::LBrace) {
            let mut body = Vec::new();
            while !self.match_kind(TokenKind::RBrace) {
                body.push(self.parse_stmt());
            }
            body
        } else {
            vec![self.parse_stmt_atom()]
        }
    }

    fn parse_redirect_target(&mut self) -> RedirectTarget {
        if self.match_kind(TokenKind::File) {
            self.expect(TokenKind::LParen);
            let path = self.parse_expr();
            let mut append = false;
            if self.match_kind(TokenKind::Comma) {
                if self.match_kind(TokenKind::Append) {
                    self.expect(TokenKind::Equals);
                    if self.match_kind(TokenKind::True) {
                        append = true;
                    } else if self.match_kind(TokenKind::False) {
                        append = false;
                    } else if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                        if s == "true" {
                            self.advance();
                            append = true;
                        } else if s == "false" {
                            self.advance();
                            append = false;
                        } else {
                            self.error("append must be bool", self.current_span());
                        }
                    } else {
                        self.error("append must be bool", self.current_span());
                    }
                }
            }
            self.expect(TokenKind::RParen);
            RedirectTarget::File { path, append }
        } else if self.match_kind(TokenKind::Stdout) {
            RedirectTarget::Stdout
        } else if self.match_kind(TokenKind::Stderr) {
            RedirectTarget::Stderr
        } else if let Some(TokenKind::Ident(s)) = self.peek_kind() {
            if s == "heredoc" {
                self.advance();
                self.expect(TokenKind::LParen);
                let content = if let Some(TokenKind::String(s)) = self.peek_kind() {
                    s.clone()
                } else {
                    self.error("Expected string", self.current_span())
                };
                self.advance();
                self.expect(TokenKind::RParen);
                RedirectTarget::HereDoc { content }
            } else {
                self.error("Expected redirect target", self.current_span());
            }
        } else {
            self.error("Expected redirect target", self.current_span());
        }
    }
}

fn is_expr_start(k: Option<&TokenKind>) -> bool {
    matches!(
        k,
        Some(
            TokenKind::String(_)
                | TokenKind::Ident(_)
                | TokenKind::Dollar
                | TokenKind::LParen
                | TokenKind::LBracket
                | TokenKind::Env
                | TokenKind::Args
                | TokenKind::Capture
                | TokenKind::Exists
                | TokenKind::IsDir
                | TokenKind::IsFile
                | TokenKind::IsSymlink
                | TokenKind::IsExec
                | TokenKind::IsReadable
                | TokenKind::IsWritable
                | TokenKind::IsNonEmpty
                | TokenKind::BoolStr
                | TokenKind::Len
                | TokenKind::Arg
                | TokenKind::Index
                | TokenKind::Join
                | TokenKind::Status
                | TokenKind::Pid
                | TokenKind::Count
                | TokenKind::Uid
                | TokenKind::Ppid
                | TokenKind::Pwd
                | TokenKind::SelfPid
                | TokenKind::Argv0
                | TokenKind::Argc
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Number(_)
                | TokenKind::Minus
                | TokenKind::Bang
                | TokenKind::Input
                | TokenKind::Confirm
        )
    )
}
