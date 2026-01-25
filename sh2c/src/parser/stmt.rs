use super::common::{ParsResult, Parser};
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::span::Span;
use crate::sudo::SudoSpec;

impl<'a> Parser<'a> {
    pub fn parse_stmt(&mut self) -> ParsResult<Stmt> {
        let mut left = self.parse_stmt_atom()?;

        loop {
            if self.match_kind(TokenKind::AndAnd) {
                let right = self.parse_stmt_atom()?;
                let span = left.span.merge(right.span);
                left = Stmt {
                    node: StmtKind::AndThen {
                        left: vec![left],
                        right: vec![right],
                    },
                    span,
                };
            } else if self.match_kind(TokenKind::OrOr) {
                let right = self.parse_stmt_atom()?;
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
        Ok(left)
    }

    fn parse_stmt_atom(&mut self) -> ParsResult<Stmt> {
        let start_span = self.current_span();
        let kind = self.peek_kind().cloned();

        if kind.is_none() {
            self.error("Expected statement, got EOF", start_span)?;
        }
        let kind = kind.unwrap();

        let stmt_kind = match kind {
            TokenKind::Let => {
                self.advance();
                let name = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                    s.clone()
                } else {
                    self.error("Expected variable name after let", self.current_span())?
                };
                self.advance();
                self.expect(TokenKind::Equals)?;
                let value = self.parse_expr()?;
                StmtKind::Let { name, value }
            }
            TokenKind::Run => {
                let mut segments = Vec::new();
                segments.push(self.parse_run_call()?);

                while self.match_kind(TokenKind::Pipe) {
                    if self.peek_kind() == Some(&TokenKind::Run) {
                        segments.push(self.parse_run_call()?);
                    } else {
                        self.error(
                            "expected run(...) after '|' in pipeline",
                            self.current_span(),
                        )?;
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
                self.expect(TokenKind::LParen)?;
                let mut args = Vec::new();
                if !self.match_kind(TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if !self.match_kind(TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                }
                if args.is_empty() {
                    self.error("exec requires at least one argument", self.current_span())?;
                }
                StmtKind::Exec(args)
            }
            TokenKind::Print => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                StmtKind::Print(expr)
            }
            TokenKind::PrintErr => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                StmtKind::PrintErr(expr)
            }
            TokenKind::If => {
                self.advance();
                let cond = self.parse_expr()?;
                self.expect(TokenKind::LBrace)?;
                let mut then_body = Vec::new();
                while !self.match_kind(TokenKind::RBrace) {
                    then_body.push(self.parse_stmt()?);
                }

                let mut elifs = Vec::new();
                loop {
                    if self.match_kind(TokenKind::Elif) {
                        let cond = self.parse_expr()?;
                        self.expect(TokenKind::LBrace)?;
                        let mut body = Vec::new();
                        while !self.match_kind(TokenKind::RBrace) {
                            body.push(self.parse_stmt()?);
                        }
                        elifs.push(Elif { cond, body });
                    } else if self.peek_kind() == Some(&TokenKind::Else) {
                        // Check if `else if` (legacy/compat?)
                        // "else if" logic:
                        if self.tokens.get(self.pos + 1).map(|t| &t.kind) == Some(&TokenKind::If) {
                            self.advance(); // else
                            self.advance(); // if
                            let cond = self.parse_expr()?;
                            self.expect(TokenKind::LBrace)?;
                            let mut body = Vec::new();
                            while !self.match_kind(TokenKind::RBrace) {
                                body.push(self.parse_stmt()?);
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
                    self.expect(TokenKind::LBrace)?;
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt()?);
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
                let expr = self.parse_expr()?;
                self.expect(TokenKind::LBrace)?;
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
                                self.expect(TokenKind::LParen)?;
                                if let Some(TokenKind::String(p)) = self.peek_kind() {
                                    patterns.push(Pattern::Glob(p.clone()));
                                    self.advance();
                                } else {
                                    self.error(
                                        "Expected string literal for glob",
                                        self.current_span(),
                                    )?;
                                }
                                self.expect(TokenKind::RParen)?;
                            } else {
                                self.error(
                                    "Expected string, glob(\"...\"), or _",
                                    self.current_span(),
                                )?;
                            }
                        } else if self.match_kind(TokenKind::Underscore) {
                            patterns.push(Pattern::Wildcard);
                        } else {
                            self.error("Expected pattern", self.current_span())?;
                        }

                        if !self.match_kind(TokenKind::Pipe) {
                            break;
                        }
                    }
                    self.expect(TokenKind::Arrow)?;
                    self.expect(TokenKind::LBrace)?;
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt()?);
                    }
                    arms.push(CaseArm { patterns, body });
                }
                StmtKind::Case { expr, arms }
            }
            TokenKind::While => {
                self.advance();
                let cond = self.parse_expr()?;
                self.expect(TokenKind::LBrace)?;
                let mut body = Vec::new();
                while !self.match_kind(TokenKind::RBrace) {
                    body.push(self.parse_stmt()?);
                }
                StmtKind::While { cond, body }
            }
            TokenKind::For => {
                self.advance();
                if self.match_kind(TokenKind::LParen) {
                    let key_var = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                        s.clone()
                    } else {
                        self.error("Expected ident", self.current_span())?
                    };
                    self.advance();
                    self.expect(TokenKind::Comma)?;
                    let val_var = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                        s.clone()
                    } else {
                        self.error("Expected ident", self.current_span())?
                    };
                    self.advance();
                    self.expect(TokenKind::RParen)?;
                    self.expect(TokenKind::In)?;

                    let map = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                        s.clone()
                    } else {
                        self.error("Expected map ident", self.current_span())?
                    };
                    self.advance();

