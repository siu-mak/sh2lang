#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub commands: Vec<Cmd>,
    pub file: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Literal(String),
    Var(String),
    Command(Vec<Val>),
    CommandPipe(Vec<Vec<Val>>),
    Concat(Box<Val>, Box<Val>),
    Arith {
        left: Box<Val>,
        op: ArithOp,
        right: Box<Val>,
    },
    Compare {
        left: Box<Val>,
        op: CompareOp,
        right: Box<Val>,
    },
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
    ArgDynamic(Box<Val>),
    Index {
        list: Box<Val>,
        index: Box<Val>,
    },
    Join {
        list: Box<Val>,
        sep: Box<Val>,
    },
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
    Confirm { prompt: Box<Val>, default: bool },
    Matches(Box<Val>, Box<Val>),
    ParseArgs,
    ArgsFlags(Box<Val>),
    ArgsPositionals(Box<Val>),
    Call {
        name: String,
        args: Vec<Val>,
    },
    LoadEnvfile(Box<Val>),
    JsonKv(Box<Val>),
    MapLiteral(Vec<(String, Val)>),
    MapIndex {
        map: String,
        key: String,
    },
    Which(Box<Val>),
    ReadFile(Box<Val>),
    TryRun(Vec<Val>),
    Home,
    PathJoin(Vec<Val>),
    Lines(Box<Val>),

    ContainsList {
        list: Box<Val>,
        needle: Box<Val>,
    },
    ContainsSubstring {
        haystack: Box<Val>,
        needle: Box<Val>,
    },
    ContainsLine {
        file: Box<Val>,
        needle: Box<Val>,
    },
    StartsWith { text: Box<Val>, prefix: Box<Val> },
    Split { s: Box<Val>, delim: Box<Val> },
    /// A variable known to hold a boolean value ("1" or "0").
    /// Used in conditions to emit `[ "$var" = "1" ]` instead of non-empty check.
    BoolVar(String),
    Capture {
        value: Box<Val>,
        allow_fail: bool,
    },
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

#[derive(Debug, Clone, PartialEq)]
pub enum ForIterable {
    List(Vec<Val>),
    Range(Val, Val),
}

#[derive(Debug)]
pub enum Cmd {
    Assign(String, Val, Option<String>),
    Exec {
        args: Vec<Val>,
        allow_fail: bool,
        loc: Option<String>,
    },
    Print(Val),
    PrintErr(Val),
    If {
        cond: Val,
        then_body: Vec<Cmd>,
        elifs: Vec<(Val, Vec<Cmd>)>,
        else_body: Vec<Cmd>,
    },
    Pipe(Vec<(Vec<Val>, bool)>, Option<String>),
    PipeBlocks(Vec<Vec<Cmd>>, Option<String>),
    Case {
        expr: Val,
        arms: Vec<(Vec<Pattern>, Vec<Cmd>)>,
    },
    For {
        var: String,
        iterable: ForIterable,
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
    Raw(Val, Option<String>),
    RawLine { line: String, loc: Option<String> },
    Call {
        name: String,
        args: Vec<Val>,
    },
    Subshell {
        body: Vec<Cmd>,
    },
    Group {
        body: Vec<Cmd>,
    },
    WithRedirect {
        stdout: Option<Vec<RedirectOutputTarget>>,
        stderr: Option<Vec<RedirectOutputTarget>>,
        stdin: Option<RedirectInputTarget>,
        body: Vec<Cmd>,
    },
    Spawn(Box<Cmd>),
    Wait(Option<Val>),
    TryCatch {
        try_body: Vec<Cmd>,
        catch_body: Vec<Cmd>,
    },
    AndThen {
        left: Vec<Cmd>,
        right: Vec<Cmd>,
    },
    OrElse {
        left: Vec<Cmd>,
        right: Vec<Cmd>,
    },
    Export {
        name: String,
        value: Option<Val>,
    },
    Unset(String),
    Source(Val),
    ExecReplace(Vec<Val>, Option<String>),
    SaveEnvfile {
        path: Val,
        env: Val,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectOutputTarget {
    File { path: Val, append: bool },
    ToStdout,         // cross-stream: stderr → stdout
    ToStderr,         // cross-stream: stdout → stderr
    InheritStdout,    // keep stdout visible to terminal
    InheritStderr,    // keep stderr visible to terminal
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectInputTarget {
    File { path: Val },
    HereDoc { content: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(String),
    Glob(String),
    Wildcard,
}

impl Function {
    pub fn strip_spans(&mut self) {
        self.file.clear();
        for cmd in &mut self.commands {
            cmd.strip_spans();
        }
    }
}

impl Cmd {
    pub fn strip_spans(&mut self) {
        match self {
            Cmd::Assign(_, _, loc) => *loc = None,
            Cmd::Exec { loc, .. } => *loc = None,
            Cmd::Pipe(_, loc) => *loc = None,
            Cmd::PipeBlocks(blocks, loc) => {
                *loc = None;
                for block in blocks {
                    for cmd in block { cmd.strip_spans(); }
                }
            }
            Cmd::ExecReplace(_, loc) => *loc = None,
            
            // Recursive cases
            Cmd::If { then_body, elifs, else_body, .. } => {
                for c in then_body { c.strip_spans(); }
                for (_, body) in elifs { for c in body { c.strip_spans(); } }
                for c in else_body { c.strip_spans(); }
            }
            Cmd::While { body, .. } => for c in body { c.strip_spans(); },
            Cmd::For { body, .. } => for c in body { c.strip_spans(); },
            Cmd::ForMap { body, .. } => for c in body { c.strip_spans(); },
            Cmd::Case { arms, .. } => for (_, body) in arms { for c in body { c.strip_spans(); } },
            Cmd::WithEnv { body, .. } => for c in body { c.strip_spans(); },
            Cmd::WithLog { body, .. } => for c in body { c.strip_spans(); },
            Cmd::WithCwd { body, .. } => for c in body { c.strip_spans(); },
            Cmd::Subshell { body } => for c in body { c.strip_spans(); },
            Cmd::Group { body } => for c in body { c.strip_spans(); },
            Cmd::WithRedirect { body, .. } => for c in body { c.strip_spans(); },
            Cmd::Spawn(cmd) => cmd.strip_spans(),
            Cmd::TryCatch { try_body, catch_body } => {
                for c in try_body { c.strip_spans(); }
                for c in catch_body { c.strip_spans(); }
            }
            Cmd::AndThen { left, right } => {
                for c in left { c.strip_spans(); }
                for c in right { c.strip_spans(); }
            }
            Cmd::OrElse { left, right } => {
                for c in left { c.strip_spans(); }
                for c in right { c.strip_spans(); }
            }
            
            // No spans/recursion needed
            Cmd::Print(_) => {},
            Cmd::PrintErr(_) => {},
            Cmd::Break => {},
            Cmd::Continue => {},
            Cmd::Return(_) => {},
            Cmd::Require(_) => {},
            Cmd::Exit(_) => {},
            Cmd::WriteFile { .. } => {},
            Cmd::Log { .. } => {},
            Cmd::Cd(_) => {},
            Cmd::Raw(..) => {},
            Cmd::RawLine { loc, .. } => *loc = None,
            Cmd::Call { .. } => {},
            Cmd::Wait(_) => {},
            Cmd::Export { .. } => {},
            Cmd::Unset(_) => {},
            Cmd::Source(_) => {},
            Cmd::SaveEnvfile { .. } => {},

        }
    }
}
