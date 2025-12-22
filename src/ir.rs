#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub commands: Vec<Cmd>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Literal(String),
    Var(String),
    Command(Vec<Val>),
    CommandPipe(Vec<Vec<Val>>),
    Concat(Box<Val>, Box<Val>),
    Compare { left: Box<Val>, op: CompareOp, right: Box<Val> },
    And(Box<Val>, Box<Val>),
    Or(Box<Val>, Box<Val>),
    Not(Box<Val>),
    Exists(Box<Val>),
    IsDir(Box<Val>),
    IsFile(Box<Val>),
    Len(Box<Val>),
    Arg(u32),
    Index { list: Box<Val>, index: u32 },
    Number(u32),
    List(Vec<Val>),
    Args,
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
        elifs: Vec<(Val, Vec<Cmd>)>,
        else_body: Vec<Cmd>,
    },
    Pipe(Vec<Vec<Val>>),
    Case {
        expr: Val,
        arms: Vec<(Vec<Pattern>, Vec<Cmd>)>,
    },
    For {
        var: String,
        items: Vec<Val>,
        body: Vec<Cmd>,
    },
    While {
        cond: Val,
        body: Vec<Cmd>,
    },
    Break,
    Continue,
    Return(Option<Val>),
    Exit(Option<Val>),
    WithEnv {
        bindings: Vec<(String, Val)>,
        body: Vec<Cmd>,
    },
    WithCwd {
        path: Val,
        body: Vec<Cmd>,
    },
    Cd(Val),
    Raw(String),
    Call { name: String, args: Vec<Val> },
    Subshell { body: Vec<Cmd> },
    Group { body: Vec<Cmd> },
    WithRedirect {
        stdout: Option<RedirectTarget>,
        stderr: Option<RedirectTarget>,
        stdin: Option<RedirectTarget>,
        body: Vec<Cmd>,
    },
    Spawn(Box<Cmd>),
    Wait(Option<Val>),
    TryCatch {
        try_body: Vec<Cmd>,
        catch_body: Vec<Cmd>,
    },
    AndThen { left: Vec<Cmd>, right: Vec<Cmd> },
    OrElse { left: Vec<Cmd>, right: Vec<Cmd> },
    Export { name: String, value: Option<Val> },
    Unset(String),
    Source(Val),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectTarget {
    File { path: Val, append: bool },
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(String),
    Wildcard,
}
