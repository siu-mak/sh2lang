#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub commands: Vec<Cmd>,
}

#[derive(Debug)]
pub enum Val {
    Literal(String),
    Var(String),
}

#[derive(Debug)]
pub enum Cmd {
    Assign(String, String),
    Exec(Vec<Val>),
    Print(Val),
    PrintErr(Val),
    IfNonEmpty {
        var: String,
        then_body: Vec<Cmd>,
        else_body: Vec<Cmd>,
    },
}
