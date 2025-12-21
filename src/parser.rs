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
    let mut left = parse_primary(tokens, i);

    while let Some(Token::Plus) = tokens.get(*i) {
        *i += 1;
        let right = parse_primary(tokens, i);
        left = Expr::Concat(Box::new(left), Box::new(right));
    }

    left
}

fn parse_primary(tokens: &[Token], i: &mut usize) -> Expr {
    match &tokens[*i] {
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
            expect(tokens, i, Token::Run);
            expect(tokens, i, Token::LParen);
            
            let mut args = Vec::new();
            loop {
                if matches!(tokens.get(*i), Some(Token::RParen)) {
                   break; 
                }
                
                args.push(parse_expr(tokens, i));

                if matches!(tokens.get(*i), Some(Token::Comma)) {
                    *i += 1;
                } else {
                    break;
                }
            }

            expect(tokens, i, Token::RParen);
            expect(tokens, i, Token::RParen);
            Expr::Command(args)
        }
        _ => panic!("Expected string or variable, got {:?}", tokens.get(*i)),
    }
}
