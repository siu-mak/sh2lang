#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub commands: Vec<Cmd>,
}

#[derive(Debug)]
pub enum Cmd {
    Exec(Vec<String>),
    Print(String),
    PrintErr(String),
    IfNonEmpty {
        var: String,
        then_body: Vec<Cmd>,
        else_body: Vec<Cmd>,
    },    
}
