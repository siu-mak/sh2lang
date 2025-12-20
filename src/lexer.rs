#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Func,
    Run,
    Print,
    If,
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
                    "if" => tokens.push(Token::If),
                    _ => tokens.push(Token::Ident(ident)),
                }
            }
            _ => panic!("Unexpected char: {}", c),
        }
    }

    tokens
}
