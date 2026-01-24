use crate::span::Span;
use crate::span::SourceMap;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }
}

pub type Expr = Spanned<ExprKind>;
pub type Stmt = Spanned<StmtKind>;

#[derive(Debug, PartialEq)]
pub struct Program {
    pub imports: Vec<String>,
    pub functions: Vec<Function>,

    pub span: Span,
    pub source_maps: HashMap<String, SourceMap>,
    pub entry_file: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub span: Span,
    pub file: String,
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
pub enum LValue {
    Var(String),
    Env(String),
}



#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Literal(String),
    Var(String),
    Command(Vec<Expr>),
    CommandPipe(Vec<Vec<Expr>>),
    Concat(Box<Expr>, Box<Expr>),
    Arith {
        left: Box<Expr>,
        op: ArithOp,
        right: Box<Expr>,
    },
    Compare {
        left: Box<Expr>,
        op: CompareOp,
        right: Box<Expr>,
    },
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Exists(Box<Expr>),
    IsDir(Box<Expr>),
    IsFile(Box<Expr>),
    IsSymlink(Box<Expr>),
    IsExec(Box<Expr>),
    IsReadable(Box<Expr>),
    IsWritable(Box<Expr>),
    IsNonEmpty(Box<Expr>),
    BoolStr(Box<Expr>),
    Len(Box<Expr>),
    Arg(Box<Expr>),
    Index {
        list: Box<Expr>,
        index: Box<Expr>,
    },
    Field {
        base: Box<Expr>,
        name: String,
    },
    Join {
        list: Box<Expr>,
        sep: Box<Expr>,
    },
    Count(Box<Expr>),
    Bool(bool),
    Number(u32),
    List(Vec<Expr>),
    Args,
    Status,
    Pid,
    Env(Box<Expr>),
    Uid,
    Ppid,
    Pwd,
    SelfPid,
    Argv0,
    Argc,
    EnvDot(String),
    Input(Box<Expr>),
    Confirm(Box<Expr>),
    Call {
        name: String,
        args: Vec<Expr>,
    },
    MapLiteral(Vec<(String, Expr)>),
    MapIndex {
        map: String,
        key: String,
    },
    Capture {
        expr: Box<Expr>,
        options: Vec<RunOption>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunOption {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunCall {
    pub args: Vec<Expr>,
    pub options: Vec<RunOption>,
}



#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Let {
        name: String,
        value: Expr,
    },
    Run(RunCall),
    Print(Expr),
    PrintErr(Expr),
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        elifs: Vec<Elif>,
        else_body: Option<Vec<Stmt>>,
    },
    Pipe(Vec<RunCall>),
    Case {
        expr: Expr,
        arms: Vec<CaseArm>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    For {
        var: String,
        items: Vec<Expr>,
        body: Vec<Stmt>,
    },
    ForMap {
        key_var: String,
        val_var: String,
        map: String,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    Return(Option<Expr>),
    Exit(Option<Expr>),
    WithLog {
        path: Expr,
        append: bool,
        body: Vec<Stmt>,
    },
    WithEnv {
        bindings: Vec<(String, Expr)>,
        body: Vec<Stmt>,
    },
    AndThen {
        left: Vec<Stmt>,
        right: Vec<Stmt>,
    },
    OrElse {
        left: Vec<Stmt>,
        right: Vec<Stmt>,
    },
    WithCwd {
        path: Expr,
        body: Vec<Stmt>,
    },
    Cd {
        path: Expr,
    },
    Sh(Expr),
    ShBlock(Vec<String>),
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Subshell {
        body: Vec<Stmt>,
    },
    Group {
        body: Vec<Stmt>,
    },
    WithRedirect {
        stdout: Option<RedirectTarget>,
        stderr: Option<RedirectTarget>,
        stdin: Option<RedirectTarget>,
        body: Vec<Stmt>,
    },
    Spawn {
        stmt: Box<Stmt>,
    },
    Wait(Option<Expr>),
    TryCatch {
        try_body: Vec<Stmt>,
        catch_body: Vec<Stmt>,
    },
    Export {
        name: String,
        value: Option<Expr>,
    },
    Unset {
        name: String,
    },
    Source {
        path: Expr,
    },
    Exec(Vec<Expr>),
    Set {
        target: LValue,
        value: Expr,
    },
    PipeBlocks {
        segments: Vec<Vec<Stmt>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectTarget {
    File { path: Expr, append: bool },
    HereDoc { content: String },
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseArm {
    pub patterns: Vec<Pattern>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(String),
    Glob(String),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Elif {
    pub cond: Expr,
    pub body: Vec<Stmt>,
}

impl Program {
    pub fn strip_spans(&mut self) {
        self.span = Span::new(0, 0);
        self.source_maps.clear();
        self.entry_file.clear(); // Clear for deterministic snapshot
        for f in &mut self.functions {
            f.strip_spans();
        }

    }
}

impl Function {
    pub fn strip_spans(&mut self) {
        self.span = Span::new(0, 0);
        self.file.clear(); // Clear for deterministic snapshot
        for s in &mut self.body {
            s.strip_spans();
        }
    }
}

impl Stmt {
    pub fn strip_spans(&mut self) {
        self.span = Span::new(0, 0);
        self.node.strip_spans();
    }
}

impl StmtKind {
    pub fn strip_spans(&mut self) {
        match self {
            StmtKind::Let { value, .. } => value.strip_spans(),
            StmtKind::Run(call) => call.strip_spans(),
            StmtKind::Exec(args) => for a in args { a.strip_spans(); },
            StmtKind::Print(e) => e.strip_spans(),
            StmtKind::PrintErr(e) => e.strip_spans(),
            StmtKind::If { cond, then_body, elifs, else_body } => {
                cond.strip_spans();
                for s in then_body { s.strip_spans(); }
                for e in elifs {
                    e.cond.strip_spans();
                    for s in &mut e.body { s.strip_spans(); }
                }
                if let Some(body) = else_body {
                    for s in body { s.strip_spans(); }
                }
            }
            StmtKind::While { cond, body } => {
                cond.strip_spans();
                for s in body { s.strip_spans(); }
            }
            StmtKind::For { items, body, .. } => {
                for e in items { e.strip_spans(); }
                for s in body { s.strip_spans(); }
            }
            StmtKind::ForMap { body, .. } => {
                for s in body { s.strip_spans(); }
            }
            StmtKind::TryCatch { try_body, catch_body } => {
                for s in try_body { s.strip_spans(); }
                for s in catch_body { s.strip_spans(); }
            }
            StmtKind::Pipe(segments) => {
                for c in segments { c.strip_spans(); }
            }
            StmtKind::PipeBlocks { segments } => {
                for seg in segments { for s in seg { s.strip_spans(); } }
            }
            StmtKind::Return(Some(e)) => e.strip_spans(),
            StmtKind::Exit(Some(e)) => e.strip_spans(),
            StmtKind::Cd { path } => path.strip_spans(),
            StmtKind::Export { value: Some(v), .. } => v.strip_spans(),
            StmtKind::Source { path } => path.strip_spans(),
            StmtKind::Call { args, .. } => for a in args { a.strip_spans(); },
            StmtKind::AndThen { left, right } => {
                for s in left { s.strip_spans(); }
                for s in right { s.strip_spans(); }
            }
            StmtKind::OrElse { left, right } => {
                for s in left { s.strip_spans(); }
                for s in right { s.strip_spans(); }
            }
            StmtKind::WithEnv { bindings, body } => {
                 for (_, v) in bindings { v.strip_spans(); }
                 for s in body { s.strip_spans(); }
            }
            StmtKind::WithCwd { path, body } => {
                 path.strip_spans();
                 for s in body { s.strip_spans(); }
            }
            StmtKind::WithLog { path, body, .. } => {
                 path.strip_spans();
                 for s in body { s.strip_spans(); }
            }
            StmtKind::WithRedirect { stdout, stderr, stdin, body } => {
                if let Some(t) = stdout { t.strip_spans(); }
                if let Some(t) = stderr { t.strip_spans(); }
                if let Some(t) = stdin { t.strip_spans(); }
                for s in body { s.strip_spans(); }
            }
            StmtKind::Subshell { body } => {
                 for s in body { s.strip_spans(); }
            }
            StmtKind::Group { body } => {
                 for s in body { s.strip_spans(); }
            }
            StmtKind::Spawn { stmt } => stmt.strip_spans(),
            StmtKind::Wait(Some(e)) => e.strip_spans(),
            StmtKind::Set { value, .. } => value.strip_spans(),
            StmtKind::Case { expr, arms } => {
                expr.strip_spans();
                for arm in arms {
                    for s in &mut arm.body { s.strip_spans(); }
                }
            },
            _ => {}
        }
    }
}

impl Expr {
    pub fn strip_spans(&mut self) {
        self.span = Span::new(0, 0);
        self.node.strip_spans();
    }
}

impl ExprKind {
    pub fn strip_spans(&mut self) {
        match self {
            ExprKind::Command(args) => for a in args { a.strip_spans(); },
            ExprKind::CommandPipe(segs) => for s in segs { for a in s { a.strip_spans(); } },
            ExprKind::Concat(l, r) => { l.strip_spans(); r.strip_spans(); },
            ExprKind::Arith { left, right, .. } => { left.strip_spans(); right.strip_spans(); },
            ExprKind::Compare { left, right, .. } => { left.strip_spans(); right.strip_spans(); },
            ExprKind::And(l, r) => { l.strip_spans(); r.strip_spans(); },
            ExprKind::Or(l, r) => { l.strip_spans(); r.strip_spans(); },
            ExprKind::Not(e) => e.strip_spans(),
            ExprKind::Exists(e) => e.strip_spans(),
            ExprKind::IsDir(e) => e.strip_spans(),
            ExprKind::IsFile(e) => e.strip_spans(),
            ExprKind::IsSymlink(e) => e.strip_spans(),
            ExprKind::IsExec(e) => e.strip_spans(),
            ExprKind::IsReadable(e) => e.strip_spans(),
            ExprKind::IsWritable(e) => e.strip_spans(),
            ExprKind::IsNonEmpty(e) => e.strip_spans(),
            ExprKind::BoolStr(e) => e.strip_spans(),
            ExprKind::Len(e) => e.strip_spans(),
            ExprKind::Index { list, index } => { list.strip_spans(); index.strip_spans(); },
            ExprKind::Field { base, .. } => base.strip_spans(),
            ExprKind::Join { list, sep } => { list.strip_spans(); sep.strip_spans(); },
            ExprKind::Count(e) => e.strip_spans(),
            ExprKind::List(items) => for i in items { i.strip_spans(); },
            ExprKind::Env(e) => e.strip_spans(),
            ExprKind::Input(e) => e.strip_spans(),
            ExprKind::Confirm(e) => e.strip_spans(),
            ExprKind::Call { args, .. } => for a in args { a.strip_spans(); },
            ExprKind::MapLiteral(entries) => for (_, v) in entries { v.strip_spans(); },
            ExprKind::Capture { expr, options } => {
                expr.strip_spans();
                for o in options {
                    o.span = Span::new(0, 0);
                    o.value.strip_spans();
                }
            },
            _ => {}
        }
    }
}

impl RunCall {
    pub fn strip_spans(&mut self) {
         for a in &mut self.args { a.strip_spans(); }
         for o in &mut self.options { 
             o.span = Span::new(0, 0);
             o.value.strip_spans();
         }
    }
}

impl RedirectTarget {
    pub fn strip_spans(&mut self) {
         match self {
             RedirectTarget::File { path, .. } => path.strip_spans(),
             _ => {}
         }
    }
}

