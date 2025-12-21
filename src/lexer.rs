#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Func,
    Run,
    Print,
    PrintErr,
    If,
    Else,
    Let,
    Plus,
    Equals,
    EqEq,
    NotEq,
    Dollar,
    Ident(String),
    String(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\n' | '\t' => { chars.next(); }
            '(' => { tokens.push(Token::LParen); chars.next(); }
            ')' => { tokens.push(Token::RParen); chars.next(); }
            '{' => { tokens.push(Token::LBrace); chars.next(); }
            '}' => { tokens.push(Token::RBrace); chars.next(); }
            ',' => { tokens.push(Token::Comma); chars.next(); }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                     tokens.push(Token::EqEq);
                     chars.next();
                } else {
                     tokens.push(Token::Equals);
                }
            }
            '+' => { tokens.push(Token::Plus); chars.next(); }
            '$' => { tokens.push(Token::Dollar); chars.next(); }
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    tokens.push(Token::NotEq);
                    chars.next();
                } else {
                    panic!("Unexpected character '!'");
                }
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' { break; }
                    s.push(ch);
                    chars.next();
                }
                chars.next();
                tokens.push(Token::String(s));
            }
            _ if c.is_alphabetic() => {
                let mut ident = String::new();
                while let Some(&ch) = chars.peek() {
                    if !ch.is_alphanumeric() && ch != '_' { break; }
                    ident.push(ch);
                    chars.next();
                }
                match ident.as_str() {
                    "func" => tokens.push(Token::Func),
                    "run" => tokens.push(Token::Run),
                    "print" => tokens.push(Token::Print),
                    "print_err" => tokens.push(Token::PrintErr),
                    "if" => tokens.push(Token::If),
                    "else" => tokens.push(Token::Else),
                    "let" => tokens.push(Token::Let),
                    _ => tokens.push(Token::Ident(ident)),
                }
            }
            _ => panic!("Unexpected char: {}", c),
        }
    }

    tokens
}
