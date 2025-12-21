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
pub enum Expr {
    Literal(String),
    Var(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let { name: String, value: String },
    Run(Vec<Expr>),
    Print(Expr),
    PrintErr(Expr),
    If {
        var: String,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
}