                    self.expect(TokenKind::LBrace)?;
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt()?);
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
                        self.error("Expected ident", self.current_span())?
                    };
                    self.advance();
                    self.expect(TokenKind::In)?;

                    let items = if self.match_kind(TokenKind::LParen) {
                        let mut items = Vec::new();
                        if !self.match_kind(TokenKind::RParen) {
                            loop {
                                items.push(self.parse_expr()?);
                                if !self.match_kind(TokenKind::Comma) {
                                    break;
                                }
                            }
                            self.expect(TokenKind::RParen)?;
                        }
                        items
                    } else {
                        vec![self.parse_expr()?]
                    };

                    self.expect(TokenKind::LBrace)?;
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt()?);
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
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                StmtKind::Return(val)
            }
            TokenKind::Exit => {
                self.advance();
                let code = if is_expr_start(self.peek_kind()) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                StmtKind::Exit(code)
            }
            TokenKind::With => {
                self.advance();
                if self.match_kind(TokenKind::Env) {
                    self.expect(TokenKind::LBrace)?;
                    let mut bindings = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        let name = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                            s.clone()
                        } else {
                            self.error("Expected ident", self.current_span())?
                        };
                        self.advance();
                        if self.match_kind(TokenKind::Colon) || self.match_kind(TokenKind::Equals) {
                            // consumed
                        } else {
                            self.error("Expected : or = after env key", self.current_span())?;
                        }
                        let val = self.parse_expr()?;
                        bindings.push((name, val));
                        self.match_kind(TokenKind::Comma);
                    }
                    self.expect(TokenKind::LBrace)?;
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt()?);
                    }
                    StmtKind::WithEnv { bindings, body }
                } else if self.match_kind(TokenKind::Cwd) {
                    let path = self.parse_expr()?;
                    self.expect(TokenKind::LBrace)?;
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt()?);
                    }
                    StmtKind::WithCwd { path, body }
                } else if self.match_kind(TokenKind::Redirect) {
                    self.expect(TokenKind::LBrace)?;
                    let mut stdout = None;
                    let mut stderr = None;
                    let mut stdin = None;
                    while !self.match_kind(TokenKind::RBrace) {
                        if self.match_kind(TokenKind::Stdout) {
                            self.expect(TokenKind::Colon)?;
                            let targets = self.parse_redirect_output_list("stdout")?;
                            stdout = if targets.is_empty() { None } else { Some(targets) };
                        } else if self.match_kind(TokenKind::Stderr) {
                            self.expect(TokenKind::Colon)?;
                            let targets = self.parse_redirect_output_list("stderr")?;
                            stderr = if targets.is_empty() { None } else { Some(targets) };
                        } else if self.match_kind(TokenKind::Stdin) {
                            self.expect(TokenKind::Colon)?;
                            // Check for list form (not supported for stdin)
                            if self.match_kind(TokenKind::LBracket) {
                                self.error("stdin does not support multi-sink redirect", self.previous_span())?;
                            }
                            let t = self.parse_redirect_input_target()?;
                            stdin = Some(t);
                        } else {
                            self.error("Expected stdout, stderr, or stdin", self.current_span())?;
                        }
                        self.match_kind(TokenKind::Comma);
                    }
                    self.expect(TokenKind::LBrace)?;
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt()?);
                    }
                    StmtKind::WithRedirect {
                        stdout,
                        stderr,
                        stdin,
                        body,
                    }
                } else if self.match_kind(TokenKind::Log) {
                    self.expect(TokenKind::LParen)?;
                    let path = self.parse_expr()?;
                    let mut append = false;
                    while self.match_kind(TokenKind::Comma) {
                        if self.match_kind(TokenKind::Append) {
                            self.expect(TokenKind::Equals)?;
                            if self.match_kind(TokenKind::True) {
                                append = true;
                            } else if self.match_kind(TokenKind::False) {
                                append = false;
                            } else {
                                self.error("append must be true/false", self.current_span())?;
                            }
                        } else {
                            self.error("Expected option name (e.g. append)", self.current_span())?;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    self.expect(TokenKind::LBrace)?;
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt()?);
                    }
                    StmtKind::WithLog { path, append, body }
                } else {
                    self.error(
                        "Expected 'env', 'cwd', 'redirect', or 'log' after 'with'",
                        self.current_span(),
                    )?
                }
            }
            TokenKind::Spawn => {
                self.advance();
                if self.match_kind(TokenKind::LBrace) {
                    let mut body = Vec::new();
                    while !self.match_kind(TokenKind::RBrace) {
                        body.push(self.parse_stmt()?);
                    }
                    StmtKind::Spawn {
                        stmt: Box::new(Stmt {
                            node: StmtKind::Group { body },
                            span: start_span.merge(self.previous_span()),
                        }),
                    }
                } else {
                    let stmt = self.parse_stmt()?;
                    StmtKind::Spawn {
                        stmt: Box::new(stmt),
                    }
                }
            }
            TokenKind::Wait => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = if is_expr_start(self.peek_kind()) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.expect(TokenKind::RParen)?;
                StmtKind::Wait(expr)
            }
            TokenKind::Try => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut try_body = Vec::new();
                while !self.match_kind(TokenKind::RBrace) {
                    try_body.push(self.parse_stmt()?);
                }
                self.expect(TokenKind::Catch)?;
                self.expect(TokenKind::LBrace)?;
                let mut catch_body = Vec::new();
                while !self.match_kind(TokenKind::RBrace) {
                    catch_body.push(self.parse_stmt()?);
                }
                StmtKind::TryCatch {
                    try_body,
                    catch_body,
                }
            }
            TokenKind::Cd => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let path = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                StmtKind::Cd { path }
            }
            TokenKind::Export => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let name = if let Some(TokenKind::String(s)) = self.peek_kind() {
                    s.clone()
                } else {
                    self.error("Expected string", self.current_span())?
                };
                self.advance();
                let value = if self.match_kind(TokenKind::Comma) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.expect(TokenKind::RParen)?;
                StmtKind::Export { name, value }
            }
            TokenKind::Unset => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let name = if let Some(TokenKind::String(s)) = self.peek_kind() {
                    s.clone()
                } else {
                    self.error("Expected string", self.current_span())?
                };
                self.advance();
                self.expect(TokenKind::RParen)?;
                StmtKind::Unset { name }
            }
            TokenKind::Source => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let path = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                StmtKind::Source { path }
            }
            TokenKind::Subshell => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut body = Vec::new();
                while !self.match_kind(TokenKind::RBrace) {
                    body.push(self.parse_stmt()?);
                }
                StmtKind::Subshell { body }
            }
            TokenKind::Group => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut body = Vec::new();
                while !self.match_kind(TokenKind::RBrace) {
                    body.push(self.parse_stmt()?);
                }
                StmtKind::Group { body }
            }
            TokenKind::Sh => {
                self.advance();
                if self.match_kind(TokenKind::LParen) {
                    let expr = self.parse_expr()?;
                    self.expect(TokenKind::RParen)?;
                    StmtKind::Sh(expr)
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
                            )?;
                        }
                        if !self.match_kind(TokenKind::Comma) {
                            if self.peek_kind() != Some(&TokenKind::RBrace) {
                                self.error("Expected comma or closing brace", self.current_span())?;
                            }
                        }
                    }
                    StmtKind::ShBlock(lines)
                } else {
                    self.error("Expected ( or {{ after sh", self.current_span())?
                }
            }
            TokenKind::Set => {
                self.advance();
                let target = if let Some(TokenKind::Ident(name)) = self.peek_kind() {
                    let name = name.clone();
                    self.advance();
                    LValue::Var(name)
                } else if self.match_kind(TokenKind::Env) {
                    self.expect(TokenKind::Dot)?;
                    let name = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                        s.clone()
                    } else {
                        self.error("Expected ident", self.current_span())?
                    };
                    self.advance();
                    LValue::Env(name)
                } else {
                    self.error("Expected ident or env.VAR", self.current_span())?
                };
                self.expect(TokenKind::Equals)?;
                let value = self.parse_expr()?;
                StmtKind::Set { target, value }
            }
            TokenKind::PipeKw => {
                self.advance();
                let mut segments = Vec::new();

                segments.push(self.parse_pipe_segment()?);
                if !self.match_kind(TokenKind::Pipe) {
                    self.error("pipe requires at least two segments", self.current_span())?;
                }
                segments.push(self.parse_pipe_segment()?);

                while self.match_kind(TokenKind::Pipe) {
                    segments.push(self.parse_pipe_segment()?);
                }
                StmtKind::PipeBlocks { segments }
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                
                // Special handling for statement-form sh()
                if name == "sh" {
                    self.expect(TokenKind::LParen)?;
                    let cmd = self.parse_expr()?;
                    
                    // Parse options: shell=..., allow_fail=...
                    let mut shell_expr = Expr {
                        node: ExprKind::Literal("sh".to_string()),
                        span: start_span,
                    };
                    let mut options = Vec::new();
                    let mut seen_shell = false;
                    let mut seen_allow_fail = false;
                    
                    while self.match_kind(TokenKind::Comma) {
                        let opt_start = self.current_span();
                        if let Some(TokenKind::Ident(opt_name)) = self.peek_kind() {
                            let opt_name = opt_name.clone();
                            let name_span = self.advance().unwrap().span;
                            self.expect(TokenKind::Equals)?;
                            let value = self.parse_expr()?;
                            
                            match opt_name.as_str() {
                                "shell" => {
                                    if seen_shell {
                                        self.error("shell specified more than once", opt_start)?;
                                    }
                                    seen_shell = true;
                                    shell_expr = value;
                                }
                                "allow_fail" => {
                                    if seen_allow_fail {
                                        self.error("allow_fail specified more than once", opt_start)?;
                                    }
                                    seen_allow_fail = true;
                                    options.push(RunOption {
                                        name: opt_name,
                                        value,
                                        span: name_span,
                                    });
                                }
                                _ => {
                                    return self.error(
                                        &format!("unknown sh() option '{}'; supported: shell, allow_fail", opt_name),
                                        opt_start,
                                    );
                                }
                            }
                        } else {
                            return self.error("expected option name", self.current_span());
                        }
                    }
                    
                    self.expect(TokenKind::RParen)?;
                    
                    // Build argv: [shell, "-c", cmd]
                    let args = vec![
                        shell_expr,
                        Expr {
                            node: ExprKind::Literal("-c".to_string()),
                            span: start_span,
                        },
                        cmd,
                    ];
                    
                    StmtKind::Run(RunCall { args, options })
                } else if name == "sudo" {
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
                                let name_token = if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                                    s.clone()
                                } else {
                                    unreachable!()
                                };
                                let name_span = self.advance().unwrap().span;
                                self.expect(TokenKind::Equals)?;
                                let value = self.parse_expr()?;

                                options.push(RunOption {
                                    name: name_token,
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
                    let spec = match SudoSpec::from_options(&options) {
                         Ok(s) => s,
                         Err((msg, span)) => return self.error(&msg, span),
                    };

                    if args.is_empty() {
                        return self.error(
                            "sudo() requires at least one positional argument (the command)",
                            start_span,
                        );
                    }
                    
                    // Construct argv: ["sudo", flags..., cmd_args...]
                    let mut run_args = Vec::new();
                    run_args.push(Expr {
                        node: ExprKind::Literal("sudo".to_string()),
                        span: start_span,
                    });
                    
                    for flag in spec.to_flags_argv() {
                        run_args.push(Expr {
                            node: ExprKind::Literal(flag),
                            span: start_span, // Using call span for generated flags
                        });
                    }
                    
                    run_args.extend(args);

                    // Handle allow_fail: passing it to StmtKind::Run options
                    let mut run_options = Vec::new();
                    if let Some((allow, span)) = spec.allow_fail {
                        run_options.push(RunOption {
                            name: "allow_fail".to_string(),
                            value: Expr {
                                node: ExprKind::Bool(allow),
                                span,
                            },
                            span,
                        });
                    }

                    StmtKind::Run(RunCall {
                        args: run_args,
                        options: run_options,
                    })                } else {
                    self.expect(TokenKind::LParen)?;
                    let mut args = Vec::new();
                    if !self.match_kind(TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if !self.match_kind(TokenKind::Comma) {
                                break;
                            }
                        }
                        self.expect(TokenKind::RParen)?;
                    }
                    StmtKind::Call { name, args }
                }
            }
            _ => self.error(&format!("Expected statement, got {:?}", kind), start_span)?,
        };

        Ok(Stmt {
            node: stmt_kind,
            span: start_span.merge(self.previous_span()),
        })
    }

    fn parse_run_call(&mut self) -> ParsResult<RunCall> {
        self.expect(TokenKind::Run)?;
        self.expect(TokenKind::LParen)?;
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
                self.expect(TokenKind::Equals)?;
                let value = self.parse_expr()?;
                
                options.push(RunOption {
                    name,
                    value,
                    span: name_span,
                });
            } else {
                args.push(self.parse_expr()?);
            }

            self.match_kind(TokenKind::Comma);
        }
        Ok(RunCall { args, options })
    }

    fn parse_pipe_segment(&mut self) -> ParsResult<Vec<Stmt>> {
        if self.match_kind(TokenKind::LBrace) {
            let mut body = Vec::new();
            while !self.match_kind(TokenKind::RBrace) {
                body.push(self.parse_stmt()?);
            }
            Ok(body)
        } else {
            Ok(vec![self.parse_stmt_atom()?])
        }
    }


    // Parse stdout/stderr redirect target (single or list)
    fn parse_redirect_output_list(&mut self, stream_name: &str) -> ParsResult<Vec<Spanned<RedirectOutputTarget>>> {
        // Check if it's a list
        if self.match_kind(TokenKind::LBracket) {
            let list_start = self.previous_span();
            let mut targets = Vec::new();
            let mut seen_inherit = false;

            if self.match_kind(TokenKind::RBracket) {
                // Empty list
                let list_span = Span::new(list_start.start, self.previous_span().end);
                return self.error("redirect target list cannot be empty", list_span);
            }

            loop {
                let target_start = self.current_span();
                let target = self.parse_redirect_output_target()?;
                let target_end = self.previous_span();
                let target_span = Span::new(target_start.start, target_end.end);

                // Validate per-element constraints
                match &target {
                    // Check for wrong stream inherit
                    RedirectOutputTarget::InheritStdout if stream_name == "stderr" => {
                        return self.error("inherit_stdout() is only valid for stdout redirects", target_span);
                    }
                    RedirectOutputTarget::InheritStderr if stream_name == "stdout" => {
                        return self.error("inherit_stderr() is only valid for stderr redirects", target_span);
                    }
                    // Check for duplicate inherit
                    RedirectOutputTarget::InheritStdout if stream_name == "stdout" => {
                        if seen_inherit {
                            return self.error("duplicate inherit_stdout()", target_span);
                        }
                        seen_inherit = true;
                    }
                    RedirectOutputTarget::InheritStderr if stream_name == "stderr" => {
                        if seen_inherit {
                            return self.error("duplicate inherit_stderr()", target_span);
                        }
                        seen_inherit = true;
                    }
                    // Check for cross-stream in list
                    RedirectOutputTarget::ToStdout | RedirectOutputTarget::ToStderr => {
                        return self.error("cross-stream redirect not allowed in multi-sink list", target_span);
                    }
                    _ => {}
                }

                targets.push(Spanned::new(target, target_span));

                if !self.match_kind(TokenKind::Comma) {
                    break;
                }
                // Allow trailing comma
                if matches!(self.peek_kind(), Some(TokenKind::RBracket)) {
                    break;
                }
            }

            self.expect(TokenKind::RBracket)?;

            Ok(targets)
        } else {
            // Single target (non-list)
            let target_start = self.current_span();
            let target = self.parse_redirect_output_target()?;
            let target_end = self.previous_span();
            let target_span = Span::new(target_start.start, target_end.end);
            
            // Reject inherit_* in non-list form
            match &target {
                RedirectOutputTarget::InheritStdout => {
                    return self.error("inherit_stdout() is only valid in redirect lists", target_span);
                }
                RedirectOutputTarget::InheritStderr => {
                    return self.error("inherit_stderr() is only valid in redirect lists", target_span);
                }
                _ => {}
            }
            
            Ok(vec![Spanned::new(target, target_span)])
        }
    }

    fn parse_redirect_output_target(&mut self) -> ParsResult<RedirectOutputTarget> {
        if self.match_kind(TokenKind::File) {
            self.expect(TokenKind::LParen)?;
            let path = self.parse_expr()?;
            let mut append = false;
            if self.match_kind(TokenKind::Comma) {
                if self.match_kind(TokenKind::Append) {
                    self.expect(TokenKind::Equals)?;
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
                            self.error("append must be bool", self.current_span())?;
                        }
                    } else {
                        self.error("append must be bool", self.current_span())?;
                    }
                }
            }
            self.expect(TokenKind::RParen)?;
            Ok(RedirectOutputTarget::File { path, append })
        } else if let Some(TokenKind::Ident(s)) = self.peek_kind() {
            // Parse function-like forms: to_stdout(), to_stderr(), inherit_stdout(), inherit_stderr()
            let name = s.clone();
            match name.as_str() {
                "to_stdout" => {
                    self.advance();
                    self.expect(TokenKind::LParen)?;
                    self.expect(TokenKind::RParen)?;
                    Ok(RedirectOutputTarget::ToStdout)
                }
                "to_stderr" => {
                    self.advance();
                    self.expect(TokenKind::LParen)?;
                    self.expect(TokenKind::RParen)?;
                    Ok(RedirectOutputTarget::ToStderr)
                }
                "inherit_stdout" => {
                    self.advance();
                    self.expect(TokenKind::LParen)?;
                    self.expect(TokenKind::RParen)?;
                    Ok(RedirectOutputTarget::InheritStdout)
                }
                "inherit_stderr" => {
                    self.advance();
                    self.expect(TokenKind::LParen)?;
                    self.expect(TokenKind::RParen)?;
                    Ok(RedirectOutputTarget::InheritStderr)
                }
                _ => self.error("Expected redirect output target (file, to_stdout, to_stderr, inherit_stdout, inherit_stderr)", self.current_span())?
            }
        } else {
            self.error("Expected redirect output target (file, to_stdout, to_stderr, inherit_stdout, inherit_stderr)", self.current_span())?
        }
    }

    fn parse_redirect_input_target(&mut self) -> ParsResult<RedirectInputTarget> {
        if self.match_kind(TokenKind::File) {
            self.expect(TokenKind::LParen)?;
            let path = self.parse_expr()?;
            // Handle optional comma + parameters (must reject append)
            if self.match_kind(TokenKind::Comma) {
                if self.match_kind(TokenKind::Append) {
                    // Consume the = and value to avoid leaving tokens
                    self.expect(TokenKind::Equals)?;
                    if self.match_kind(TokenKind::True) || self.match_kind(TokenKind::False) {
                        // consumed
                    } else if let Some(TokenKind::Ident(s)) = self.peek_kind() {
                        if s == "true" || s == "false" {
                            self.advance();
                        }
                    }
                    return self.error("cannot append to stdin", self.previous_span());
                } else {
                    return self.error("unexpected parameter for stdin file()", self.current_span());
                }
            }
            self.expect(TokenKind::RParen)?;
            Ok(RedirectInputTarget::File { path })
        } else if let Some(TokenKind::Ident(s)) = self.peek_kind() {
            if s == "heredoc" {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let content = if let Some(TokenKind::String(s)) = self.peek_kind() {
                    s.clone()
                } else {
                    self.error("Expected string", self.current_span())?
                };
                self.advance();
                self.expect(TokenKind::RParen)?;
                Ok(RedirectInputTarget::HereDoc { content })
            } else {
                self.error("Expected redirect input target (file or heredoc)", self.current_span())?
            }
        } else {
            self.error("Expected redirect input target (file or heredoc)", self.current_span())?
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
