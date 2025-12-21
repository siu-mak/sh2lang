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
        else_body: Option<Vec<Stmt>>,
    },
    Pipe(Vec<Vec<Expr>>),
}

