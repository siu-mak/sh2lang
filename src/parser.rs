use crate::lexer::Token;
use crate::ast::*;

pub fn parse(tokens: &[Token]) -> Program {
    let mut i = 0;

    let mut functions = Vec::new();

    while i < tokens.len() {
        expect(tokens, &mut i, Token::Func);

        let name = match &tokens[i] {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected function name"),
        };
        i += 1;

        expect(tokens, &mut i, Token::LParen);
        
        let mut params = Vec::new();
        if !matches!(tokens.get(i), Some(Token::RParen)) {
             loop {
                 let param_name = match &tokens[i] {
                     Token::Ident(s) => s.clone(),
                     _ => panic!("Expected parameter name"),
                 };
                 i += 1;
                 params.push(param_name);
                 
                 if matches!(tokens.get(i), Some(Token::Comma)) {
                     i += 1;
                 } else {
                     break;
                 }
             }
        }

        expect(tokens, &mut i, Token::RParen);
        expect(tokens, &mut i, Token::LBrace);

        let mut body = Vec::new();
        while !matches!(tokens[i], Token::RBrace) {
            body.push(parse_stmt(tokens, &mut i));
        }

        expect(tokens, &mut i, Token::RBrace);

        functions.push(Function { name, params, body });
    }

    Program { functions }
}

