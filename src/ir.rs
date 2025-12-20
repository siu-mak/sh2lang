#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub commands: Vec<Cmd>,
}

#[derive(Debug)]
pub enum Cmd {
    Exec(Vec<String>),
    Print(String),
    IfNonEmpty {
        var: String,
        body: Vec<Cmd>,
    },    
}
