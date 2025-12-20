#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub commands: Vec<Cmd>,
}

#[derive(Debug)]
pub enum Cmd {
    Exec(Vec<String>),
    Print(String),
}
