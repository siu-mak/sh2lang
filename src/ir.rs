#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub commands: Vec<Cmd>,
}

#[derive(Debug)]
pub enum Val {
    Literal(String),
    Var(String),
    Command(Vec<Val>),
    Concat(Box<Val>, Box<Val>),
    Compare { left: Box<Val>, op: CompareOp, right: Box<Val> },
    And(Box<Val>, Box<Val>),
    Or(Box<Val>, Box<Val>),
    Not(Box<Val>),
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
    Raw(String),
    Call { name: String, args: Vec<Val> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(String),
    Wildcard,
}
