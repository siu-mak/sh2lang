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

            let value = match &tokens[*i] {
                Token::String(s) => s.clone(),
                _ => panic!("Expected string literal in let assignment"),
            };
            *i += 1;

            Stmt::Let { name, value }
        }

        Token::Run => {
            *i += 1;
            expect(tokens, i, Token::LParen);

            let mut args = Vec::new();
            loop {
                match &tokens[*i] {
                    Token::String(s) => {
                        args.push(Expr::Literal(s.clone()));
                        *i += 1;
                        if matches!(tokens[*i], Token::Comma) {
                            *i += 1;
                        } else {
                            break;
                        }
                    }
                    Token::Ident(s) => {
                        args.push(Expr::Var(s.clone()));
                        *i += 1;
                        if matches!(tokens[*i], Token::Comma) {
                            *i += 1;
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }

            expect(tokens, i, Token::RParen);
            Stmt::Run(args)
        }

        Token::Print => {
            *i += 1;
            expect(tokens, i, Token::LParen);

            let expr = match &tokens[*i] {
                Token::String(s) => Expr::Literal(s.clone()),
                Token::Ident(s) => Expr::Var(s.clone()),
                _ => panic!("Expected string or variable in print()"),
            };
            *i += 1;

            expect(tokens, i, Token::RParen);
            Stmt::Print(expr)
        }

        Token::PrintErr => {
            *i += 1;
            expect(tokens, i, Token::LParen);

            let expr = match &tokens[*i] {
                Token::String(s) => Expr::Literal(s.clone()),
                Token::Ident(s) => Expr::Var(s.clone()),
                _ => panic!("Expected string or variable in print_err()"),
            };
            *i += 1;

            expect(tokens, i, Token::RParen);
            Stmt::PrintErr(expr)
        }

        Token::If => {
            *i += 1;

            let var = match &tokens[*i] {
                Token::Ident(s) => s.clone(),
                _ => panic!("Expected variable name after if"),
            };
            *i += 1;

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
                var,
                then_body,
                else_body,
            }

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
