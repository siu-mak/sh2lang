use crate::lexer::Token;
use crate::ast::*;

pub fn parse(tokens: &[Token]) -> Program {
    let mut i = 0;

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
        match &tokens[i] {
            Token::Run => {
                i += 1;
                expect(tokens, &mut i, Token::LParen);

                let mut args = Vec::new();
                loop {
                    match &tokens[i] {
                        Token::String(s) => {
                            args.push(s.clone());
                            i += 1;
                            if matches!(tokens[i], Token::Comma) {
                                i += 1;
                            } else {
                                break;
                            }
                        }
                        _ => break,
                    }
                }

                expect(tokens, &mut i, Token::RParen);
                body.push(Stmt::Run(args));
            }

            Token::Print => {
                i += 1;
                expect(tokens, &mut i, Token::LParen);

                let msg = match &tokens[i] {
                    Token::String(s) => s.clone(),
                    _ => panic!("Expected string literal in print()"),
                };
                i += 1;

                expect(tokens, &mut i, Token::RParen);
                body.push(Stmt::Print(msg));
            }

            _ => panic!("Expected statement"),
        }
    }


    expect(tokens, &mut i, Token::RBrace);

    Program {
        functions: vec![
            Function { name, body }
        ],
    }
}

fn expect(tokens: &[Token], i: &mut usize, t: Token) {
    if tokens.get(*i) != Some(&t) {
        panic!("Expected {:?}, got {:?}", t, tokens.get(*i));
    }
    *i += 1;
}
