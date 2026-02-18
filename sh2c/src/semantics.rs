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
    /// Variables guaranteed declared/assigned at current point (for use checks + set checks)
    definitely_assigned: HashSet<String>,
    
    /// Variables declared on the CURRENT straight-line path (for immediate redeclare checks)
    /// This tracks declarations in the current block/path segment.
    declared_in_straight_line: HashSet<String>,

    /// Variables EVER declared in the current function scope (to prevent shadowing if desired, though Policy A allows disjoint)
    /// Used mainly for internal tracking or future policy extensions.
    #[allow(dead_code)]
    ever_declared: HashSet<String>,

    /// Source map for diagnostics
    sm: &'a SourceMap,
    /// File path for diagnostics
    file: &'a str,
    /// Options
    opts: &'a SemanticOptions,
}

#[derive(Clone)]
struct BinderContextState {
    definitely_assigned: HashSet<String>,
    declared_in_straight_line: HashSet<String>,
}

impl<'a> BinderContext<'a> {
    fn new(sm: &'a SourceMap, file: &'a str, opts: &'a SemanticOptions) -> Self {
        Self {
            definitely_assigned: HashSet::new(),
            declared_in_straight_line: HashSet::new(),
            ever_declared: HashSet::new(),
            sm,
            file,
            opts,
        }
    }

    fn clone_state(&self) -> BinderContextState {
        BinderContextState {
            definitely_assigned: self.definitely_assigned.clone(),
            declared_in_straight_line: self.declared_in_straight_line.clone(),
        }
    }

