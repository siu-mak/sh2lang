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
        body: Vec<Stmt>,
    },
}
