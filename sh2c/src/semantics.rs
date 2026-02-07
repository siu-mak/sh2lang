//! Semantic analysis pass (binder) for sh2
//!
//! This pass runs after parsing and before lowering to check:
//! - Declaration before use (undefined variable errors)
//! - Set requires prior let (undeclared set errors)
//! - No redeclaration in same scope

use crate::ast::{self, ExprKind, StmtKind};
use crate::error::CompileError;
use crate::span::SourceMap;
use std::collections::HashSet;
use std::path::PathBuf;

/// Options for semantic analysis
pub struct SemanticOptions {
    pub diag_base_dir: Option<PathBuf>,
}

impl Default for SemanticOptions {
    fn default() -> Self {
        Self { diag_base_dir: None }
    }
}

/// State for semantic analysis within a function
struct BinderContext<'a> {
    /// All variables ever declared in this function (for redeclare + set checks)
    all_declared: HashSet<String>,
    /// Variables guaranteed declared at current point (for use checks)
    definitely_declared: HashSet<String>,
    /// Source map for diagnostics
    sm: &'a SourceMap,
    /// File path for diagnostics
    file: &'a str,
    /// Options
    opts: &'a SemanticOptions,
}

impl<'a> BinderContext<'a> {
    fn new(sm: &'a SourceMap, file: &'a str, opts: &'a SemanticOptions) -> Self {
        Self {
            all_declared: HashSet::new(),
            definitely_declared: HashSet::new(),
            sm,
            file,
            opts,
        }
    }

    fn clone_definitely(&self) -> HashSet<String> {
        self.definitely_declared.clone()
    }

    fn restore_definitely(&mut self, set: HashSet<String>) {
        self.definitely_declared = set;
    }

    fn merge_definitely(&mut self, other: &HashSet<String>) {
        self.definitely_declared = self.definitely_declared.intersection(other).cloned().collect();
    }

    fn format_error(&self, msg: &str, span: crate::span::Span) -> String {
        self.sm.format_diagnostic(self.file, self.opts.diag_base_dir.as_deref(), msg, span)
    }
}

/// Check semantics for a whole program
pub fn check_semantics(program: &ast::Program, opts: &SemanticOptions) -> Result<(), CompileError> {
    for func in &program.functions {
        let sm = program.source_maps.get(&func.file)
            .ok_or_else(|| CompileError::new(format!("internal error: missing source map for file {}", func.file)))?;
        check_function(func, sm, opts)?;
    }
    Ok(())
}

/// Check semantics for a single function
fn check_function(func: &ast::Function, sm: &SourceMap, opts: &SemanticOptions) -> Result<(), CompileError> {
    let mut ctx = BinderContext::new(sm, &func.file, opts);

    // Function parameters are pre-declared
    for param in &func.params {
        ctx.all_declared.insert(param.clone());
        ctx.definitely_declared.insert(param.clone());
    }

    // Check all statements
    check_block(&func.body, &mut ctx)?;

    Ok(())
}

/// Check a block of statements
fn check_block(stmts: &[ast::Stmt], ctx: &mut BinderContext) -> Result<(), CompileError> {
    for stmt in stmts {
        check_stmt(stmt, ctx)?;
    }
    Ok(())
}

// check_run_call and check_run_arg removed




fn check_function_call(
    name: &str,
    args: &[ast::Expr],
    span: crate::span::Span,
    ctx: &mut BinderContext,
) -> Result<(), CompileError> {
    // Special validation logic:
    if name == "try_run" {
            return Err(CompileError::new(ctx.format_error(
            "try_run() must be bound via let (e.g., let r = try_run(...))",
            span,
        )));
    }
    
    if name == "write_file" {
        // write_file(path, content, append?)
        // If 3rd arg exists, it MUST be a boolean literal
        if args.len() >= 3 {
            if !matches!(args[2].node, ast::ExprKind::Bool(_)) {
                    return Err(CompileError::new(ctx.format_error(
                    "write_file: append must be boolean literal",
                    args[2].span,
                )));
            }
        }
    }
    
    for a in args {
        check_expr(a, ctx)?;
    }
    Ok(())
}

