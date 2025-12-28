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
    Arith { left: Box<Val>, op: ArithOp, right: Box<Val> },
    Compare { left: Box<Val>, op: CompareOp, right: Box<Val> },
    And(Box<Val>, Box<Val>),
    Or(Box<Val>, Box<Val>),
    Not(Box<Val>),
    Exists(Box<Val>),
    IsDir(Box<Val>),
    IsFile(Box<Val>),
    IsSymlink(Box<Val>),
    IsExec(Box<Val>),
    IsReadable(Box<Val>),
    IsWritable(Box<Val>),
    IsNonEmpty(Box<Val>),
    Len(Box<Val>),
    Arg(u32),
    Index { list: Box<Val>, index: Box<Val> },
    Join { list: Box<Val>, sep: Box<Val> },
    Count(Box<Val>),
    Bool(bool),
    Number(u32),
    List(Vec<Val>),
    Args,
    Status,
    Pid,
    Env(Box<Val>),
    EnvDot(String),
    Uid,
    Ppid,
    BoolStr(Box<Val>),
    Pwd,
    SelfPid,
    Argv0,
    Argc,
    Input(Box<Val>),
    Confirm(Box<Val>),
    Matches(Box<Val>, Box<Val>),
    ParseArgs,
    ArgsFlags(Box<Val>),
    ArgsPositionals(Box<Val>),
    Call { name: String, args: Vec<Val> },
    LoadEnvfile(Box<Val>),
    JsonKv(Box<Val>),
    MapLiteral(Vec<(String, Val)>),
    MapIndex { map: String, key: String },
    Which(Box<Val>),
    ReadFile(Box<Val>),
    Home,
    PathJoin(Vec<Val>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompareOp {
    Eq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug)]
pub enum Cmd {
    Assign(String, Val),
    Exec { args: Vec<Val>, allow_fail: bool },
    Print(Val),
    PrintErr(Val),
    If {
        cond: Val,
        then_body: Vec<Cmd>,
        elifs: Vec<(Val, Vec<Cmd>)>,
        else_body: Vec<Cmd>,
    },
    Pipe(Vec<(Vec<Val>, bool)>),
    PipeBlocks(Vec<Vec<Cmd>>),
    Case {
        expr: Val,
        arms: Vec<(Vec<Pattern>, Vec<Cmd>)>,
    },
    For {
        var: String,
        items: Vec<Val>,
        body: Vec<Cmd>,
    },
    ForMap {
        key_var: String,
        val_var: String,
        map: String,
        body: Vec<Cmd>,
    },
    While {
        cond: Val,
        body: Vec<Cmd>,
    },
    Break,
    Continue,
    Return(Option<Val>),
    Require(Vec<Val>),
    Exit(Option<Val>),
    WithEnv {
        bindings: Vec<(String, Val)>,
        body: Vec<Cmd>,
    },
    WithLog {
        path: Val,
        append: bool,
        body: Vec<Cmd>,
    },
    WithCwd {
        path: Val,
        body: Vec<Cmd>,
    },
    WriteFile {
        path: Val,
        content: Val,
        append: bool,
    },
    Log {
        level: LogLevel,
        msg: Val,
        timestamp: bool,
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
    ExecReplace(Vec<Val>),
    SaveEnvfile { path: Val, env: Val },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectTarget {
    File { path: Val, append: bool },
    HereDoc { content: String },
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(String),
    Glob(String),
    Wildcard,
}