fn parse_stmt(tokens: &[Token], i: &mut usize) -> Stmt {
    let mut left = parse_stmt_atom(tokens, i);

    while let Some(token) = tokens.get(*i) {
        match token {
            Token::AndAnd => {
                *i += 1;
                let right = parse_stmt_atom(tokens, i);
                left = Stmt::AndThen {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            }
            Token::OrOr => {
                *i += 1;
                let right = parse_stmt_atom(tokens, i);
                left = Stmt::OrElse {
                    left: Box::new(left),
                    right: Box::new(right),
                };
            }
            _ => break,
        }
    }

    left
}

fn parse_stmt_atom(tokens: &[Token], i: &mut usize) -> Stmt {
    
    match &tokens[*i] {
        Token::Let => {
            *i += 1;
            let name = match &tokens[*i] {
                Token::Ident(s) => s.clone(),
                _ => panic!("Expected variable name after let"),
            };
            *i += 1;
            
            expect(tokens, i, Token::Equals);

            let value = parse_expr(tokens, i);
            Stmt::Let { name, value }
        }

        Token::Run => {
            let mut segments = Vec::new();
            
            // First run(...)
            *i += 1;
            expect(tokens, i, Token::LParen);

            let mut args = Vec::new();
            loop {
                if matches!(tokens.get(*i), Some(Token::RParen)) { break; }
                args.push(parse_expr(tokens, i));
                if matches!(tokens.get(*i), Some(Token::Comma)) { *i += 1; } else { break; }
            }
            expect(tokens, i, Token::RParen);
            segments.push(args);

            // Additional run(...) segments separated by `|`
            while matches!(tokens.get(*i), Some(Token::Pipe)) {
                *i += 1;
                expect(tokens, i, Token::Run);
                expect(tokens, i, Token::LParen);
                
                let mut next_args = Vec::new();
                loop {
                    if matches!(tokens.get(*i), Some(Token::RParen)) { break; }
                    next_args.push(parse_expr(tokens, i));
                    if matches!(tokens.get(*i), Some(Token::Comma)) { *i += 1; } else { break; }
                }
                expect(tokens, i, Token::RParen);
                segments.push(next_args);
            }

            if segments.len() == 1 {
                Stmt::Run(segments.pop().unwrap())
            } else {
                Stmt::Pipe(segments)
            }
        }

        Token::Exec => {
            *i += 1;
            expect(tokens, i, Token::LParen);

            let mut args = Vec::new();
            loop {
                if matches!(tokens.get(*i), Some(Token::RParen)) { break; }
                args.push(parse_expr(tokens, i));
                if matches!(tokens.get(*i), Some(Token::Comma)) { *i += 1; } else { break; }
            }
            expect(tokens, i, Token::RParen);

            if args.is_empty() {
                panic!("exec requires at least one argument (the command)");
            }
            Stmt::Exec(args)
        }

        Token::Print => {
            *i += 1;
            expect(tokens, i, Token::LParen);

            let expr = parse_expr(tokens, i);

            expect(tokens, i, Token::RParen);
            Stmt::Print(expr)
        }

        Token::PrintErr => {
            *i += 1;
            expect(tokens, i, Token::LParen);

            let expr = parse_expr(tokens, i);

            expect(tokens, i, Token::RParen);
            Stmt::PrintErr(expr)
        }

        Token::If => {
            *i += 1;

            let cond = parse_expr(tokens, i);
            
            expect(tokens, i, Token::LBrace);

            let mut then_body = Vec::new();
            while !matches!(tokens[*i], Token::RBrace) {
                then_body.push(parse_stmt(tokens, i));
            }
            expect(tokens, i, Token::RBrace);

            let mut elifs = Vec::new();
            loop {
                if matches!(tokens.get(*i), Some(Token::Elif)) {
                    *i += 1;
                } else if matches!(tokens.get(*i), Some(Token::Else)) && matches!(tokens.get(*i + 1), Some(Token::If)) {
                    *i += 2;
                } else {
                    break;
                }
                
                let cond = parse_expr(tokens, i);
                expect(tokens, i, Token::LBrace);
                let mut body = Vec::new();
                while !matches!(tokens[*i], Token::RBrace) {
                    body.push(parse_stmt(tokens, i));
                }
                expect(tokens, i, Token::RBrace);
                elifs.push(Elif { cond, body });
            }

            // optional else
            let else_body = if matches!(tokens.get(*i), Some(Token::Else)) {
                *i += 1;
                expect(tokens, i, Token::LBrace);

                let mut body = Vec::new();
                while !matches!(tokens[*i], Token::RBrace) {
                    body.push(parse_stmt(tokens, i));
                }
                expect(tokens, i, Token::RBrace);
                Some(body)
            } else {
                None
            };

            Stmt::If {
                cond,
                then_body,
                elifs,
                else_body,
            }

        }

        Token::Case => {
            *i += 1;
            let expr = parse_expr(tokens, i);
            expect(tokens, i, Token::LBrace);
            
            let mut arms = Vec::new();
            while !matches!(tokens[*i], Token::RBrace) {
                // Parse patterns
                let mut patterns = Vec::new();
                loop {
                    match &tokens[*i] {
                        Token::String(s) => {
                            patterns.push(Pattern::Literal(s.clone()));
                            *i += 1;
                        }
                        Token::Underscore => {
                            patterns.push(Pattern::Wildcard);
                            *i += 1;
                        }
                        _ => panic!("Expected string or _ pattern"),
                    }
                    
                    if matches!(tokens.get(*i), Some(Token::Pipe)) {
                        *i += 1;
                    } else {
                        break;
                    }
                }
                
                expect(tokens, i, Token::Arrow);
                expect(tokens, i, Token::LBrace);
                
                let mut body = Vec::new();
                while !matches!(tokens[*i], Token::RBrace) {
                    body.push(parse_stmt(tokens, i));
                }
                expect(tokens, i, Token::RBrace);
                
                arms.push(CaseArm { patterns, body });
            }
            expect(tokens, i, Token::RBrace);
            
            Stmt::Case { expr, arms }
        }

        Token::While => {
            *i += 1;
            let cond = parse_expr(tokens, i);
            expect(tokens, i, Token::LBrace);
            
            let mut body = Vec::new();
            while !matches!(tokens[*i], Token::RBrace) {
                body.push(parse_stmt(tokens, i));
            }
            expect(tokens, i, Token::RBrace);
            
            Stmt::While { cond, body }
        }

        Token::For => {
            *i += 1;
            let var = match &tokens[*i] {
                Token::Ident(s) => s.clone(),
                _ => panic!("Expected identifier for loop variable"),
            };
            *i += 1;
            
            expect(tokens, i, Token::In);

            let items = if matches!(tokens.get(*i), Some(Token::LParen)) {
                // Legacy: (e1, e2, ...)
                *i += 1;
                let mut items = Vec::new();
                loop {
                    if matches!(tokens.get(*i), Some(Token::RParen)) { break; }
                    items.push(parse_expr(tokens, i));
                    if matches!(tokens.get(*i), Some(Token::Comma)) { *i += 1; } else { break; }
                }
                expect(tokens, i, Token::RParen);
                items
            } else {
                // New: single expression (e.g., list literal)
                vec![parse_expr(tokens, i)]
            };

            expect(tokens, i, Token::LBrace);
            
            let mut body = Vec::new();
            while !matches!(tokens[*i], Token::RBrace) {
                body.push(parse_stmt(tokens, i));
            }
            expect(tokens, i, Token::RBrace);
            
            Stmt::For { var, items, body }
        }

        Token::Break => {
            *i += 1;
            Stmt::Break
        }

        Token::Continue => {
            *i += 1;
            Stmt::Continue
        }

        Token::Return => {
            *i += 1;
            let val = if is_expr_start(tokens.get(*i)) {
                Some(parse_expr(tokens, i))
            } else {
                None
            };
            Stmt::Return(val)
        }

        Token::Exit => {
            *i += 1;
            let code = if is_expr_start(tokens.get(*i)) {
                Some(parse_expr(tokens, i))
            } else {
                None
            };
            Stmt::Exit(code)
        }

        Token::With => {
            *i += 1;
            
            if matches!(tokens.get(*i), Some(Token::Env)) {
                *i += 1;
                
                expect(tokens, i, Token::LBrace);
                let mut bindings = Vec::new();
                while !matches!(tokens[*i], Token::RBrace) {
                     let name = match &tokens[*i] {
                         Token::Ident(s) => s.clone(),
                         _ => panic!("Expected identifier for env binding"),
                     };
                     *i += 1;
                     expect(tokens, i, Token::Equals);
                     let val = parse_expr(tokens, i);
                     bindings.push((name, val));
                }
                expect(tokens, i, Token::RBrace);

                expect(tokens, i, Token::LBrace);
                let mut body = Vec::new();
                while !matches!(tokens[*i], Token::RBrace) {
                     body.push(parse_stmt(tokens, i));
                }
                expect(tokens, i, Token::RBrace);

                Stmt::WithEnv { bindings, body }

            } else if matches!(tokens.get(*i), Some(Token::Cwd)) {
                *i += 1;
                
                let path = parse_expr(tokens, i);
                
                expect(tokens, i, Token::LBrace);
                let mut body = Vec::new();
                while !matches!(tokens[*i], Token::RBrace) {
                     body.push(parse_stmt(tokens, i));
                }
                expect(tokens, i, Token::RBrace);
                
                Stmt::WithCwd { path, body }
                
            } else if matches!(tokens.get(*i), Some(Token::Redirect)) {
                *i += 1;
                expect(tokens, i, Token::LBrace);
                
                let mut stdin = None;
                let mut stdout = None;
                let mut stderr = None;

                while !matches!(tokens.get(*i), Some(Token::RBrace)) {
                    let next_tok = tokens.get(*i);
                    *i += 1;

                    match next_tok {
                         Some(Token::Stdout) => {
                             expect(tokens, i, Token::Colon);
                             stdout = Some(parse_redirect_target(tokens, i));
                         }
                         Some(Token::Stderr) => {
                             expect(tokens, i, Token::Colon);
                             stderr = Some(parse_redirect_target(tokens, i));
                         }
                         Some(Token::Stdin) => {
                             expect(tokens, i, Token::Colon);
                             let target = parse_redirect_target(tokens, i);
                                match target {
                                    RedirectTarget::File { append, .. } => {
                                        if append {
                                            panic!("Cannot append to stdin (use 'file' without append for input)");
                                        }
                                    }
                                    _ => panic!("stdin can only be redirected from a file currently"),
                                }
                             stdin = Some(target);
                         }
                         _ => panic!("Expected stdout, stderr, or stdin"),
                    }

                    if matches!(tokens.get(*i), Some(Token::Comma)) {
                        *i += 1;
                    } else {
                        break;
                    }
                }
                expect(tokens, i, Token::RBrace);

                expect(tokens, i, Token::LBrace);
                let mut body = Vec::new();
                while !matches!(tokens[*i], Token::RBrace) {
                    body.push(parse_stmt(tokens, i));
                }
                expect(tokens, i, Token::RBrace);

                Stmt::WithRedirect { stdout, stderr, stdin, body }

            } else {
                panic!("Expected 'env', 'cwd', or 'redirect' after 'with'");
            }
        }

        Token::Spawn => {
            *i += 1;
            let stmt = parse_stmt(tokens, i);
            Stmt::Spawn { stmt: Box::new(stmt) }
        }

        Token::Wait => {
            *i += 1;
            expect(tokens, i, Token::LParen);
            
            let expr = if is_expr_start(tokens.get(*i)) {
              Some(parse_expr(tokens, i))
            } else {
              None
            };

            expect(tokens, i, Token::RParen);
            Stmt::Wait(expr)
        }

        Token::Try => {
             *i += 1;
             expect(tokens, i, Token::LBrace);
             let mut try_body = Vec::new();
             while !matches!(tokens[*i], Token::RBrace) {
                 try_body.push(parse_stmt(tokens, i));
             }
             expect(tokens, i, Token::RBrace);
             
             expect(tokens, i, Token::Catch);
             expect(tokens, i, Token::LBrace);
             let mut catch_body = Vec::new();
             while !matches!(tokens[*i], Token::RBrace) {
                 catch_body.push(parse_stmt(tokens, i));
             }
             expect(tokens, i, Token::RBrace);
             
             Stmt::TryCatch { try_body, catch_body }
        }

        Token::Cd => {
            *i += 1;
            expect(tokens, i, Token::LParen);
            let path = parse_expr(tokens, i);
            expect(tokens, i, Token::RParen);
            Stmt::Cd { path }
        }

        Token::Export => {
            *i += 1;
            expect(tokens, i, Token::LParen);
            let name = match &tokens[*i] {
                Token::String(s) => s.clone(),
                _ => panic!("Expected string literal for export name"),
            };
            *i += 1;
            
            let value = if matches!(tokens.get(*i), Some(Token::Comma)) {
                *i += 1;
                Some(parse_expr(tokens, i))
            } else {
                None
            };
            
            expect(tokens, i, Token::RParen);
            Stmt::Export { name, value }
        }

        Token::Unset => {
            *i += 1;
            expect(tokens, i, Token::LParen);
            let name = match &tokens[*i] {
                Token::String(s) => s.clone(),
                _ => panic!("Expected string literal for unset name"),
            };
            *i += 1;
            expect(tokens, i, Token::RParen);
            Stmt::Unset { name }
        }

        Token::Source => {
            *i += 1;
            expect(tokens, i, Token::LParen);
            let path = parse_expr(tokens, i);
            expect(tokens, i, Token::RParen);
            Stmt::Source { path }
        }

        Token::Subshell => {
            *i += 1;
            expect(tokens, i, Token::LBrace);
            let mut body = Vec::new();
            while !matches!(tokens[*i], Token::RBrace) {
                body.push(parse_stmt(tokens, i));
            }
            expect(tokens, i, Token::RBrace);
            Stmt::Subshell { body }
        }

        Token::Group => {
            *i += 1;
            expect(tokens, i, Token::LBrace);
            let mut body = Vec::new();
            while !matches!(tokens[*i], Token::RBrace) {
                body.push(parse_stmt(tokens, i));
            }
            expect(tokens, i, Token::RBrace);
            Stmt::Group { body }
        }

        Token::Sh => {
            *i += 1;
            if matches!(tokens.get(*i), Some(Token::LParen)) {
                expect(tokens, i, Token::LParen);
                let s = match &tokens[*i] {
                    Token::String(s) => { *i += 1; s.clone() }
                    _ => panic!("Expected string literal in sh(...)"),
                };
                expect(tokens, i, Token::RParen);
                Stmt::Sh(s)
            } else if matches!(tokens.get(*i), Some(Token::LBrace)) {
                expect(tokens, i, Token::LBrace);
                let mut lines = Vec::new();
                loop {
                    if matches!(tokens.get(*i), Some(Token::RBrace)) { break; }
                    match &tokens[*i] {
                        Token::String(s) => {
                             lines.push(s.clone());
                             *i += 1;
                        },
                        _ => panic!("Expected string literal in sh {{ ... }}"),
                    }
                    if matches!(tokens.get(*i), Some(Token::Comma)) {
                        *i += 1;
                    } else {
                        // If no comma, expect end of block
                        if !matches!(tokens.get(*i), Some(Token::RBrace)) {
                            panic!("Expected comma or closing brace in sh {{ ... }}");
                        }
                    }
                }
                expect(tokens, i, Token::RBrace);
                Stmt::ShBlock(lines)
            } else {
                panic!("Expected parenthesis or brace after 'sh'");
            }
        }

        Token::Ident(name) => {
            *i += 1;
            expect(tokens, i, Token::LParen);
            let mut args = Vec::new();
            loop {
                if matches!(tokens.get(*i), Some(Token::RParen)) { break; }
                args.push(parse_expr(tokens, i));
                if matches!(tokens.get(*i), Some(Token::Comma)) { *i += 1; } else { break; }
            }
            expect(tokens, i, Token::RParen);
            Stmt::Call { name: name.clone(), args }
        }

        
        _ => panic!("Expected statement"),
    }
}

fn expect(tokens: &[Token], i: &mut usize, t: Token) {
    if tokens.get(*i) != Some(&t) {
        panic!("Expected {:?}, got {:?}", t, tokens.get(*i));
    }
    *i += 1;
}

fn is_expr_start(t: Option<&Token>) -> bool {
    matches!(t,
        Some(Token::String(_)
           | Token::Ident(_)
           | Token::Dollar
           | Token::LParen
           | Token::LBracket
           | Token::Args
           | Token::Capture
           | Token::Exists
           | Token::IsDir
           | Token::IsFile
           | Token::Len
           | Token::Arg
           | Token::Index
           | Token::Join
           | Token::Count
           | Token::True
           | Token::False
           | Token::Number(_)
        )
    )
}

fn parse_expr(tokens: &[Token], i: &mut usize) -> Expr {
    parse_or(tokens, i)
}

fn parse_or(tokens: &[Token], i: &mut usize) -> Expr {
    let mut left = parse_and(tokens, i);

    while let Some(Token::OrOr) = tokens.get(*i) {
        *i += 1;
        let right = parse_and(tokens, i);
        left = Expr::Or(Box::new(left), Box::new(right));
    }

    left
}

fn parse_and(tokens: &[Token], i: &mut usize) -> Expr {
    let mut left = parse_comparison(tokens, i);

    while let Some(Token::AndAnd) = tokens.get(*i) {
        *i += 1;
        let right = parse_comparison(tokens, i);
        left = Expr::And(Box::new(left), Box::new(right));
    }

    left
}

fn parse_comparison(tokens: &[Token], i: &mut usize) -> Expr {
    let left = parse_sum(tokens, i);

    if let Some(token) = tokens.get(*i) {
        let op = match token {
            Token::EqEq => Some(CompareOp::Eq),
            Token::NotEq => Some(CompareOp::NotEq),
            Token::Lt => Some(CompareOp::Lt),
            Token::Le => Some(CompareOp::Le),
            Token::Gt => Some(CompareOp::Gt),
            Token::Ge => Some(CompareOp::Ge),
            _ => None,
        };

        if let Some(op) = op {
            *i += 1;
            let right = parse_sum(tokens, i);
            return Expr::Compare {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
    }

    left
}

fn parse_sum(tokens: &[Token], i: &mut usize) -> Expr {
    let mut left = parse_term(tokens, i);

    loop {
        match tokens.get(*i) {
            Some(Token::Plus) => {
                *i += 1;
                let right = parse_term(tokens, i);
                left = Expr::Arith { left: Box::new(left), op: ArithOp::Add, right: Box::new(right) };
            }
            Some(Token::Minus) => {
                *i += 1;
                let right = parse_term(tokens, i);
                left = Expr::Arith { left: Box::new(left), op: ArithOp::Sub, right: Box::new(right) };
            }
            _ => break,
        }
    }

    left
}

fn parse_term(tokens: &[Token], i: &mut usize) -> Expr {
    let mut left = parse_unary(tokens, i);

    loop {
        match tokens.get(*i) {
            Some(Token::Star) => {
                *i += 1;
                let right = parse_unary(tokens, i);
                left = Expr::Arith { left: Box::new(left), op: ArithOp::Mul, right: Box::new(right) };
            }
            Some(Token::Slash) => {
                *i += 1;
                let right = parse_unary(tokens, i);
                left = Expr::Arith { left: Box::new(left), op: ArithOp::Div, right: Box::new(right) };
            }
            Some(Token::Percent) => {
                *i += 1;
                let right = parse_unary(tokens, i);
                left = Expr::Arith { left: Box::new(left), op: ArithOp::Mod, right: Box::new(right) };
            }
            _ => break,
        }
    }

    left
}

fn parse_unary(tokens: &[Token], i: &mut usize) -> Expr {
    if let Some(Token::Bang) = tokens.get(*i) {
        *i += 1;
        let expr = parse_unary(tokens, i);
        Expr::Not(Box::new(expr))
    } else if let Some(Token::Minus) = tokens.get(*i) {
        // Desugar -x to 0 - x
        *i += 1;
        let right = parse_unary(tokens, i);
        Expr::Arith {
            left: Box::new(Expr::Number(0)),
            op: ArithOp::Sub,
            right: Box::new(right),
        }
    } else {
        parse_primary(tokens, i)
    }
}

fn parse_primary(tokens: &[Token], i: &mut usize) -> Expr {
    match &tokens[*i] {
        Token::LParen => {
            *i += 1;
            let e = parse_expr(tokens, i);
            expect(tokens, i, Token::RParen);
            e
        }
        Token::String(s) => {
            *i += 1;
            Expr::Literal(s.clone())
        }
        Token::Ident(s) => {
            *i += 1;
            Expr::Var(s.clone())
        }
        Token::Dollar => {
            *i += 1;
            parse_command_substitution(tokens, i)
        }
        Token::LBracket => {
            *i += 1;
            let mut exprs = Vec::new();
            loop {
                if matches!(tokens.get(*i), Some(Token::RBracket)) { break; }
                exprs.push(parse_expr(tokens, i));
                if matches!(tokens.get(*i), Some(Token::Comma)) { *i += 1; } else { break; }
            }
            expect(tokens, i, Token::RBracket);
            Expr::List(exprs)
        }
        Token::Args => {
            *i += 1;
            Expr::Args
        }
        Token::Capture => {
            *i += 1;
            parse_command_substitution(tokens, i)
        }
        Token::Exists => {
            *i += 1;
            expect(tokens, i, Token::LParen);
            let path = parse_expr(tokens, i);
            expect(tokens, i, Token::RParen);
            Expr::Exists(Box::new(path))
        }
        Token::IsDir => {
            *i += 1;
            expect(tokens, i, Token::LParen);
            let path = parse_expr(tokens, i);
            expect(tokens, i, Token::RParen);
            Expr::IsDir(Box::new(path))
        }
        Token::IsFile => {
            *i += 1;
            expect(tokens, i, Token::LParen);
            let path = parse_expr(tokens, i);
            expect(tokens, i, Token::RParen);
            Expr::IsFile(Box::new(path))
        }
        Token::Len => {
            *i += 1;
            expect(tokens, i, Token::LParen);
            let expr = parse_expr(tokens, i);
            expect(tokens, i, Token::RParen);
            Expr::Len(Box::new(expr))
        }
        Token::Arg => {
            *i += 1;
            expect(tokens, i, Token::LParen);
            let n = match tokens[*i] {
                Token::Number(v) => v,
                 _ => panic!("Expected number in arg(n)"),
            };
            *i += 1;
            expect(tokens, i, Token::RParen);
            Expr::Arg(n)
        }
        Token::Index => {
            *i += 1;
            expect(tokens, i, Token::LParen);
            let list = parse_expr(tokens, i);
            expect(tokens, i, Token::Comma);
            let index = parse_expr(tokens, i);
            expect(tokens, i, Token::RParen);
            Expr::Index { list: Box::new(list), index: Box::new(index) }
        }
        Token::Join => {
            *i += 1;
            expect(tokens, i, Token::LParen);
            let list = parse_expr(tokens, i);
            expect(tokens, i, Token::Comma);
            let sep = parse_expr(tokens, i);
            expect(tokens, i, Token::RParen);
            Expr::Join { list: Box::new(list), sep: Box::new(sep) }
        }
        Token::Count => {
            *i += 1;
            expect(tokens, i, Token::LParen);
            let inner = parse_expr(tokens, i);
            expect(tokens, i, Token::RParen);
            Expr::Count(Box::new(inner))
        }
        Token::True => {
            *i += 1;
            Expr::Bool(true)
        }
        Token::False => {
            *i += 1;
            Expr::Bool(false)
        }
        Token::Number(n) => {
            *i += 1;
            Expr::Number(*n)
        }
        _ => panic!("Expected string or variable, got {:?}", tokens.get(*i)),
    }
}
fn parse_command_substitution(tokens: &[Token], i: &mut usize) -> Expr {
    // Expect '(' after '$' or 'capture'
    expect(tokens, i, Token::LParen);
    let mut segments: Vec<Vec<Expr>> = Vec::new();
    loop {
        let mut args: Vec<Expr> = Vec::new();
        // Parse a command segment: either run(...), a function call (ident), or a string literal command name
        if matches!(tokens.get(*i), Some(Token::Run)) {
            *i += 1;
            expect(tokens, i, Token::LParen);
            // parse arguments inside run()
            loop {
                if matches!(tokens.get(*i), Some(Token::RParen)) { break; }
                args.push(parse_expr(tokens, i));
                if matches!(tokens.get(*i), Some(Token::Comma)) { *i += 1; } else { break; }
            }
            expect(tokens, i, Token::RParen);
        } else if let Some(Token::Ident(name)) = tokens.get(*i).cloned() {
            *i += 1;
            // function name becomes first argument (command name)
            args.push(Expr::Literal(name));
            expect(tokens, i, Token::LParen);
            loop {
                if matches!(tokens.get(*i), Some(Token::RParen)) { break; }
                args.push(parse_expr(tokens, i));
                if matches!(tokens.get(*i), Some(Token::Comma)) { *i += 1; } else { break; }
            }
            expect(tokens, i, Token::RParen);
        } else if let Some(Token::String(s)) = tokens.get(*i).cloned() {
            // string literal command name (e.g., capture("printf", ...))
            *i += 1;
            args.push(Expr::Literal(s));
            // parse remaining arguments until closing parenthesis, handling commas
            loop {
                if matches!(tokens.get(*i), Some(Token::RParen)) { break; }
                if matches!(tokens.get(*i), Some(Token::Comma)) { *i += 1; }
                args.push(parse_expr(tokens, i));
            }
        } else {
            panic!("Expected run(...), function call, or string literal command name inside command substitution");
        }
        segments.push(args);
        // If there is a pipe, continue parsing next segment
        if matches!(tokens.get(*i), Some(Token::Pipe)) {
            *i += 1;
        } else {
            break;
        }
    }
    expect(tokens, i, Token::RParen);
    if segments.len() == 1 {
        Expr::Command(segments.pop().unwrap())
    } else {
        Expr::CommandPipe(segments)
    }
}


fn parse_redirect_target(tokens: &[Token], i: &mut usize) -> crate::ast::RedirectTarget {
    if matches!(tokens.get(*i), Some(Token::File)) {
        *i += 1;
        expect(tokens, i, Token::LParen);
        let path = parse_expr(tokens, i);
        let mut append = false;
        
        if matches!(tokens.get(*i), Some(Token::Comma)) {
            *i += 1;
            if matches!(tokens.get(*i), Some(Token::Append)) {
                *i += 1;
                expect(tokens, i, Token::Equals);
                 // primitive true/false parsing
                 match tokens.get(*i) {
                     Some(Token::True) => { append = true; *i += 1; }
                     Some(Token::False) => { append = false; *i += 1; }
                     Some(Token::Ident(val)) if val == "true" => { append = true; *i += 1; }
                     Some(Token::Ident(val)) if val == "false" => { append = false; *i += 1; }
                     _ => panic!("Expected true/false for append"),
                 }
            }
        }

        expect(tokens, i, Token::RParen);
        crate::ast::RedirectTarget::File { path, append }
    } else if matches!(tokens.get(*i), Some(Token::Stdout)) {
        *i += 1;
        crate::ast::RedirectTarget::Stdout
    } else if matches!(tokens.get(*i), Some(Token::Stderr)) {
        *i += 1;
        crate::ast::RedirectTarget::Stderr
    } else {
        panic!("Expected file(...), stdout, or stderr as redirect target");
    }
}
