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
}

#[derive(Debug)]
pub enum Cmd {
    Assign(String, Val),
    Exec(Vec<Val>),
    Print(Val),
    PrintErr(Val),
    IfNonEmpty {
        var: String,
        then_body: Vec<Cmd>,
        else_body: Vec<Cmd>,
    },
}