/// Check a single statement
fn check_stmt(stmt: &ast::Stmt, ctx: &mut BinderContext) -> Result<(), CompileError> {
    match &stmt.node {
        StmtKind::Let { name, value } => {
            // Special handling for try_run: allowed ONLY in Let RHS
            // We check this BEFORE general check_expr to allow it here (it's disallowed elsewhere)
            if let ExprKind::Call { name: fname, args } = &value.node {
                if fname == "try_run" {
                    // Check try_run args normally
                    for arg in args {
                        check_expr(arg, ctx)?;
                    }
                    // proceed to declaration below
                } else {
                    check_expr(value, ctx)?;
                }
            } else {
                check_expr(value, ctx)?;
            }

            // Check for redeclaration
            if ctx.all_declared.contains(name) {
                return Err(CompileError::new(ctx.format_error(
                    &format!("variable '{}' already declared in this scope", name),
                    stmt.span,
                )));
            }

            // Declare the variable
            ctx.all_declared.insert(name.clone());
            ctx.definitely_declared.insert(name.clone());
        }

        StmtKind::Set { target, value } => {
            // Check the RHS expression first
            check_expr(value, ctx)?;

            // Check target
            if let ast::LValue::Var(name) = target {
                if !ctx.all_declared.contains(name) {
                    return Err(CompileError::new(ctx.format_error(
                        &format!("cannot set undeclared variable '{}'", name),
                        stmt.span,
                    )));
                }
            }
            // env.X is always allowed (no local declaration needed)
        }

        StmtKind::Print(e) | StmtKind::PrintErr(e) => {
            check_expr(e, ctx)?;
        }

        StmtKind::Run(run_call) => {
            // Run args must be valid expressions (variable refs must be declared)
            for arg in &run_call.args {
                check_expr(arg, ctx)?;
            }
            for opt in &run_call.options {
                check_expr(&opt.value, ctx)?;
            }
        }

        StmtKind::If { cond, then_body, elifs, else_body } => {
            check_expr(cond, ctx)?;

            // Save state before branches
            let before = ctx.clone_definitely();

            // Check then branch
            check_block(then_body, ctx)?;
            let after_then = ctx.clone_definitely();

            // Check elif branches
            let mut elif_results = Vec::new();
            for elif in elifs {
                ctx.restore_definitely(before.clone());
                check_expr(&elif.cond, ctx)?;
                check_block(&elif.body, ctx)?;
                elif_results.push(ctx.clone_definitely());
            }

            // Check else branch
            let after_else = if let Some(body) = else_body {
                ctx.restore_definitely(before.clone());
                check_block(body, ctx)?;
                ctx.clone_definitely()
            } else {
                // No else = could skip all branches
                before.clone()
            };

            // Merge: definitely_declared = intersection of all outcomes
            let mut merged = after_then;
            for r in elif_results {
                merged = merged.intersection(&r).cloned().collect();
            }
            merged = merged.intersection(&after_else).cloned().collect();
            ctx.restore_definitely(merged);
        }

        StmtKind::While { cond, body } => {
            check_expr(cond, ctx)?;
            let before = ctx.clone_definitely();
            check_block(body, ctx)?;
            // Loop might not execute, so merge with before
            ctx.merge_definitely(&before);
        }

        StmtKind::For { var, iterable, body } => {
            // Check iterable
            match iterable {
                ast::ForIterable::List(items) => {
                    for item in items {
                        check_expr(item, ctx)?;
                    }
                }
                ast::ForIterable::Range(start, end) => {
                    check_expr(start, ctx)?;
                    check_expr(end, ctx)?;
                }
            }

            // For-loop var is treated as implicit let (Option A: function-scoped)
            if ctx.all_declared.contains(var) {
                return Err(CompileError::new(ctx.format_error(
                    &format!("variable '{}' already declared in this scope", var),
                    stmt.span,
                )));
            }
            ctx.all_declared.insert(var.clone());
            ctx.definitely_declared.insert(var.clone());

            let before = ctx.clone_definitely();
            check_block(body, ctx)?;
            // Loop might not execute (empty range), merge with before
            ctx.merge_definitely(&before);
        }

        StmtKind::ForMap { key_var, val_var, map, body } => {
            // Check that map variable exists
            if !ctx.definitely_declared.contains(map) {
                return Err(CompileError::new(ctx.format_error(
                    &format!("undefined variable '{}'", map),
                    stmt.span,
                )));
            }

            // Declare loop vars
            for var in [key_var, val_var] {
                if ctx.all_declared.contains(var) {
                    return Err(CompileError::new(ctx.format_error(
                        &format!("variable '{}' already declared in this scope", var),
                        stmt.span,
                    )));
                }
                ctx.all_declared.insert(var.clone());
                ctx.definitely_declared.insert(var.clone());
            }

            let before = ctx.clone_definitely();
            check_block(body, ctx)?;
            ctx.merge_definitely(&before);
        }

        StmtKind::Case { expr, arms } => {
            check_expr(expr, ctx)?;

            let before = ctx.clone_definitely();
            let mut arm_results = Vec::new();

            for arm in arms {
                ctx.restore_definitely(before.clone());
                check_block(&arm.body, ctx)?;
                arm_results.push(ctx.clone_definitely());
            }

            // Merge all arms (case might not match anything without wildcard)
            let mut merged = before.clone();
            for r in arm_results {
                merged = merged.intersection(&r).cloned().collect();
            }
            ctx.restore_definitely(merged);
        }

        StmtKind::TryCatch { try_body, catch_body } => {
            let before = ctx.clone_definitely();

            check_block(try_body, ctx)?;
            let after_try = ctx.clone_definitely();

            ctx.restore_definitely(before);
            check_block(catch_body, ctx)?;
            let after_catch = ctx.clone_definitely();

            // Either path could execute
            let merged: HashSet<String> = after_try.intersection(&after_catch).cloned().collect();
            ctx.restore_definitely(merged);
        }

        StmtKind::AndThen { left, right } | StmtKind::OrElse { left, right } => {
            check_block(left, ctx)?;
            // Right might not execute
            let before_right = ctx.clone_definitely();
            check_block(right, ctx)?;
            ctx.merge_definitely(&before_right);
        }

        StmtKind::WithEnv { bindings, body } => {
            for (_, v) in bindings {
                check_expr(v, ctx)?;
            }
            check_block(body, ctx)?;
        }

        StmtKind::WithCwd { path, body } => {
            check_expr(path, ctx)?;
            check_block(body, ctx)?;
        }

        StmtKind::WithLog { path, body, .. } => {
            check_expr(path, ctx)?;
            check_block(body, ctx)?;
        }

        StmtKind::WithRedirect { stdout, stderr, stdin, body } => {
            if let Some(targets) = stdout {
                for t in targets {
                    check_redirect_output(&t.node, ctx)?;
                }
            }
            if let Some(targets) = stderr {
                for t in targets {
                    check_redirect_output(&t.node, ctx)?;
                }
            }
            if let Some(t) = stdin {
                check_redirect_input(t, ctx)?;
            }
            check_block(body, ctx)?;
        }

        StmtKind::Subshell { body } | StmtKind::Group { body } => {
            check_block(body, ctx)?;
        }

        StmtKind::Spawn { stmt } => {
            check_stmt(stmt, ctx)?;
        }

        StmtKind::Wait(Some(e)) => {
            check_expr(e, ctx)?;
        }

        StmtKind::Return(Some(e)) | StmtKind::Exit(Some(e)) => {
            check_expr(e, ctx)?;
        }

        StmtKind::Sh(e) => {
            check_expr(e, ctx)?;
        }

        StmtKind::Cd { path } => {
            check_expr(path, ctx)?;
        }

        StmtKind::Export { value: Some(v), .. } => {
            check_expr(v, ctx)?;
        }

        StmtKind::Source { path } => {
            check_expr(path, ctx)?;
        }

        StmtKind::Exec(args) => {
            for a in args {
                check_expr(a, ctx)?;
            }
        }

        StmtKind::Call { name, args } => {
            check_function_call(name, args, stmt.span, ctx)?;
        }

        StmtKind::Pipe(segments) => {
            for seg in segments {
                match &seg.node {
                    ast::PipeSegment::Run(run_call) | ast::PipeSegment::Sudo(run_call) => {
                        for arg in &run_call.args {
                            check_expr(arg, ctx)?;
                        }
                        for opt in &run_call.options {
                            check_expr(&opt.value, ctx)?;
                        }
                    }
                    ast::PipeSegment::Block(stmts) => {
                        check_block(stmts, ctx)?;
                    }
                }
            }
        }

        StmtKind::ShBlock(_) => {
            // Raw shell lines, nothing to check
        }

        // Terminal statements with no expressions
        StmtKind::Break | StmtKind::Continue | StmtKind::Return(None) 
        | StmtKind::Exit(None) | StmtKind::Wait(None) | StmtKind::Export { value: None, .. }
        | StmtKind::Unset { .. } => {}
    }

    Ok(())
}

