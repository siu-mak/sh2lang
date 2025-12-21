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
        expect(tokens, &mut i, Token::RParen);
        expect(tokens, &mut i, Token::LBrace);

        let mut body = Vec::new();
        while !matches!(tokens[i], Token::RBrace) {
            body.push(parse_stmt(tokens, &mut i));
        }

        expect(tokens, &mut i, Token::RBrace);

        functions.push(Function { name, body });
    }

    Program { functions }
}

fn parse_stmt(tokens: &[Token], i: &mut usize) -> Stmt {
    
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
            let expr = if matches!(tokens.get(*i), Some(Token::String(_) | Token::Ident(_) | Token::Dollar | Token::LParen)) {
                Some(parse_expr(tokens, i))
            } else {
                None
            };
            Stmt::Return(expr)
        }

        Token::Exit => {
            *i += 1;
            let expr = if matches!(tokens.get(*i), Some(Token::String(_) | Token::Ident(_) | Token::Dollar | Token::LParen)) {
                Some(parse_expr(tokens, i))
            } else {
                None
            };
            Stmt::Exit(expr)
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
                
            } else {
                panic!("Expected 'env' or 'cwd' after 'with'");
            }
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
    let left = parse_concat(tokens, i);

    if let Some(token) = tokens.get(*i) {
        let op = match token {
            Token::EqEq => Some(CompareOp::Eq),
            Token::NotEq => Some(CompareOp::NotEq),
            _ => None,
        };

        if let Some(op) = op {
            *i += 1;
            let right = parse_concat(tokens, i);
            return Expr::Compare {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
    }

    left
}

fn parse_concat(tokens: &[Token], i: &mut usize) -> Expr {
    let mut left = parse_unary(tokens, i);

    while let Some(Token::Plus) = tokens.get(*i) {
        *i += 1;
        let right = parse_unary(tokens, i);
        left = Expr::Concat(Box::new(left), Box::new(right));
    }

    left
}

fn parse_unary(tokens: &[Token], i: &mut usize) -> Expr {
    if let Some(Token::Bang) = tokens.get(*i) {
        *i += 1;
        let expr = parse_unary(tokens, i);
        Expr::Not(Box::new(expr))
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
            expect(tokens, i, Token::LParen);

            let mut args = Vec::new();

            if matches!(tokens.get(*i), Some(Token::Run)) {
                *i += 1;
                expect(tokens, i, Token::LParen);
                // Parse arguments for run(...) as before
                loop {
                    if matches!(tokens.get(*i), Some(Token::RParen)) { break; }
                    args.push(parse_expr(tokens, i));
                    if matches!(tokens.get(*i), Some(Token::Comma)) { *i += 1; } else { break; }
                }
                expect(tokens, i, Token::RParen);

            } else if let Some(Token::Ident(name)) = tokens.get(*i).cloned() {
                *i += 1;
                // Treat function name as first arg (command name)
                args.push(Expr::Literal(name));
                
                expect(tokens, i, Token::LParen);
                loop {
                    if matches!(tokens.get(*i), Some(Token::RParen)) { break; }
                    args.push(parse_expr(tokens, i));
                    if matches!(tokens.get(*i), Some(Token::Comma)) { *i += 1; } else { break; }
                }
                expect(tokens, i, Token::RParen);

            } else {
                panic!("Expected run(...) or function call inside $(...)");
            }

            expect(tokens, i, Token::RParen);
            Expr::Command(args)
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
        _ => panic!("Expected string or variable, got {:?}", tokens.get(*i)),
    }
}
