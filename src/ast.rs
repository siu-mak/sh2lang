#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    Run(Vec<String>),
    Print(String),
    If {
        var: String,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
}

