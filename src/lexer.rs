#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Func,
    Run,
    Print,
    PrintErr,
    If,
    Elif,
    Else,
    Let,
    Plus,
    Equals,
    EqEq,
    NotEq,
    Dollar,
    Pipe,
    OrOr,
    AndAnd,
    Bang,
    Case,
    Arrow,
    Underscore,
    While,
    For,
    In,
    Args,
    With,
    Env,
    Cwd,
    Sh,
    Break,
    Continue,
    Return,
    Exit,
    Capture,
    Ident(String),
    String(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
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
            '[' => { tokens.push(Token::LBracket); chars.next(); }
            ']' => { tokens.push(Token::RBracket); chars.next(); }
            ',' => { tokens.push(Token::Comma); chars.next(); }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                     tokens.push(Token::EqEq);
                     chars.next();
                } else if chars.peek() == Some(&'>') {
                     tokens.push(Token::Arrow);
                     chars.next();
                } else {
                     tokens.push(Token::Equals);
                }
            }
            '_' => { tokens.push(Token::Underscore); chars.next(); }
            '+' => { tokens.push(Token::Plus); chars.next(); }
            '&' => {
                 chars.next();
                 if chars.peek() == Some(&'&') {
                     tokens.push(Token::AndAnd);
                     chars.next();
                 } else {
                     panic!("Unexpected character '&'");
                 }
            }
            '|' => {
                 chars.next();
                 if chars.peek() == Some(&'|') {
                     tokens.push(Token::OrOr);
                     chars.next();
                 } else {
                     tokens.push(Token::Pipe);
                 }
            }
            '$' => { tokens.push(Token::Dollar); chars.next(); }
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    tokens.push(Token::NotEq);
                    chars.next();
                } else {
                    tokens.push(Token::Bang);
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
                    "elif" => tokens.push(Token::Elif),
                    "else" => tokens.push(Token::Else),
                    "let" => tokens.push(Token::Let),
                    "case" => tokens.push(Token::Case),
                    "while" => tokens.push(Token::While),
                    "for" => tokens.push(Token::For),
                    "in" => tokens.push(Token::In),
                    "args" => tokens.push(Token::Args),
                    "with" => tokens.push(Token::With),
                    "env" => tokens.push(Token::Env),
                    "cwd" => tokens.push(Token::Cwd),
                    "sh" => tokens.push(Token::Sh),
                    "break" => tokens.push(Token::Break),
                    "continue" => tokens.push(Token::Continue),
                    "return" => tokens.push(Token::Return),
                    "exit" => tokens.push(Token::Exit),
                    "capture" => tokens.push(Token::Capture),
                    _ => tokens.push(Token::Ident(ident)),
                }
            }
            '#' => {
                 while let Some(&ch) = chars.peek() {
                     if ch == '\n' { break; }
                     chars.next();
                 }
            }
            _ => panic!("Unexpected char: {}", c),
        }
    }

    tokens
}
