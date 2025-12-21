#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub commands: Vec<Cmd>,
}

#[derive(Debug)]
pub enum Val {
    Literal(String),
    Var(String),
    Concat(Box<Val>, Box<Val>),
    Compare { left: Box<Val>, op: CompareOp, right: Box<Val> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompareOp {
    Eq,
    NotEq,
}

#[derive(Debug)]
pub enum Cmd {
    Assign(String, Val),
    Exec(Vec<Val>),
    Print(Val),
    PrintErr(Val),
    If {
        cond: Val,
        then_body: Vec<Cmd>,
        else_body: Vec<Cmd>,
    },
}
