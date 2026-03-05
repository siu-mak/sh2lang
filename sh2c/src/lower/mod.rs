use crate::ast;
use crate::ir;
use crate::span::{Span, SourceMap};
use crate::error::CompileError;

mod expr;

mod stmt;
use self::stmt::lower_stmt;

mod sudo;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub(super) struct LoweringContext<'a> {
    run_results: HashSet<String>,
    /// Variables that hold boolean values (assigned from boolean expressions)
    bool_vars: HashSet<String>,
    /// Variables that are known to hold list values (e.g. from list literals)
    list_vars: HashSet<String>,
    /// User-defined function names for call validation
    user_funcs: &'a HashSet<String>,
    opts: &'a LowerOptions,
    in_let_rhs: bool,
    tmp_counter: usize,
}

impl<'a> LoweringContext<'a> {
    fn new(opts: &'a LowerOptions, user_funcs: &'a HashSet<String>) -> Self {
        Self {
            run_results: HashSet::new(),
            bool_vars: HashSet::new(),
            list_vars: HashSet::new(),
            user_funcs,
            opts,
            in_let_rhs: false,
            tmp_counter: 0,
        }
    }

    pub(super) fn opts(&self) -> &'a LowerOptions {
        self.opts
    }

    fn insert(&mut self, name: &str) {
        self.run_results.insert(name.to_string());
    }

    fn remove(&mut self, name: &str) {
        self.run_results.remove(name);
    }

    fn insert_bool_var(&mut self, name: &str) {
        self.bool_vars.insert(name.to_string());
    }

    fn is_bool_var(&self, name: &str) -> bool {
        self.bool_vars.contains(name)
    }

    fn insert_list_var(&mut self, name: &str) {
        self.list_vars.insert(name.to_string());
    }

    fn is_list_var(&self, name: &str) -> bool {
        self.list_vars.contains(name)
    }



    fn intersection(&self, other: &Self) -> Self {
        let run_results = self
            .run_results
            .intersection(&other.run_results)
            .cloned()
            .collect();
        let bool_vars = self
            .bool_vars
            .intersection(&other.bool_vars)
            .cloned()
            .collect();
        let list_vars = self
            .list_vars
            .intersection(&other.list_vars)
            .cloned()
            .collect();
        Self {
            run_results,
            bool_vars,
            list_vars,
            user_funcs: self.user_funcs,
            opts: self.opts,
            in_let_rhs: self.in_let_rhs,
            tmp_counter: std::cmp::max(self.tmp_counter, other.tmp_counter),
        }
    }
}

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct LowerOptions {
    pub include_diagnostics: bool,
    pub diag_base_dir: Option<PathBuf>,
    pub target: crate::codegen::TargetShell,
}

impl Default for LowerOptions {
    fn default() -> Self {
        Self {
            include_diagnostics: true,
            diag_base_dir: None,
            target: crate::codegen::TargetShell::Bash,
        }
    }
}

/// Lower a whole program (AST) into IR
pub fn lower(p: ast::Program) -> Result<Vec<ir::Function>, CompileError> {
    lower_with_options(p, &LowerOptions::default())
}

pub fn lower_with_options(p: ast::Program, opts: &LowerOptions) -> Result<Vec<ir::Function>, CompileError> {
    let has_main = p.functions.iter().any(|f| f.name == "main");

    // Collect user-defined function names for call validation
    let user_funcs: HashSet<String> = p.functions.iter().map(|f| f.name.clone()).collect();

    let entry_file = &p.entry_file;
    let maps = &p.source_maps;

    let entry_sm = maps
        .get(entry_file)
        .expect("Missing source map for entry file");

    let mut ir_funcs = Vec::new();

    if !has_main {
        return Err(CompileError::new(entry_sm.format_diagnostic(entry_file, opts.diag_base_dir.as_deref(), "No entrypoint: define `func main()`.", p.span)));
    }
    for f in p.functions {
        let sm = maps.get(&f.file).expect("Missing source map");
        ir_funcs.push(lower_function(f, sm, opts, &user_funcs)?);
    }

    Ok(ir_funcs)
}

/// Lower a single function
fn lower_function(f: ast::Function, sm: &SourceMap, opts: &LowerOptions, user_funcs: &HashSet<String>) -> Result<ir::Function, CompileError> {
    let mut body = Vec::new();
    let mut ctx = LoweringContext::new(opts, user_funcs);

    for stmt in f.body {
        ctx = lower_stmt(stmt, &mut body, ctx, sm, &f.file, opts)?;
    }

    Ok(ir::Function {
        name: f.name,
        params: f.params,
        commands: body,
        file: f.file,
    })
}

/// Helper to lower a block of statements sequentially
pub(super) fn lower_block<'a>(
    stmts: &[ast::Stmt],
    out: &mut Vec<ir::Cmd>,
    mut ctx: LoweringContext<'a>,
    sm: &SourceMap,
    file: &str,
    opts: &'a LowerOptions,
) -> Result<LoweringContext<'a>, CompileError> {
    for stmt in stmts {
        // We pass a clone of the statement because lower_stmt consumes it
        // Note: lower_stmt signature might need update or we keep cloning here if lower_stmt consumes.
        // lower_stmt(stmt, ...) -> Stmt is large? Spanned<StmtKind>.
        // StmtKind can be large. 
        // If lower_stmt consumes Stmt, we must clone if we have &Stmt.
        // Let's check lower_stmt.
        ctx = lower_stmt(stmt.clone(), out, ctx, sm, file, opts)?;
    }
    Ok(ctx)
}

pub(super) fn resolve_span(
    span: Span,
    sm: &SourceMap,
    file: &str,
    base: Option<&std::path::Path>,
) -> String {
    let (line, col) = sm.line_col(span.start);
    let display_file = crate::diag_path::display_path(file, base);
    format!("{}:{}:{}", display_file, line, col)
}
