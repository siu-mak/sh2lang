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
    Amp,
    Bang,
    Case,
    Minus,
    Star,
    Slash,
    Percent,
    Lt,
    Le,
    Gt,
    Ge,
    Arrow,
    Underscore,
    While,
    For,
    In,
    Args,
    With,
    Env,
    Cd,
    Cwd,
    Sh,
    Break,
    Continue,
    Return,
    Exit,
    Capture,
    Subshell,
    Group,
    Redirect,
    Stdout,
    Stderr,
    Stdin,
    File,
    Append,
    Colon,
    Spawn,
    Wait,
    Try,
    Catch,
    Export,
    Unset,
    Exists,
    IsDir,
    IsFile,
    Len,
    Source,
    Arg,
    Index,
    Join,
    Exec,
    Status,
    Pid,
    Count,
    Uid,
    Ppid,
    Pwd,
    SelfPid,
    Argv0,
    Argc,
    True,
    False,
    Number(u32),
    Ident(String),
    String(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Set,
    PipeKw,
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
            ':' => { tokens.push(Token::Colon); chars.next(); }
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
            '-' => { tokens.push(Token::Minus); chars.next(); }
            '*' => { tokens.push(Token::Star); chars.next(); }
            '/' => { tokens.push(Token::Slash); chars.next(); }
            '%' => { tokens.push(Token::Percent); chars.next(); }
            '<' => {
                 chars.next();
                 if chars.peek() == Some(&'=') {
                     tokens.push(Token::Le);
                     chars.next();
                 } else {
                     tokens.push(Token::Lt);
                 }
            }
            '>' => {
                 chars.next();
                 if chars.peek() == Some(&'=') {
                     tokens.push(Token::Ge);
                     chars.next();
                 } else {
                     tokens.push(Token::Gt);
                 }
            }
            '&' => {
                 chars.next();
                 if chars.peek() == Some(&'&') {
                     tokens.push(Token::AndAnd);
                     chars.next();
                 } else {
                     tokens.push(Token::Amp);
                 }

            }
            '.' => { tokens.push(Token::Dot); chars.next(); }
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
                chars.next(); // consume opening quote
                let mut s = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' { break; }
                    if ch == '\\' {
                        chars.next(); // consume backslash
                        if let Some(&escaped) = chars.peek() {
                            if escaped == '$' {
                                s.push('\\');
                                s.push('$');
                                chars.next();
                            } else {
                                s.push(escaped);
                                chars.next();
                            }
                        }
                    } else {
                        s.push(ch);
                        chars.next();
                    }
                }
                if chars.peek() == Some(&'"') {
                    chars.next(); // consume closing quote
                }
                tokens.push(Token::String(s));
            }
            _ if c.is_ascii_digit() => {
                let mut num_str = String::new();
                while let Some(&ch) = chars.peek() {
                    if !ch.is_ascii_digit() { break; }
                    num_str.push(ch);
                    chars.next();
                }
                let n: u32 = num_str.parse().expect("Invalid number literal");
                tokens.push(Token::Number(n));
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
                    "cd" => tokens.push(Token::Cd),
                    "cwd" => tokens.push(Token::Cwd),
                    "sh" => tokens.push(Token::Sh),
                    "break" => tokens.push(Token::Break),
                    "continue" => tokens.push(Token::Continue),
                    "return" => tokens.push(Token::Return),
                    "exit" => tokens.push(Token::Exit),
                    "capture" => tokens.push(Token::Capture),
                    "subshell" => tokens.push(Token::Subshell),
                    "group" => tokens.push(Token::Group),
                    "redirect" => tokens.push(Token::Redirect),
                    "stdout" => tokens.push(Token::Stdout),
                    "stderr" => tokens.push(Token::Stderr),
                    "stdin" => tokens.push(Token::Stdin),
                    "file" => tokens.push(Token::File),
                    "append" => tokens.push(Token::Append),
                    "spawn" => tokens.push(Token::Spawn),
                    "wait" => tokens.push(Token::Wait),
                    "try" => tokens.push(Token::Try),
                    "catch" => tokens.push(Token::Catch),
                    "export" => tokens.push(Token::Export),
                    "unset" => tokens.push(Token::Unset),
                    "source" => tokens.push(Token::Source),
                    "set" => tokens.push(Token::Set),
                    "exists" => tokens.push(Token::Exists),
                    "arg" => tokens.push(Token::Arg),
                    "index" => tokens.push(Token::Index),
                    "join" => tokens.push(Token::Join),
                    "exec" => tokens.push(Token::Exec),
                    "status" => tokens.push(Token::Status),
                    "pid" => tokens.push(Token::Pid),
                    "count" => tokens.push(Token::Count),
                    "uid" => tokens.push(Token::Uid),
                    "ppid" => tokens.push(Token::Ppid),
                    "pwd" => tokens.push(Token::Pwd),
                    "self_pid" => tokens.push(Token::SelfPid),
                    "argv0" => tokens.push(Token::Argv0),
                    "argc" => tokens.push(Token::Argc),
                    "true" => tokens.push(Token::True),
                    "false" => tokens.push(Token::False),
                    "is_dir" => tokens.push(Token::IsDir),
                    "is_file" => tokens.push(Token::IsFile),
                    "pipe" => tokens.push(Token::PipeKw),
                    "len" => tokens.push(Token::Len),
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
