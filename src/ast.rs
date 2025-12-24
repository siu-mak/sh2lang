#[derive(Debug, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompareOp {
    Eq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LValue {
    Var(String),
    Env(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(String),
    Var(String),
    Command(Vec<Expr>),
    CommandPipe(Vec<Vec<Expr>>),
    Concat(Box<Expr>, Box<Expr>),
    Arith { left: Box<Expr>, op: ArithOp, right: Box<Expr> },
    Compare { left: Box<Expr>, op: CompareOp, right: Box<Expr> },
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Exists(Box<Expr>),
    IsDir(Box<Expr>),
    IsFile(Box<Expr>),
    Len(Box<Expr>),
    Arg(u32),
    Index { list: Box<Expr>, index: Box<Expr> },
    Join { list: Box<Expr>, sep: Box<Expr> },
    Count(Box<Expr>),
    Bool(bool),
    Number(u32),
    List(Vec<Expr>),
    Args,
    Status,
    Pid,
    Env(Box<Expr>),
    Uid,
    Ppid,
    Pwd,
    SelfPid,
    Argv0,
    Argc,
    EnvDot(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let { name: String, value: Expr },
    Run(Vec<Expr>),
    Print(Expr),
    PrintErr(Expr),
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        elifs: Vec<Elif>,
        else_body: Option<Vec<Stmt>>,
    },
    Pipe(Vec<Vec<Expr>>),
    Case {
        expr: Expr,
        arms: Vec<CaseArm>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    For {
        var: String,
        items: Vec<Expr>,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    Return(Option<Expr>),
    Exit(Option<Expr>),
    WithEnv {
        bindings: Vec<(String, Expr)>,
        body: Vec<Stmt>,
    },
    AndThen { left: Vec<Stmt>, right: Vec<Stmt> },
    OrElse { left: Vec<Stmt>, right: Vec<Stmt> },
    WithCwd {
        path: Expr,
        body: Vec<Stmt>,
    },
    Cd { path: Expr },
    Sh(String),
    ShBlock(Vec<String>),
    Call { name: String, args: Vec<Expr> },
    Subshell { body: Vec<Stmt> },
    Group { body: Vec<Stmt> },
    WithRedirect {
        stdout: Option<RedirectTarget>,
        stderr: Option<RedirectTarget>,
        stdin: Option<RedirectTarget>,
        body: Vec<Stmt>,
    },
    Spawn { stmt: Box<Stmt> },
    Wait(Option<Expr>),
    TryCatch {
        try_body: Vec<Stmt>,
        catch_body: Vec<Stmt>,
    },
    Export { name: String, value: Option<Expr> },
    Unset { name: String },
    Source { path: Expr },
    Exec(Vec<Expr>),
    Set { target: LValue, value: Expr },
    PipeBlocks { segments: Vec<Vec<Stmt>> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectTarget {
    File { path: Expr, append: bool },
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseArm {
    pub patterns: Vec<Pattern>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(String),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Elif {
    pub cond: Expr,
    pub body: Vec<Stmt>,
}