    fn restore_state(&mut self, state: BinderContextState) {
        self.definitely_assigned = state.definitely_assigned;
        self.declared_in_straight_line = state.declared_in_straight_line;
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
        ctx.declared_in_straight_line.insert(param.clone());
        ctx.definitely_assigned.insert(param.clone());
        ctx.ever_declared.insert(param.clone());
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

    if name == "stdin_lines" || name == "find0" {
            return Err(CompileError::new(ctx.format_error(
            &format!("{}() can only be used as the iterable in a for-loop", name),
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
            if let ExprKind::Call { name: fname, args, options: _ } = &value.node {
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

            // Check for redeclaration on the SAME path
            if ctx.declared_in_straight_line.contains(&name.node) {
                let msg = format!(
                    "variable '{}' already declared in this scope (on this execution path). Did you mean to use `set {} = ...`?", 
                    name.node, name.node
                );
                return Err(CompileError::new(ctx.format_error(&msg, name.span)));
            }

            // Declare the variable
            ctx.declared_in_straight_line.insert(name.node.clone());
            ctx.definitely_assigned.insert(name.node.clone());
            ctx.ever_declared.insert(name.node.clone());
        }

        StmtKind::Set { target, value } => {
            // Check the RHS expression first
            check_expr(value, ctx)?;

            // Check target
            if let ast::LValue::Var(name) = target {
                // Policy A: set requires variable to be definitely declared (safe by default)
                if !ctx.definitely_assigned.contains(&name.node) {
                    let mut msg = format!("cannot set undeclared variable '{}'", name.node);
                    // Add hint
                    msg.push_str(&format!(". Did you mean to use `let {} = ...`?", name.node));
                    
                    return Err(CompileError::new(ctx.format_error(&msg, name.span)));
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

            // Save state *before* branches
            let before = ctx.clone_state();

            // Check then branch
            // Start with snapshot to isolate branch
            ctx.restore_state(before.clone()); 
            check_block(then_body, ctx)?;
            let after_then = ctx.clone_state();

            // Check elif branches
            let mut elif_results = Vec::new();
            for elif in elifs {
                // Restore snapshot for disjoint branch
                ctx.restore_state(before.clone());
                check_expr(&elif.cond, ctx)?;
                check_block(&elif.body, ctx)?;
                elif_results.push(ctx.clone_state());
            }

            // Check else branch
            let after_else = if let Some(body) = else_body {
                // Restore snapshot for disjoint branch
                ctx.restore_state(before.clone());
                check_block(body, ctx)?;
                ctx.clone_state()
            } else {
                // No else = skip branches = returns to before state
                before.clone()
            };

            // Merge: 
            // - definitely_assigned = intersection of all outcomes
            // - declared_in_straight_line = intersection of all outcomes 
            //   (Policy A: variables declared in ONE path but not others are NOT declared on the merged path)
            
            let mut merged_def = after_then.definitely_assigned;
            let mut merged_path = after_then.declared_in_straight_line;
            
            for r in elif_results {
                merged_def = merged_def.intersection(&r.definitely_assigned).cloned().collect();
                merged_path = merged_path.intersection(&r.declared_in_straight_line).cloned().collect();
            }
            merged_def = merged_def.intersection(&after_else.definitely_assigned).cloned().collect();
            merged_path = merged_path.intersection(&after_else.declared_in_straight_line).cloned().collect();
            
            ctx.restore_state(BinderContextState {
                declared_in_straight_line: merged_path,
                definitely_assigned: merged_def,
            });
        }

        StmtKind::While { cond, body } => {
            check_expr(cond, ctx)?;
            let before = ctx.clone_state();
            
            // Body might run
            // Start with fresh snapshot (though `during` logic accumulates decls that are local to body)
            // But from perspective of *after* the loop, we merge.
            // Wait, inside the loop, we continue from `before`? Yes.
            check_block(body, ctx)?;
            
            let after_body = ctx.clone_state();
            
            // Merge with before (loop might skip)
            let merged_def: HashSet<String> = before.definitely_assigned.intersection(&after_body.definitely_assigned).cloned().collect();
            let merged_path: HashSet<String> = before.declared_in_straight_line.intersection(&after_body.declared_in_straight_line).cloned().collect();
            
            ctx.restore_state(BinderContextState {
                declared_in_straight_line: merged_path,
                definitely_assigned: merged_def,
            });
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
                ast::ForIterable::StdinLines => {}
                ast::ForIterable::Find0(spec) => {
                    if let Some(ref d) = spec.dir { check_expr(d, ctx)?; }
                    if let Some(ref n) = spec.name { check_expr(n, ctx)?; }
                    if let Some(ref t) = spec.type_filter { check_expr(t, ctx)?; }
                    if let Some(ref m) = spec.maxdepth { check_expr(m, ctx)?; }
                }
            }

            // For-loop var is treated as implicit let
            if ctx.declared_in_straight_line.contains(&var.node) {
                 let msg = format!(
                    "variable '{}' already declared in this scope (on this execution path). Did you mean to use `set {} = ...`?", 
                    var.node, var.node
                );
                return Err(CompileError::new(ctx.format_error(&msg, var.span)));
            }
            ctx.declared_in_straight_line.insert(var.node.clone());
            ctx.definitely_assigned.insert(var.node.clone());
            ctx.ever_declared.insert(var.node.clone());

            let before = ctx.clone_state();
            check_block(body, ctx)?;
            
            // Loop might not execute (empty range), merge with before
            let after_body = ctx.clone_state();
             
            let merged_def: HashSet<String> = before.definitely_assigned.intersection(&after_body.definitely_assigned).cloned().collect();
            let merged_path: HashSet<String> = before.declared_in_straight_line.intersection(&after_body.declared_in_straight_line).cloned().collect();

            // Note: In Policy A, loop variables persist after the loop.
            // Since we added `var` to `before` state (by declaring it before the body), 
            // and `after_body` also has it, the intersection `merged_def` includes `var`.

            ctx.restore_state(BinderContextState {
                declared_in_straight_line: merged_path,
                definitely_assigned: merged_def,
            });
        }

        StmtKind::ForMap { key_var, val_var, map, body } => {
            // Check that map variable exists
            if !ctx.definitely_assigned.contains(map) {
                return Err(CompileError::new(ctx.format_error(
                    &format!("undefined variable '{}'", map),
                    stmt.span,
                )));
            }

            // Declare loop vars
            for var in [key_var, val_var] {
                if ctx.declared_in_straight_line.contains(&var.node) {
                     return Err(CompileError::new(ctx.format_error(
                        &format!("variable '{}' already declared in this scope. Did you mean to use `set {} = ...`?", var.node, var.node),
                        var.span,
                    )));
                }
                ctx.declared_in_straight_line.insert(var.node.clone());
                ctx.definitely_assigned.insert(var.node.clone());
                ctx.ever_declared.insert(var.node.clone());
            }

            let before = ctx.clone_state();
            check_block(body, ctx)?;
            
            let after_body = ctx.clone_state();
            let merged_def: HashSet<String> = before.definitely_assigned.intersection(&after_body.definitely_assigned).cloned().collect();
            let merged_path: HashSet<String> = before.declared_in_straight_line.intersection(&after_body.declared_in_straight_line).cloned().collect();
            
            ctx.restore_state(BinderContextState {
                declared_in_straight_line: merged_path,
                definitely_assigned: merged_def,
            });
        }

        StmtKind::Case { expr, arms } => {
            check_expr(expr, ctx)?;

            let before = ctx.clone_state();
            let mut arm_results = Vec::new();

            for arm in arms {
                ctx.restore_state(before.clone());
                check_block(&arm.body, ctx)?;
                arm_results.push(ctx.clone_state());
            }

            // Merge all arms (case might not match anything without wildcard)
            
            let has_wildcard = arms.iter().any(|a| a.patterns.iter().any(|p| matches!(p, ast::Pattern::Wildcard)));
            
            let mut merged_def: HashSet<String>;
            let mut merged_path: HashSet<String>;
             
            if arm_results.is_empty() {
                // No arms? Just before.
                 merged_def = before.definitely_assigned.clone();
                 merged_path = before.declared_in_straight_line.clone();
            } else {
                 let first = &arm_results[0];
                 merged_def = first.definitely_assigned.clone();
                 merged_path = first.declared_in_straight_line.clone();
                 
                 for r in &arm_results[1..] {
                     merged_def = merged_def.intersection(&r.definitely_assigned).cloned().collect();
                     merged_path = merged_path.intersection(&r.declared_in_straight_line).cloned().collect();
                 }
            }

            if !has_wildcard {
                // If not exhaustive, merge with "did not enter any arm" state (which is `before`)
                merged_def = merged_def.intersection(&before.definitely_assigned).cloned().collect();
                merged_path = merged_path.intersection(&before.declared_in_straight_line).cloned().collect();
            }

             ctx.restore_state(BinderContextState {
                declared_in_straight_line: merged_path,
                definitely_assigned: merged_def,
            });
        }

        StmtKind::TryCatch { try_body, catch_body } => {
            let before = ctx.clone_state();

            ctx.restore_state(before.clone());
            check_block(try_body, ctx)?;
            let after_try = ctx.clone_state();

            ctx.restore_state(before.clone());
            check_block(catch_body, ctx)?;
            let after_catch = ctx.clone_state();

            // Either path could execute
            let merged_def: HashSet<String> = after_try.definitely_assigned.intersection(&after_catch.definitely_assigned).cloned().collect();
            let merged_path: HashSet<String> = after_try.declared_in_straight_line.intersection(&after_catch.declared_in_straight_line).cloned().collect();
            
            ctx.restore_state(BinderContextState {
                declared_in_straight_line: merged_path,
                definitely_assigned: merged_def,
            });
        }

        StmtKind::AndThen { left, right } | StmtKind::OrElse { left, right } => {
            // These constructs imply order: Left runs, then MAYBE Right.
            check_block(left, ctx)?;
            
            // Save state before Right (which includes Left's outcome).
            let before_right = ctx.clone_state();
            check_block(right, ctx)?;
            
            let after_right = ctx.clone_state();
            
            // Merge outcome: Right ran OR Right didn't run (so just before_right).
             let merged_def: HashSet<String> = before_right.definitely_assigned.intersection(&after_right.definitely_assigned).cloned().collect();
             let merged_path: HashSet<String> = before_right.declared_in_straight_line.intersection(&after_right.declared_in_straight_line).cloned().collect();

              ctx.restore_state(BinderContextState {
                declared_in_straight_line: merged_path,
                definitely_assigned: merged_def,
            });
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
                    ast::PipeSegment::EachLine(var, body) => {
                         // Similar to For loop: Implicit Declaration
                         if ctx.declared_in_straight_line.contains(&var.node) { 
                             return Err(CompileError::new(ctx.format_error(
                                 &format!("variable '{}' already declared in this scope. Did you mean to use `set {} = ...`?", var.node, var.node),
                                 var.span,
                              )));
                        }
                        ctx.declared_in_straight_line.insert(var.node.clone());
                        ctx.definitely_assigned.insert(var.node.clone());
                        ctx.ever_declared.insert(var.node.clone());

                        let before = ctx.clone_state();
                        
                        check_block(body, ctx)?;
                        
                        // Body execution conditional, but variable declaration persists (lifted)
                        // Merge with matching logic of `for` loop logic - intersection.
                        // Since `var` was in `before`, it will be in intersection if `after_body` has it.
                        let after_body = ctx.clone_state();
                        
                        // Wait, previous logic was: "Logic dictates it persists... restore(before)".
                        // BUT that was wrong. `restore(before)` restores the whole state.
                        // We must do merging logic like FOR loop.
                        
                        let merged_def: HashSet<String> = before.definitely_assigned.intersection(&after_body.definitely_assigned).cloned().collect();
                        let merged_path: HashSet<String> = before.declared_in_straight_line.intersection(&after_body.declared_in_straight_line).cloned().collect();

                        ctx.restore_state(BinderContextState {
                            declared_in_straight_line: merged_path,
                            definitely_assigned: merged_def,
                        });
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

fn check_expr(expr: &ast::Expr, ctx: &mut BinderContext) -> Result<(), CompileError> {
    match &expr.node {
        ExprKind::Var(name) => {
            if !ctx.definitely_assigned.contains(name) {
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
            if !ctx.definitely_assigned.contains(map) {
                return Err(CompileError::new(ctx.format_error(
                    &format!("undefined variable '{}'", map),
                    expr.span,
                )));
            }
        }
        ExprKind::Call { name, args, options: _ } => {
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