/// Check an expression for undefined variable references
fn check_expr(expr: &ast::Expr, ctx: &mut BinderContext) -> Result<(), CompileError> {
    match &expr.node {
        ExprKind::Var(name) => {
            if !ctx.definitely_declared.contains(name) {
                return Err(CompileError::new(ctx.format_error(
                    &format!("undefined variable '{}'", name),
                    expr.span,
                )));
            }
        }

        ExprKind::Literal(_) | ExprKind::Bool(_) | ExprKind::Number(_) => {}

        ExprKind::Concat(l, r) | ExprKind::And(l, r) | ExprKind::Or(l, r) => {
            check_expr(l, ctx)?;
            check_expr(r, ctx)?;
        }

        ExprKind::Arith { left, right, .. } | ExprKind::Compare { left, right, .. } => {
            check_expr(left, ctx)?;
            check_expr(right, ctx)?;
        }

        ExprKind::Not(e) | ExprKind::Exists(e) | ExprKind::IsDir(e) | ExprKind::IsFile(e)
        | ExprKind::IsSymlink(e) | ExprKind::IsExec(e) | ExprKind::IsReadable(e)
        | ExprKind::IsWritable(e) | ExprKind::IsNonEmpty(e) | ExprKind::BoolStr(e)
        | ExprKind::Len(e) | ExprKind::Count(e) | ExprKind::Arg(e) | ExprKind::Env(e)
        | ExprKind::Input(e) => {
            check_expr(e, ctx)?;
        }

        ExprKind::Index { list, index } => {
            check_expr(list, ctx)?;
            check_expr(index, ctx)?;
        }

        ExprKind::Field { base, .. } => {
            check_expr(base, ctx)?;
        }

        ExprKind::Join { list, sep } => {
            check_expr(list, ctx)?;
            check_expr(sep, ctx)?;
        }

        ExprKind::List(items) => {
            for item in items {
                check_expr(item, ctx)?;
            }
        }

        ExprKind::MapLiteral(entries) => {
            for (_, v) in entries {
                check_expr(v, ctx)?;
            }
        }

        ExprKind::MapIndex { map, .. } => {
            if !ctx.definitely_declared.contains(map) {
                return Err(CompileError::new(ctx.format_error(
                    &format!("undefined variable '{}'", map),
                    expr.span,
                )));
            }
        }

        ExprKind::Call { name, args } => {
            check_function_call(name, args, expr.span, ctx)?;
        }

        ExprKind::Run(run_call) => {
             for arg in &run_call.args {
                check_expr(arg, ctx)?;
            }
            for opt in &run_call.options {
                check_expr(&opt.value, ctx)?;
            }
        }

        ExprKind::Capture { expr: inner, options } => {
            check_expr(inner, ctx)?;
            for opt in options {
                check_expr(&opt.value, ctx)?;
            }
        }

        ExprKind::Sh { cmd, options } => {
            check_expr(cmd, ctx)?;
            for opt in options {
                check_expr(&opt.value, ctx)?;
            }
        }

        ExprKind::Confirm { prompt, default } => {
            check_expr(prompt, ctx)?;
            if let Some(d) = default {
                check_expr(d, ctx)?;
            }
        }

        ExprKind::Sudo { args, options } => {
            for a in args {
                check_expr(a, ctx)?;
            }
            for opt in options {
                check_expr(&opt.value, ctx)?;
            }
        }

        // Expr kinds with no sub-expressions to check
        ExprKind::Command(_) | ExprKind::CommandPipe(_) | ExprKind::Args
        | ExprKind::Status | ExprKind::Pid | ExprKind::Uid | ExprKind::Ppid
        | ExprKind::Pwd | ExprKind::SelfPid | ExprKind::Argv0 | ExprKind::Argc
        | ExprKind::EnvDot(_) => {}
    }

    Ok(())
}

fn check_redirect_output(target: &ast::RedirectOutputTarget, ctx: &mut BinderContext) -> Result<(), CompileError> {
    if let ast::RedirectOutputTarget::File { path, .. } = target {
        check_expr(path, ctx)?;
    }
    Ok(())
}

fn check_redirect_input(target: &ast::RedirectInputTarget, ctx: &mut BinderContext) -> Result<(), CompileError> {
    if let ast::RedirectInputTarget::File { path } = target {
        check_expr(path, ctx)?;
    }
    Ok(())
}
