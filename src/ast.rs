#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompareOp {
    Eq,
    NotEq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(String),
    Var(String),
    Command(Vec<Expr>),
    Concat(Box<Expr>, Box<Expr>),
    Compare { left: Box<Expr>, op: CompareOp, right: Box<Expr> },
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    List(Vec<Expr>),
    Args,
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
    WithCwd {
        path: Expr,
        body: Vec<Stmt>,
    },
    Sh(String),
    ShBlock(Vec<String>),
    Call { name: String, args: Vec<Expr> },
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

