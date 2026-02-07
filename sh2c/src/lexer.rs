use crate::span::{Diagnostic, SourceMap, Span};

// ... existing enum TokenKind ...
// ... existing struct Token ...

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    pos: usize,
    sm: &'a SourceMap,
    file: &'a str,
}

impl<'a> Lexer<'a> {
    fn new(sm: &'a SourceMap, file: &'a str) -> Self {
        Lexer {
            chars: sm.src().chars().peekable(),
            pos: 0,
            sm,
            file,
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.chars.next();
        if let Some(ch) = c {
            self.pos += ch.len_utf8();
        }
        c
    }

    fn error<T>(&self, msg: &str, start: usize) -> Result<T, Diagnostic> {
        let span = Span::new(start, self.pos);
        Err(Diagnostic {
            msg: msg.to_string(),
            span,
            sm: Some(self.sm.clone()),
            file: Some(self.file.to_string()),
        })
    }
}

pub fn lex(sm: &SourceMap, file: &str) -> Result<Vec<Token>, Diagnostic> {
    let mut tokens = Vec::new();
    let mut lexer = Lexer::new(sm, file);

    while let Some(&c) = lexer.peek() {
        let start = lexer.pos;
        match c {
            ' ' | '\n' | '\t' => {
                lexer.next();
            }
            // ... simple tokens same as before ...
            '#' => {
                while let Some(&c) = lexer.peek() {
                    if c == '\n' {
                        break;
                    }
                    lexer.next();
                }
            }
            '(' => { lexer.next(); tokens.push(Token { kind: TokenKind::LParen, span: Span::new(start, lexer.pos) }); }
            ')' => { lexer.next(); tokens.push(Token { kind: TokenKind::RParen, span: Span::new(start, lexer.pos) }); }
            '{' => { lexer.next(); tokens.push(Token { kind: TokenKind::LBrace, span: Span::new(start, lexer.pos) }); }
            '}' => { lexer.next(); tokens.push(Token { kind: TokenKind::RBrace, span: Span::new(start, lexer.pos) }); }
            '[' => { lexer.next(); tokens.push(Token { kind: TokenKind::LBracket, span: Span::new(start, lexer.pos) }); }
            ']' => { lexer.next(); tokens.push(Token { kind: TokenKind::RBracket, span: Span::new(start, lexer.pos) }); }
            ',' => { lexer.next(); tokens.push(Token { kind: TokenKind::Comma, span: Span::new(start, lexer.pos) }); }
            ':' => { lexer.next(); tokens.push(Token { kind: TokenKind::Colon, span: Span::new(start, lexer.pos) }); }
            '=' => {
                lexer.next();
                if lexer.peek() == Some(&'=') {
                    lexer.next();
                    tokens.push(Token { kind: TokenKind::EqEq, span: Span::new(start, lexer.pos) });
                } else if lexer.peek() == Some(&'>') {
                    lexer.next();
                    tokens.push(Token { kind: TokenKind::Arrow, span: Span::new(start, lexer.pos) });
                } else {
                    tokens.push(Token { kind: TokenKind::Equals, span: Span::new(start, lexer.pos) });
                }
            }
            '_' => { lexer.next(); tokens.push(Token { kind: TokenKind::Underscore, span: Span::new(start, lexer.pos) }); }
            '+' => { lexer.next(); tokens.push(Token { kind: TokenKind::Plus, span: Span::new(start, lexer.pos) }); }
            '-' => {
                lexer.next();
                tokens.push(Token { kind: TokenKind::Minus, span: Span::new(start, lexer.pos) });
            }
            '*' => { lexer.next(); tokens.push(Token { kind: TokenKind::Star, span: Span::new(start, lexer.pos) }); }
            '/' => {
                lexer.next();
                if lexer.peek() == Some(&'/') {
                    // Comment
                    while let Some(&c) = lexer.peek() {
                        if c == '\n' {
                            break;
                        }
                        lexer.next();
                    }
                } else {
                     tokens.push(Token { kind: TokenKind::Slash, span: Span::new(start, lexer.pos) });
                }
            }
            '%' => { lexer.next(); tokens.push(Token { kind: TokenKind::Percent, span: Span::new(start, lexer.pos) }); }
            '<' => {
                lexer.next();
                if lexer.peek() == Some(&'=') {
                    lexer.next();
                    tokens.push(Token { kind: TokenKind::Le, span: Span::new(start, lexer.pos) });
                } else {
                    tokens.push(Token { kind: TokenKind::Lt, span: Span::new(start, lexer.pos) });
                }
            }
            '>' => {
                lexer.next();
                if lexer.peek() == Some(&'=') {
                    lexer.next();
                    tokens.push(Token { kind: TokenKind::Ge, span: Span::new(start, lexer.pos) });
                } else {
                    tokens.push(Token { kind: TokenKind::Gt, span: Span::new(start, lexer.pos) });
                }
            }
            '&' => {
                lexer.next();
                if lexer.peek() == Some(&'&') {
                    lexer.next();
                    tokens.push(Token { kind: TokenKind::AndAnd, span: Span::new(start, lexer.pos) });
                } else {
                    tokens.push(Token { kind: TokenKind::Amp, span: Span::new(start, lexer.pos) });
                }
            }
            '.' => {
                lexer.next();
                if let Some('.') = lexer.peek() {
                     lexer.next();
                     tokens.push(Token {
                         kind: TokenKind::DotDot,
                         span: Span::new(start, lexer.pos),
                     });
                } else {
                     tokens.push(Token {
                         kind: TokenKind::Dot,
                         span: Span::new(start, lexer.pos),
                     });
                }
            }
            '|' => {
                lexer.next();
                if lexer.peek() == Some(&'|') {
                    lexer.next();
                    tokens.push(Token { kind: TokenKind::OrOr, span: Span::new(start, lexer.pos) });
                } else {
                    tokens.push(Token { kind: TokenKind::Pipe, span: Span::new(start, lexer.pos) });
                }
            }
            '$' => { lexer.next(); tokens.push(Token { kind: TokenKind::Dollar, span: Span::new(start, lexer.pos) }); }
            '!' => {
                lexer.next();
                 if lexer.peek() == Some(&'=') {
                    lexer.next();
                    tokens.push(Token { kind: TokenKind::NotEq, span: Span::new(start, lexer.pos) });
                } else {
                    tokens.push(Token { kind: TokenKind::Bang, span: Span::new(start, lexer.pos) });
                }
            }
            '"' => {
                // Check for triple quote start """
                let mut is_triple = false;
                {
                    // Lookahead
                    let mut la = lexer.chars.clone();
                    la.next(); // skip current "
                    if la.next() == Some('"') && la.next() == Some('"') {
                        is_triple = true;
                    }
                }

                if is_triple {
                    lexer.next(); // "
                    lexer.next(); // "
                    lexer.next(); // "

                    let mut s = String::new();
                    loop {
                        // Check for triple quote end """
                        {
                            let mut la = lexer.chars.clone();
                            if la.next() == Some('"')
                                && la.next() == Some('"')
                                && la.next() == Some('"')
                            {
                                lexer.next();
                                lexer.next();
                                lexer.next();
                                break;
                            }
                        }

                        if let Some(ch) = lexer.next() {
                            if ch == '\\' {
                                if let Some(escaped) = lexer.next() {
                                    match escaped {
                                        'n' => s.push('\n'),
                                        't' => s.push('\t'),
                                        'r' => s.push('\r'),
                                        '\\' => s.push('\\'),
                                        '"' => s.push('"'),
                                        '$' => {
                                            s.push('\\');
                                            s.push('$');
                                        }
                                        _ => s.push(escaped),
                                    }
                                } else {
                                    return lexer.error("Unexpected EOF in string escape", start);
                                }
                            } else {
                                s.push(ch);
                            }
                        } else {
                             return lexer.error("Unterminated triple-quoted string", start);
                        }
                    }
                    tokens.push(Token {
                        kind: TokenKind::String(s),
                        span: Span::new(start, lexer.pos),
                    });
                } else {
                    // Regular string
                    lexer.next(); // consume opening quote
                    let mut s = String::new();
                    while let Some(&ch) = lexer.peek() {
                        if ch == '"' {
                            break;
                        }
                         if ch == '\\' {
                            lexer.next(); // consume backslash
                            if let Some(&escaped) = lexer.peek() {
                                match escaped {
                                    'n' => { s.push('\n'); lexer.next(); }
                                    't' => { s.push('\t'); lexer.next(); }
                                    'r' => { s.push('\r'); lexer.next(); }
                                    '\\' => { s.push('\\'); lexer.next(); }
                                    '"' => { s.push('"'); lexer.next(); }
                                    '$' => { s.push('\\'); s.push('$'); lexer.next(); }
                                    _ => { s.push(escaped); lexer.next(); }
                                }
                            }
                        } else {
                            s.push(ch);
                            lexer.next();
                        }
                    }
                    if lexer.peek() == Some(&'"') {
                        lexer.next(); // consume closing quote
                    } else {
                        // EOF before quote
                        return lexer.error("Unterminated string (missing closing quote)", start);
                    }
                    tokens.push(Token {
                        kind: TokenKind::String(s),
                        span: Span::new(start, lexer.pos),
                    });
                }
            }
            _ if c.is_ascii_digit() => {
                let mut num_str = String::new();
                while let Some(&ch) = lexer.peek() {
                    if !ch.is_ascii_digit() {
                        break;
                    }
                    num_str.push(ch);
                    lexer.next();
                }
                let n: u32 = num_str.parse().expect("Invalid number literal");
                tokens.push(Token {
                    kind: TokenKind::Number(n),
                    span: Span::new(start, lexer.pos),
                });
            }
            'r' => {
                lexer.next(); // consume 'r'
                if lexer.peek() == Some(&'"') {
                    // Raw string
                    let mut is_triple = false;
                    {
                        // Lookahead for triple "
                        let mut la = lexer.chars.clone();
                        la.next(); // skip current "
                        if la.next() == Some('"') && la.next() == Some('"') {
                            is_triple = true;
                        }
                    }

                    if is_triple {
                        lexer.next(); // "
                        lexer.next(); // "
                        lexer.next(); // "

                        let mut s = String::new();
                        loop {
                            // Check for triple quote end """
                            {
                                let mut la = lexer.chars.clone();
                                if la.next() == Some('"')
                                    && la.next() == Some('"')
                                    && la.next() == Some('"')
                                {
                                    lexer.next();
                                    lexer.next();
                                    lexer.next();
                                    break;
                                }
                            }
                            if let Some(ch) = lexer.next() {
                                s.push(ch);
                            } else {
                                return lexer.error("Unterminated raw triple-quoted string", start);
                            }
                        }
                        tokens.push(Token {
                            kind: TokenKind::String(s),
                            span: Span::new(start, lexer.pos),
                        });
                    } else {
                        // Regular raw string
                        lexer.next(); // "
                        let mut s = String::new();
                        loop {
                           if let Some(ch) = lexer.peek().cloned() {
                               if ch == '"' {
                                   lexer.next();
                                   break;
                               }
                               // Allow escaping quote? Python raw string keeps backslash.
                               // r\"\\\"\" -> string is "\\""
                               if ch == '\\' {
                                   lexer.next();
                                   if lexer.peek() == Some(&'\"') {
                                       s.push('\\');
                                       s.push('\"');
                                       lexer.next();
                                       continue;
                                   }
                                   s.push('\\');
                               } else {
                                   s.push(ch);
                                   lexer.next();
                               }
                           } else {
                               return lexer.error("Unterminated raw string", start);
                           }
                        }
                        tokens.push(Token {
                            kind: TokenKind::String(s),
                            span: Span::new(start, lexer.pos),
                        });
                    }
                } else {
                    // Identifier starting with r
                    let mut ident = String::from("r");
                    while let Some(&ch) = lexer.peek() {
                        if !ch.is_ascii_alphanumeric() && ch != '_' {
                            break;
                        }
                        ident.push(ch);
                        lexer.next();
                    }
                    let kind = match ident.as_str() {
                         "run" => TokenKind::Run,
                         "return" => TokenKind::Return,
                         "redirect" => TokenKind::Redirect,
                         _ => TokenKind::Ident(ident),
                    };
                    tokens.push(Token { kind, span: Span::new(start, lexer.pos) });
                }
            }
            _ if c.is_ascii_alphabetic() || c == '_' => {

                let mut ident = String::new();
                while let Some(&ch) = lexer.peek() {
                    if !ch.is_ascii_alphanumeric() && ch != '_' {
                        break;
                    }
                    ident.push(ch);
                    lexer.next();
                }
                
                let kind = match ident.as_str() {
                    "func" => TokenKind::Func,
                    "run" => TokenKind::Run,
                    "print" => TokenKind::Print,
                    "print_err" => TokenKind::PrintErr,
                    "if" => TokenKind::If,
                    "elif" => TokenKind::Elif,
                    "else" => TokenKind::Else,
                    "let" => TokenKind::Let,
                    "case" => TokenKind::Case,
                    "while" => TokenKind::While,
                    "for" => TokenKind::For,
                    "in" => TokenKind::In,
                    "args" => TokenKind::Args,
                    "with" => TokenKind::With,
                    "env" => TokenKind::Env,
                    "cd" => TokenKind::Cd,
                    "cwd" => TokenKind::Cwd,
                    "sh" => TokenKind::Sh,
                    "break" => TokenKind::Break,
                    "continue" => TokenKind::Continue,
                    "return" => TokenKind::Return,
                    "exit" => TokenKind::Exit,
                    "capture" => TokenKind::Capture,
                    "subshell" => TokenKind::Subshell,
                    "group" => TokenKind::Group,
                    "redirect" => TokenKind::Redirect,
                    "stdout" => TokenKind::Stdout,
                    "stderr" => TokenKind::Stderr,
                    "stdin" => TokenKind::Stdin,
                    "file" => TokenKind::File,
                    "append" => TokenKind::Append,
                    "spawn" => TokenKind::Spawn,
                    "wait" => TokenKind::Wait,
                    "try" => TokenKind::Try,
                    "catch" => TokenKind::Catch,
                    "export" => TokenKind::Export,
                    "unset" => TokenKind::Unset,
                    "exists" => TokenKind::Exists,
                    "is_dir" => TokenKind::IsDir,
                    "is_file" => TokenKind::IsFile,
                    "is_symlink" => TokenKind::IsSymlink,
                    "is_exec" => TokenKind::IsExec,
                    "is_readable" => TokenKind::IsReadable,
                    "is_writable" => TokenKind::IsWritable,
                    "is_non_empty" => TokenKind::IsNonEmpty,
                    "bool_str" => TokenKind::BoolStr,
                    "len" => TokenKind::Len,
                    "source" => TokenKind::Source,
                    "arg" => TokenKind::Arg,
                    "index" => TokenKind::Index,
                    "join" => TokenKind::Join,
                    "exec" => TokenKind::Exec,
                    "status" => TokenKind::Status,
                    "pid" => TokenKind::Pid,
                    "count" => TokenKind::Count,
                    "uid" => TokenKind::Uid,
                    "ppid" => TokenKind::Ppid,
                    "pwd" => TokenKind::Pwd,
                    "self_pid" => TokenKind::SelfPid,
                    "argv0" => TokenKind::Argv0,
                    "argc" => TokenKind::Argc,
                    "true" => TokenKind::True,
                    "false" => TokenKind::False,
                    "set" => TokenKind::Set,
                    "pipe" => TokenKind::PipeKw,
                    "log" => TokenKind::Log,
                    "import" => TokenKind::Import,
                    "input" => TokenKind::Input,
                    "confirm" => TokenKind::Confirm,
                    _ => TokenKind::Ident(ident),
                };
                tokens.push(Token {
                    kind,
                    span: Span::new(start, lexer.pos),
                });
            }
            ';' => {
                lexer.next();
                tokens.push(Token { kind: TokenKind::Semi, span: Span::new(start, lexer.pos) });
            }
            _ => { 
                return lexer.error(&format!("Unexpected character: {}", c), start);
            }
        }
    }
    // ...
    Ok(tokens)
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
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
    IsSymlink,
    IsExec,
    IsReadable,
    IsWritable,
    IsNonEmpty,
    BoolStr,
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
    DotDot,
    Set,
    PipeKw,
    Log,
    Import,
    Input,
    Confirm,
    Semi,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

