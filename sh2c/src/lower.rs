use crate::ast::{self, ArithOp, CompareOp, Expr, ExprKind, LValue, Pattern, Spanned, Stmt, StmtKind};
use crate::ir;
use crate::span::{Span, SourceMap};
use crate::error::CompileError;
use crate::sudo::SudoSpec;
use std::collections::HashSet;

#[derive(Clone, Debug)]
struct LoweringContext<'a> {
    run_results: HashSet<String>,
    /// Variables that hold boolean values (assigned from boolean expressions)
    bool_vars: HashSet<String>,
    opts: &'a LowerOptions,
    in_let_rhs: bool,
}

impl<'a> LoweringContext<'a> {
    fn new(opts: &'a LowerOptions) -> Self {
        Self {
            run_results: HashSet::new(),
            bool_vars: HashSet::new(),
            opts,
            in_let_rhs: false,
        }
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
        Self {
            run_results,
            bool_vars,
            opts: self.opts,
            in_let_rhs: self.in_let_rhs,
        }
    }
}

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct LowerOptions {
    pub include_diagnostics: bool,
    pub diag_base_dir: Option<PathBuf>,
}

impl Default for LowerOptions {
    fn default() -> Self {
        Self {
            include_diagnostics: true,
            diag_base_dir: None,
        }
    }
}

/// Lower a whole program (AST) into IR
pub fn lower(p: ast::Program) -> Result<Vec<ir::Function>, CompileError> {
    lower_with_options(p, &LowerOptions::default())
}

pub fn lower_with_options(p: ast::Program, opts: &LowerOptions) -> Result<Vec<ir::Function>, CompileError> {
    let has_main = p.functions.iter().any(|f| f.name == "main");


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
        ir_funcs.push(lower_function(f, sm, opts)?);
    }

    Ok(ir_funcs)
}

/// Lower a single function
fn lower_function(f: ast::Function, sm: &SourceMap, opts: &LowerOptions) -> Result<ir::Function, CompileError> {
    let mut body = Vec::new();
    let mut ctx = LoweringContext::new(opts);

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
fn lower_block<'a>(
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

fn resolve_span(
    span: Span,
    sm: &SourceMap,
    file: &str,
    base: Option<&std::path::Path>,
) -> String {
    let (line, col) = sm.line_col(span.start);
    let display_file = crate::diag_path::display_path(file, base);
    format!("{}:{}:{}", display_file, line, col)
}

/// Check if an AST expression is boolean-typed (comparison, logical op, predicate, etc.)
/// Used to determine if a let-binding should track the variable as bool-typed.
///
/// IMPORTANT: This is an intentional allowlist of known boolean-returning expressions:
/// - Bool literals (true, false)
/// - Comparisons (==, !=, <, >, <=, >=)
/// - Logical operators (&&, ||, !)
/// - Known predicate builtins: exists, is_dir, is_file, is_symlink, is_exec,
///   is_readable, is_writable, is_non_empty, matches, contains, contains_line, confirm
///
/// If new boolean-returning builtins are added, this list must be updated.
fn is_bool_expr(e: &ast::Expr) -> bool {
    match &e.node {
        ast::ExprKind::Bool(_) => true,
        ast::ExprKind::Compare { .. } => true,
        ast::ExprKind::And(_, _) | ast::ExprKind::Or(_, _) | ast::ExprKind::Not(_) => true,
        ast::ExprKind::Call { name, .. } => {
            // Allowlist of known boolean-returning builtins
            matches!(
                name.as_str(),
                "exists"
                    | "is_dir"
                    | "is_file"
                    | "is_symlink"
                    | "is_exec"
                    | "is_readable"
                    | "is_writable"
                    | "is_non_empty"
                    | "matches"
                    | "contains"
                    | "contains_line"
                    | "confirm"
            )
        }
        _ => false,
    }
}

/// Lower one AST statement into IR commands. Returns the updated context after this statement.
fn lower_stmt<'a>(
    stmt: ast::Stmt,
    out: &mut Vec<ir::Cmd>,
    mut ctx: LoweringContext<'a>,
    sm: &SourceMap,
    file: &str,
    opts: &'a LowerOptions,
) -> Result<LoweringContext<'a>, CompileError> {
    let loc = if opts.include_diagnostics {
        Some(resolve_span(
            stmt.span,
            sm,
            file,
            opts.diag_base_dir.as_deref(),
        ))
    } else {
        None
    };
    match stmt.node {
        ast::StmtKind::Let { name, value } => {
            // Special handling for try_run to allow it ONLY during strict let-binding lowering.
            if let ast::ExprKind::Call {
                name: func_name,
                args,
            } = &value.node
            {
                if func_name == "try_run" {
                    if args.is_empty() {
                        return Err(CompileError::new(sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            "try_run() requires at least 1 argument (cmd)",
                            value.span,
                        )));
                    }
                    let lowered_args = args
                        .clone()
                        .into_iter()
                        .map(|a| lower_expr(a, &mut ctx, sm, file))
                        .collect::<Result<Vec<_>, _>>()?;
                    out.push(ir::Cmd::Assign(
                        name.clone(),
                        ir::Val::TryRun(lowered_args),
                        loc,
                    ));
                    ctx.insert(&name);
                    return Ok(ctx);
                }
            }
            // Check if RHS is a boolean expression and track it
            let is_bool = is_bool_expr(&value);
            
            ctx.in_let_rhs = true;
            let val_ir = lower_expr(value, &mut ctx, sm, file)?;
            ctx.in_let_rhs = false;
            
            let is_capture_result = matches!(&val_ir, ir::Val::Capture { allow_fail: true, .. });

            out.push(ir::Cmd::Assign(
                name.clone(),
                val_ir,
                loc,
            ));
            ctx.remove(&name);
            if is_capture_result {
                ctx.insert(&name);
            }
            if is_bool {
                ctx.insert_bool_var(&name);
            }
            Ok(ctx)
        }

        ast::StmtKind::Run(run_call) => {
            let ir_args = run_call
                .args
                .into_iter()
                .map(|a| lower_expr(a, &mut ctx, sm, file))
                .collect::<Result<Vec<_>, _>>()?;

            let mut allow_fail = false;
            let mut seen_allow_fail = false;
            for opt in run_call.options {
                if opt.name == "allow_fail" {
                    if seen_allow_fail {
                        return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "allow_fail specified more than once", opt.span)));
                    }
                    seen_allow_fail = true;
                    if let ast::ExprKind::Bool(b) = opt.value.node {
                        allow_fail = b;
                    } else {
                        return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "allow_fail must be true/false literal", opt.value.span)));
                    }
                } else {
                    return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), format!("unknown run option: {}", opt.name).as_str(), opt.span)));
                }
            }

            out.push(ir::Cmd::Exec {
                args: ir_args,
                allow_fail,
                loc,
            });
            Ok(ctx)
        }

        ast::StmtKind::Print(e) => {
            if let ast::ExprKind::Call { name, args: _ } = &e.node {
                if name == "split" {
                     return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "Cannot emit boolean/list value as string",
                        e.span,
                    )));
                }
            }
            out.push(ir::Cmd::Print(lower_expr(e.clone(), &mut ctx, sm, file)?));
            Ok(ctx)
        }

        ast::StmtKind::PrintErr(e) => {
            out.push(ir::Cmd::PrintErr(lower_expr(e.clone(), &mut ctx, sm, file)?));
            Ok(ctx)
        }
        ast::StmtKind::If {
            cond,
            then_body,
            elifs,
            else_body,
        } => {
            let cond_val = lower_expr(cond, &mut ctx, sm, file)?;

            let mut t_cmds = Vec::new();
            let ctx_then = lower_block(&then_body, &mut t_cmds, ctx.clone(), sm, file, opts)?;

            let mut lowered_elifs = Vec::new();
            let mut ctx_elifs = Vec::new();

            for elif in elifs {
                let mut body_cmds = Vec::new();
                let elif_cond = lower_expr(elif.cond.clone(), &mut ctx, sm, file)?; // Evaluate cond in original context
                let ctx_elif = lower_block(&elif.body, &mut body_cmds, ctx.clone(), sm, file, opts)?;
                lowered_elifs.push((elif_cond, body_cmds));
                ctx_elifs.push(ctx_elif);
            }

            let mut e_cmds = Vec::new();
            let ctx_else = if let Some(body) = else_body {
                lower_block(&body, &mut e_cmds, ctx.clone(), sm, file, opts)?
            } else {
                ctx.clone()
            };

            out.push(ir::Cmd::If {
                cond: cond_val,
                then_body: t_cmds,
                elifs: lowered_elifs,
                else_body: e_cmds,
            });

            // Intersection of all paths
            let mut final_ctx = ctx_then.intersection(&ctx_else);
            for c in ctx_elifs {
                final_ctx = final_ctx.intersection(&c);
            }
            Ok(final_ctx)
        }
        ast::StmtKind::Case { expr, arms } => {
            let expr_val = lower_expr(expr, &mut ctx, sm, file)?;
            let mut lower_arms = Vec::new();
            let mut arm_ctxs = Vec::new();
            let mut has_wildcard = false;

            for arm in arms {
                let mut body_cmds = Vec::new();

                for p in &arm.patterns {
                    if matches!(p, ast::Pattern::Wildcard) {
                        has_wildcard = true;
                    }
                }

                let ctx_arm = lower_block(&arm.body, &mut body_cmds, ctx.clone(), sm, file, opts)?;

                let patterns = arm
                    .patterns
                    .into_iter()
                    .map(|p| match p {
                        ast::Pattern::Literal(s) => ir::Pattern::Literal(s),
                        ast::Pattern::Glob(s) => ir::Pattern::Glob(s),
                        ast::Pattern::Wildcard => ir::Pattern::Wildcard,
                    })
                    .collect();

                lower_arms.push((patterns, body_cmds));
                arm_ctxs.push(ctx_arm);
            }

            out.push(ir::Cmd::Case {
                expr: expr_val,
                arms: lower_arms,
            });

            if arm_ctxs.is_empty() {
                return Ok(ctx);
            }

            let mut final_ctx = arm_ctxs[0].clone();
            for c in arm_ctxs.iter().skip(1) {
                final_ctx = final_ctx.intersection(c);
            }

            if !has_wildcard {
                final_ctx = final_ctx.intersection(&ctx);
            }

            Ok(final_ctx)
        }
        ast::StmtKind::While { cond, body } => {
            let cond_val = lower_expr(cond, &mut ctx, sm, file)?;
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(&body, &mut lower_body, ctx.clone(), sm, file, opts)?;
            out.push(ir::Cmd::While {
                cond: cond_val,
                body: lower_body,
            });
            Ok(ctx.intersection(&ctx_body))
        }
        ast::StmtKind::For { var, items, body } => {
            let lowered_items = items
                .into_iter()
                .map(|i| lower_expr(i, &mut ctx, sm, file))
                .collect::<Result<Vec<_>, _>>()?;
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(&body, &mut lower_body, ctx.clone(), sm, file, opts)?;

            out.push(ir::Cmd::For {
                var,
                items: lowered_items,
                body: lower_body,
            });
            Ok(ctx.intersection(&ctx_body))
        }
        ast::StmtKind::Pipe(segments) => {
            // Optimization: if all segments are Run(...) or Sudo(...), use ir::Cmd::Pipe
            
            let all_cmds = segments.iter().all(|s| matches!(s.node, ast::PipeSegment::Run(_) | ast::PipeSegment::Sudo(_)));

            if all_cmds {
                // Pure command pipeline optimization path
                let mut lowered_segments = Vec::new();
                for seg in segments {
                     match &seg.node {
                        ast::PipeSegment::Run(run_call) => {
                             let (args, allow_fail) = lower_run_call_args(run_call, &mut ctx, sm, file, opts)?;
                             lowered_segments.push((args, allow_fail));
                        }
                        ast::PipeSegment::Sudo(run_call) => {
                             let (args, allow_fail) = lower_sudo_call_args(run_call, &mut ctx, sm, file, opts)?;
                             lowered_segments.push((args, allow_fail));
                        }
                        _ => unreachable!(),
                     }
                }
                out.push(ir::Cmd::Pipe(lowered_segments, loc));
            } else {
                // Mixed or Block pipeline path -> ir::Cmd::PipeBlocks
                let mut lower_segments = Vec::new();
                
                for seg in segments {
                    let mut block_cmds = Vec::new(); 
                    let seg_loc = if opts.include_diagnostics {
                        Some(resolve_span(seg.span, sm, file, opts.diag_base_dir.as_deref()))
                    } else {
                        None
                    };

                    match &seg.node {
                        ast::PipeSegment::Block(stmts) => {
                             lower_block(stmts, &mut block_cmds, ctx.clone(), sm, file, opts)?;
                        }
                        ast::PipeSegment::Run(run_call) => {
                            let mut seg_ctx = ctx.clone();
                            let (args, allow_fail) = lower_run_call_args(run_call, &mut seg_ctx, sm, file, opts)?;
                            
                            block_cmds.push(ir::Cmd::Exec {
                                args,
                                allow_fail,
                                loc: seg_loc, 
                            });
                        }
                        ast::PipeSegment::Sudo(run_call) => {
                            let mut seg_ctx = ctx.clone();
                            let (args, allow_fail) = lower_sudo_call_args(run_call, &mut seg_ctx, sm, file, opts)?;
                            
                            block_cmds.push(ir::Cmd::Exec {
                                args,
                                allow_fail,
                                loc: seg_loc, 
                            });
                        }
                    }
                    lower_segments.push(block_cmds);
                }
                out.push(ir::Cmd::PipeBlocks(lower_segments, loc));
            }
            Ok(ctx)
        }
        ast::StmtKind::Break => {
            out.push(ir::Cmd::Break);
            Ok(ctx)
        }
        ast::StmtKind::Continue => {
            out.push(ir::Cmd::Continue);
            Ok(ctx)
        }
        ast::StmtKind::Return(e) => {
            out.push(ir::Cmd::Return(
                e.map(|x| lower_expr(x, &mut ctx, sm, file)).transpose()?,
            ));
            Ok(ctx)
        }
        ast::StmtKind::Exit(e) => {
            out.push(ir::Cmd::Exit(
                e.map(|x| lower_expr(x, &mut ctx, sm, file)).transpose()?,
            ));
            Ok(ctx)
        }
        ast::StmtKind::WithEnv { bindings, body } => {
            let lowered_bindings = bindings
                .into_iter()
                .map(|(k, v)| {
                     let v = lower_expr(v, &mut ctx, sm, file)?;
                     Ok((k, v))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(&body, &mut lower_body, ctx.clone(), sm, file, opts)?;
            out.push(ir::Cmd::WithEnv {
                bindings: lowered_bindings,
                body: lower_body,
            });
            Ok(ctx_body)
        }
        ast::StmtKind::AndThen { left, right } => {
            let mut lower_left = Vec::new();
            let ctx_left = lower_block(&left, &mut lower_left, ctx.clone(), sm, file, opts)?;

            let mut lower_right = Vec::new();
            let ctx_right = lower_block(&right, &mut lower_right, ctx_left.clone(), sm, file, opts)?;

            out.push(ir::Cmd::AndThen {
                left: lower_left,
                right: lower_right,
            });
            Ok(ctx_left.intersection(&ctx_right))
        }
        ast::StmtKind::OrElse { left, right } => {
            let mut lower_left = Vec::new();
            let ctx_left = lower_block(&left, &mut lower_left, ctx.clone(), sm, file, opts)?;

            let mut lower_right = Vec::new();
            let ctx_right = lower_block(&right, &mut lower_right, ctx_left.clone(), sm, file, opts)?;

            out.push(ir::Cmd::OrElse {
                left: lower_left,
                right: lower_right,
            });
            Ok(ctx_left.intersection(&ctx_right))
        }
        ast::StmtKind::WithCwd { path, body } => {
            if !matches!(path.node, ast::ExprKind::Literal(_)) {
                 return Err(CompileError::new(sm.format_diagnostic(
                    file,
                    opts.diag_base_dir.as_deref(),
                    "cwd(...) requires a string literal path. Computed expressions are not allowed.\n\nhelp: if you need a computed cwd, use run(\"sh\", \"-c\", ...) (and cd inside the shell snippet).",
                    path.span,
                )));
            }
            let lowered_path = lower_expr(path, &mut ctx, sm, file)?;
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(&body, &mut lower_body, ctx.clone(), sm, file, opts)?;
            out.push(ir::Cmd::WithCwd {
                path: lowered_path,
                body: lower_body,
            });
            Ok(ctx_body)
        }
        ast::StmtKind::WithLog { path, append, body } => {
            let lowered_path = lower_expr(path, &mut ctx, sm, file)?;
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(&body, &mut lower_body, ctx.clone(), sm, file, opts)?;
            out.push(ir::Cmd::WithLog {
                path: lowered_path,
                append,
                body: lower_body,
            });
            Ok(ctx_body)
        }
        ast::StmtKind::Cd { path } => {
            out.push(ir::Cmd::Cd(lower_expr(path, &mut ctx, sm, file)?));
            Ok(ctx)
        }
        ast::StmtKind::Sh(expr) => {
            out.push(ir::Cmd::Raw(lower_expr(expr, &mut ctx, sm, file)?, loc));
            Ok(ctx)
        }
        ast::StmtKind::ShBlock(lines) => {
            for s in lines {
                out.push(ir::Cmd::RawLine { line: s, loc: loc.clone() });
            }
            Ok(ctx)
        }
        ast::StmtKind::Call { name, args } => {
            if name == "save_envfile" {
                if args.len() != 2 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "save_envfile() requires exactly 2 arguments (path, env_blob)",
                        stmt.span,
                    )));
                }
                let mut iter = args.into_iter();
                let path = lower_expr(iter.next().unwrap(), &mut ctx, sm, file)?;
                let env = lower_expr(iter.next().unwrap(), &mut ctx, sm, file)?;
                out.push(ir::Cmd::SaveEnvfile { path, env });
            } else if name == "load_envfile" {
                return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "load_envfile() returns a value; use it in an expression (e.g., let m = load_envfile(\"env.meta\"))", stmt.span)));
            } else if name == "which" {
                return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "which() returns a value; use it in an expression (e.g., let p = which(\"cmd\"))", stmt.span)));
            } else if name == "require" {
                if args.len() != 1 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "require() requires exactly one argument (cmd_list)",
                        stmt.span,
                    )));
                }
                let arg = &args[0];
                if let ast::ExprKind::List(elems) = &arg.node {
                    let mut valid_cmds = Vec::new();
                    for e in elems {
                        valid_cmds.push(lower_expr(e.clone(), &mut ctx, sm, file)?);
                    }
                    out.push(ir::Cmd::Require(valid_cmds));
                } else {
                    return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "require() expects a list literal", arg.span)));
                }
            } else if name == "append_file" {
                if args.len() != 2 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "append_file() requires exactly 2 arguments (path, content)",
                        stmt.span,
                    )));
                }
                let mut iter = args.into_iter();
                let path = lower_expr(iter.next().unwrap(), &mut ctx, sm, file)?;
                let content = lower_expr(iter.next().unwrap(), &mut ctx, sm, file)?;
                out.push(ir::Cmd::WriteFile {
                    path,
                    content,
                    append: true,
                });
            } else if name == "write_file" {
                if args.len() < 2 || args.len() > 3 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "write_file() requires 2 or 3 arguments (path, content, [append])",
                        stmt.span,
                    )));
                }
                let mut iter = args.into_iter();
                let path = lower_expr(iter.next().unwrap(), &mut ctx, sm, file)?;
                let content = lower_expr(iter.next().unwrap(), &mut ctx, sm, file)?;
                let append = if iter.len() > 0 {
                    let arg = iter.next().unwrap();
                    if let ast::ExprKind::Bool(b) = arg.node {
                        b
                    } else {
                        return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "write_file: append must be boolean literal", arg.span)));
                    }
                } else {
                    false
                };
                out.push(ir::Cmd::WriteFile {
                    path,
                    content,
                    append,
                });
            } else if name == "read_file" {
                return Err(CompileError::new(sm.format_diagnostic(
                    file,
                    opts.diag_base_dir.as_deref(),
                    "read_file() returns a value; use it in an expression (e.g., let c = read_file(\"file.txt\"))",
                    stmt.span,
                )));
            } else if name == "home" {
                return Err(CompileError::new(sm.format_diagnostic(
                    file,
                    opts.diag_base_dir.as_deref(),
                    "home() returns a value; use it in an expression",
                    stmt.span,
                )));
            } else if name == "path_join" {
                return Err(CompileError::new(sm.format_diagnostic(
                    file,
                    opts.diag_base_dir.as_deref(),
                    "path_join() returns a value; use it in an expression",
                    stmt.span,
                )));
            } else if matches!(name.as_str(), "log_info" | "log_warn" | "log_error") {
                let lvl = match name.as_str() {
                    "log_info" => ir::LogLevel::Info,
                    "log_warn" => ir::LogLevel::Warn,
                    "log_error" => ir::LogLevel::Error,
                    _ => unreachable!(),
                };
                
                if args.is_empty() || args.len() > 2 {
                     return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        format!("{}() requires 1 or 2 arguments (msg, [timestamp])", name).as_str(),
                        stmt.span
                    )));
                }

                let mut iter = args.into_iter();
                let msg = lower_expr(iter.next().unwrap(), &mut ctx, sm, file)?;
                let timestamp = if iter.len() > 0 {
                    let arg = iter.next().unwrap();
                    if let ast::ExprKind::Bool(b) = arg.node {
                        b
                    } else {
                         return Err(CompileError::new(sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            format!("{}() second argument must be a boolean literal", name).as_str(),
                            arg.span
                        )));
                    }
                } else {
                    false
                };

                out.push(ir::Cmd::Log {
                    level: lvl,
                    msg,
                    timestamp,
                });
            } else {
                // Generic call (Command)
                let mut cmd_args = vec![ir::Val::Literal(name)];
                for a in args {
                    cmd_args.push(lower_expr(a, &mut ctx, sm, file)?);
                }
                out.push(ir::Cmd::Exec {
                    args: cmd_args,
                    allow_fail: false,
                    loc,
                });
            }
            Ok(ctx)
        }
        ast::StmtKind::Subshell { body } => {
            let mut lower_body = Vec::new();
            lower_block(&body, &mut lower_body, ctx.clone(), sm, file, opts)?;
            out.push(ir::Cmd::Subshell { body: lower_body });
            Ok(ctx)
        }
        ast::StmtKind::Group { body } => {
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(&body, &mut lower_body, ctx.clone(), sm, file, opts)?;
            out.push(ir::Cmd::Group { body: lower_body });
            Ok(ctx_body)
        }
        ast::StmtKind::WithRedirect {
            stdout,
            stderr,
            stdin,
            body,
        } => {
            let mut lowered_body = Vec::new();
            let ctx_body = lower_block(&body, &mut lowered_body, ctx.clone(), sm, file, opts)?;

            let lower_output_target = |t: ast::RedirectOutputTarget, c: &mut LoweringContext| -> Result<ir::RedirectOutputTarget, CompileError> {
                 Ok(match t {
                    ast::RedirectOutputTarget::File { path, append } => ir::RedirectOutputTarget::File {
                        path: lower_expr(path, c, sm, file)?,
                        append,
                    },
                    ast::RedirectOutputTarget::ToStdout => ir::RedirectOutputTarget::ToStdout,
                    ast::RedirectOutputTarget::ToStderr => ir::RedirectOutputTarget::ToStderr,
                    ast::RedirectOutputTarget::InheritStdout => ir::RedirectOutputTarget::InheritStdout,
                    ast::RedirectOutputTarget::InheritStderr => ir::RedirectOutputTarget::InheritStderr,
                })
            };

            let lower_input_target = |t: ast::RedirectInputTarget, c: &mut LoweringContext| -> Result<ir::RedirectInputTarget, CompileError> {
                 Ok(match t {
                    ast::RedirectInputTarget::File { path } => ir::RedirectInputTarget::File {
                        path: lower_expr(path, c, sm, file)?,
                    },
                    ast::RedirectInputTarget::HereDoc { content } => ir::RedirectInputTarget::HereDoc { content },
                })
            };

            let lower_output_vec = |targets: Vec<Spanned<ast::RedirectOutputTarget>>, c: &mut LoweringContext| -> Result<Vec<ir::RedirectOutputTarget>, CompileError> {
                targets.into_iter().map(|spanned| lower_output_target(spanned.node, c)).collect()
            };

            out.push(ir::Cmd::WithRedirect {
                stdout: stdout.map(|targets| lower_output_vec(targets, &mut ctx)).transpose()?,
                stderr: stderr.map(|targets| lower_output_vec(targets, &mut ctx)).transpose()?,
                stdin: stdin.map(|t| lower_input_target(t, &mut ctx)).transpose()?,
                body: lowered_body,
            });
            Ok(ctx_body)
        }
        ast::StmtKind::Spawn { stmt } => {
            let mut lower_cmds = Vec::new();
            lower_stmt(*stmt, &mut lower_cmds, ctx.clone(), sm, file, opts)?;

            if lower_cmds.len() == 1 {
                out.push(ir::Cmd::Spawn(Box::new(lower_cmds.remove(0))));
            } else {
                out.push(ir::Cmd::Spawn(Box::new(ir::Cmd::Group {
                    body: lower_cmds,
                })));
            }
            Ok(ctx)
        }
        ast::StmtKind::Wait(expr) => {
            out.push(ir::Cmd::Wait(
                expr.map(|e| lower_expr(e, &mut ctx, sm, file)).transpose()?,
            ));
            Ok(ctx)
        }
        ast::StmtKind::TryCatch {
            try_body,
            catch_body,
        } => {
            let mut lower_try = Vec::new();
            let ctx_try = lower_block(&try_body, &mut lower_try, ctx.clone(), sm, file, opts)?;

            let mut lower_catch = Vec::new();
            let ctx_catch = lower_block(&catch_body, &mut lower_catch, ctx.clone(), sm, file, opts)?;

            out.push(ir::Cmd::TryCatch {
                try_body: lower_try,
                catch_body: lower_catch,
            });
            Ok(ctx_try.intersection(&ctx_catch))
        }
        ast::StmtKind::Export { name, value } => {
            out.push(ir::Cmd::Export {
                name,
                value: value.map(|v| lower_expr(v, &mut ctx, sm, file)).transpose()?,
            });
            Ok(ctx)
        }
        ast::StmtKind::Unset { name } => {
            out.push(ir::Cmd::Unset(name));
            Ok(ctx)
        }
        ast::StmtKind::Source { path } => {
            out.push(ir::Cmd::Source(lower_expr(path, &mut ctx, sm, file)?));
            Ok(ctx)
        }
        ast::StmtKind::Exec(args) => {
            out.push(ir::Cmd::ExecReplace(
                args.into_iter()
                    .map(|a| lower_expr(a, &mut ctx, sm, file))
                    .collect::<Result<Vec<_>, _>>()?,
                loc,
            ));
            Ok(ctx)
        }
        ast::StmtKind::Set { target, value } => {
            match target {
                ast::LValue::Var(name) => {
                    out.push(ir::Cmd::Assign(
                        name,
                        lower_expr(value, &mut ctx, sm, file)?,
                        loc,
                    ));
                }
                ast::LValue::Env(name) => {
                    if matches!(&value.node, ast::ExprKind::List(_) | ast::ExprKind::Args) {
                        return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "set env.<NAME> requires a scalar string/number; lists/args are not supported", stmt.span)));
                    }

                    let val = lower_expr(value, &mut ctx, sm, file)?;

                    if matches!(&val, ir::Val::List(_) | ir::Val::Args) {
                        return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "set env.<NAME> requires a scalar string/number; lists/args are not supported", stmt.span)));
                    }

                    out.push(ir::Cmd::Export {
                        name,
                        value: Some(val),
                    });
                }
            }
            Ok(ctx)
        }
        ast::StmtKind::ForMap {
            key_var,
            val_var,
            map,
            body,
        } => {
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(&body, &mut lower_body, ctx.clone(), sm, file, opts)?;
            out.push(ir::Cmd::ForMap {
                key_var,
                val_var,
                map,
                body: lower_body,
            });
            Ok(ctx.intersection(&ctx_body))
        }

    }
}

fn lower_run_call_args<'a>(
    run_call: &ast::RunCall,
    ctx: &mut LoweringContext<'a>,
    sm: &SourceMap,
    file: &str,
    opts: &'a LowerOptions,
) -> Result<(Vec<ir::Val>, bool), CompileError> {
    let lowered_args = run_call
        .args
        .iter()
        .map(|a| lower_expr(a.clone(), ctx, sm, file))
        .collect::<Result<Vec<_>, _>>()?;

    let mut allow_fail = false;
    for opt in &run_call.options {
        if opt.name == "shell" {
             return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "shell option is not supported in run(...); use sh(...) for raw shell code", opt.span)));
        } else if opt.name == "allow_fail" {
             if let ast::ExprKind::Bool(b) = opt.value.node {
                 allow_fail = b;
             } else {
                 return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "allow_fail must be true/false", opt.value.span)));
             }
        } else {
             return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), &format!("Unknown option {:?}", opt.name), opt.span)));
        }
    }
    
    Ok((lowered_args, allow_fail))
}

fn lower_sudo_command<'a>(
    args: Vec<ast::Expr>,
    options: Vec<ast::RunOption>,
    ctx: &mut LoweringContext<'a>,
    sm: &SourceMap,
    file: &str,
) -> Result<(Vec<ir::Val>, Option<crate::span::Span>), CompileError> {
    let opts = ctx.opts;

    // Use SudoSpec to parse/validate options
    // Note: SudoSpec::from_options uses (String, Span), we need compatibility
    let spec = SudoSpec::from_options(&options)
        .map_err(|(msg, span)| CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), &msg, span)))?;

    let mut argv = Vec::new();
    argv.push(ir::Val::Literal("sudo".to_string()));

    // Use deterministic flag generation from spec
    let flags = spec.to_flags_argv();
    for flag in flags {
        argv.push(ir::Val::Literal(flag));
    }

    // Add positional args (command + args), lowered
    for arg in args {
        argv.push(lower_expr(arg, ctx, sm, file)?);
    }
    
    // Extract allow_fail check from spec (it handles the boolean logic)
    // Note spec maps (bool, span).
    // We return the span if allow_fail is present (true or false doesn't matter for the error check in Expr, 
    // but usually user=true is the trigger. Wait, spec says "is_some() -> Err".
    // We traverse options to find the name span for correct highlighting
    let allow_fail_name_span = options.iter()
        .find(|o| o.name == "allow_fail")
        .map(|o| o.span);

    Ok((argv, allow_fail_name_span))
}


fn lower_expr<'a>(e: ast::Expr, ctx: &mut LoweringContext<'a>, sm: &SourceMap, file: &str) -> Result<ir::Val, CompileError> {
    let opts = ctx.opts; // Get opts from context for diagnostic formatting
    match e.node {
        ast::ExprKind::Literal(s) => Ok(ir::Val::Literal(s)),
        ast::ExprKind::Var(s) => {
            if ctx.is_bool_var(&s) {
                Ok(ir::Val::BoolVar(s))
            } else {
                Ok(ir::Val::Var(s))
            }
        }
        ast::ExprKind::Concat(l, r) => Ok(ir::Val::Concat(
            Box::new(lower_expr(*l, ctx, sm, file)?),
            Box::new(lower_expr(*r, ctx, sm, file)?),
        )),
        ast::ExprKind::Arith { left, op, right } => {
            if matches!(op, ast::ArithOp::Add) {
                let l_is_lit = matches!(left.node, ast::ExprKind::Literal(_));
                let r_is_lit = matches!(right.node, ast::ExprKind::Literal(_));
                if l_is_lit || r_is_lit {
                    return Ok(ir::Val::Concat(
                        Box::new(lower_expr(*left, ctx, sm, file)?),
                        Box::new(lower_expr(*right, ctx, sm, file)?),
                    ));
                }
            }

            let op = match op {
                ast::ArithOp::Add => ir::ArithOp::Add,
                ast::ArithOp::Sub => ir::ArithOp::Sub,
                ast::ArithOp::Mul => ir::ArithOp::Mul,
                ast::ArithOp::Div => ir::ArithOp::Div,
                ast::ArithOp::Mod => ir::ArithOp::Mod,
            };
            Ok(ir::Val::Arith {
                left: Box::new(lower_expr(*left, ctx, sm, file)?),
                op,
                right: Box::new(lower_expr(*right, ctx, sm, file)?),
            })
        }
        ast::ExprKind::Compare { left, op, right } => {
            // Check for boolean literal comparisons (eq/neq with true/false_
            let get_bool = |e: &ast::Expr| if let ast::ExprKind::Bool(b) = e.node { Some(b) } else { None };
            let l_bool = get_bool(&left);
            let r_bool = get_bool(&right);

            if (l_bool.is_some() || r_bool.is_some()) && matches!(op, ast::CompareOp::Eq | ast::CompareOp::NotEq) {
                // If one side is a bool literal and op is Eq/NotEq, canonicalize to unary conditional
                // Supported cases: true == pred, false == pred, pred == true, pred == false (and !=)
                
                // Identify which side is literal and which is predicate
                let (lit_val, pred_expr) = if let Some(b) = l_bool {
                     (b, *right)
                } else {
                     (r_bool.unwrap(), *left)
                };

                let pred_val = lower_expr(pred_expr, ctx, sm, file)?;
                
                // Logic table:
                // Eq:   true == pred -> pred
                // Eq:   false == pred -> !pred
                // NotEq: true != pred -> !pred
                // NotEq: false != pred -> pred
                
                let is_eq = matches!(op, ast::CompareOp::Eq);
                match (is_eq, lit_val) {
                    (true, true) => Ok(pred_val),      // pred == true
                    (true, false) => Ok(ir::Val::Not(Box::new(pred_val))), // pred == false
                    (false, true) => Ok(ir::Val::Not(Box::new(pred_val))), // pred != true
                    (false, false) => Ok(pred_val),    // pred != false
                }
            } else {
                let op = match op {
                    ast::CompareOp::Eq => ir::CompareOp::Eq,
                    ast::CompareOp::NotEq => ir::CompareOp::NotEq,
                    ast::CompareOp::Lt => ir::CompareOp::Lt,
                    ast::CompareOp::Le => ir::CompareOp::Le,
                    ast::CompareOp::Gt => ir::CompareOp::Gt,
                    ast::CompareOp::Ge => ir::CompareOp::Ge,
                };
                Ok(ir::Val::Compare {
                    left: Box::new(lower_expr(*left, ctx, sm, file)?),
                    op,
                    right: Box::new(lower_expr(*right, ctx, sm, file)?),
                })
            }
        }
        ast::ExprKind::And(left, right) => Ok(ir::Val::And(
            Box::new(lower_expr(*left, ctx, sm, file)?),
            Box::new(lower_expr(*right, ctx, sm, file)?),
        )),
        ast::ExprKind::Or(left, right) => Ok(ir::Val::Or(
            Box::new(lower_expr(*left, ctx, sm, file)?),
            Box::new(lower_expr(*right, ctx, sm, file)?),
        )),
        ast::ExprKind::Not(expr) => Ok(ir::Val::Not(Box::new(lower_expr(*expr, ctx, sm, file)?))),
        ast::ExprKind::Exists(path) => Ok(ir::Val::Exists(Box::new(lower_expr(*path, ctx, sm, file)?))),
        ast::ExprKind::IsDir(path) => Ok(ir::Val::IsDir(Box::new(lower_expr(*path, ctx, sm, file)?))),
        ast::ExprKind::IsFile(path) => Ok(ir::Val::IsFile(Box::new(lower_expr(*path, ctx, sm, file)?))),
        ast::ExprKind::IsSymlink(path) => {
            Ok(ir::Val::IsSymlink(Box::new(lower_expr(*path, ctx, sm, file)?)))
        }
        ast::ExprKind::IsExec(path) => Ok(ir::Val::IsExec(Box::new(lower_expr(*path, ctx, sm, file)?))),
        ast::ExprKind::IsReadable(path) => {
            Ok(ir::Val::IsReadable(Box::new(lower_expr(*path, ctx, sm, file)?)))
        }
        ast::ExprKind::IsWritable(path) => {
            Ok(ir::Val::IsWritable(Box::new(lower_expr(*path, ctx, sm, file)?)))
        }
        ast::ExprKind::IsNonEmpty(path) => {
            Ok(ir::Val::IsNonEmpty(Box::new(lower_expr(*path, ctx, sm, file)?)))
        }
        ast::ExprKind::BoolStr(inner) => {
            Ok(ir::Val::BoolStr(Box::new(lower_expr(*inner, ctx, sm, file)?)))
        }
        ast::ExprKind::Len(expr) => Ok(ir::Val::Len(Box::new(lower_expr(*expr, ctx, sm, file)?))),
        ast::ExprKind::Arg(expr) => {
            let index_val = lower_expr(*expr, ctx, sm, file)?;
            
            // Optimize: if literal number >= 1, use Val::Arg(n) for direct $n expansion
            if let ir::Val::Number(n) = &index_val {
                if *n >= 1 {
                    Ok(ir::Val::Arg(*n))
                } else {
                    // Invalid literal (0 or would-be-negative): use dynamic path which returns empty
                    Ok(ir::Val::ArgDynamic(Box::new(index_val)))
                }
            } else {
                // Dynamic case: use helper function
                Ok(ir::Val::ArgDynamic(Box::new(index_val)))
            }
        }
        ast::ExprKind::Index { list, index } => Ok(ir::Val::Index {
            list: Box::new(lower_expr(*list, ctx, sm, file)?),
            index: Box::new(lower_expr(*index, ctx, sm, file)?),
        }),
        ast::ExprKind::Field { base, name } => {
            let b = lower_expr(*base, ctx, sm, file)?;

            match name.as_str() {
                "flags" => Ok(ir::Val::ArgsFlags(Box::new(b))),
                "positionals" => Ok(ir::Val::ArgsPositionals(Box::new(b))),
                "status" | "stdout" | "stderr" => {
                    if let ir::Val::Var(vname) = &b {
                        if ctx.run_results.contains(vname) {
                            Ok(ir::Val::Var(format!("{}__{}", vname, name)))
                        } else {
                            Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), format!(".{} is only valid on try_run() results (bind via let)", name).as_str(), e.span)))
                        }
                    } else {
                        Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), format!("Field access '{}' only supported on variables (e.g. r.status)", name).as_str(), e.span)))
                    }
                }
                _ => Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), format!("Unknown field '{}'. Supported: status, stdout, stderr, flags, positionals.", name).as_str(), e.span))),
            }
        }
        ast::ExprKind::Join { list, sep } => Ok(ir::Val::Join {
            list: Box::new(lower_expr(*list, ctx, sm, file)?),
            sep: Box::new(lower_expr(*sep, ctx, sm, file)?),
        }),
        ast::ExprKind::Count(inner) => Ok(ir::Val::Count(Box::new(lower_expr(*inner, ctx, sm, file)?))),
        ast::ExprKind::Bool(b) => Ok(ir::Val::Bool(b)),
        ast::ExprKind::Number(n) => Ok(ir::Val::Number(n)),
        ast::ExprKind::Command(args) => {
            let lowered_args = args
                .into_iter()
                .map(|a| lower_expr(a, ctx, sm, file))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ir::Val::Command(lowered_args))
        }
        ast::ExprKind::CommandPipe(segments) => {
            let lowered_segments = segments
                .into_iter()
                .map(|seg| {
                    seg.into_iter()
                        .map(|a| lower_expr(a, ctx, sm, file))
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ir::Val::CommandPipe(lowered_segments))
        }
        ast::ExprKind::List(exprs) => {
            let lowered_exprs = exprs
                .into_iter()
                .map(|e| lower_expr(e, ctx, sm, file))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ir::Val::List(lowered_exprs))
        }
        ast::ExprKind::Args => Ok(ir::Val::Args),
        ast::ExprKind::Status => Ok(ir::Val::Status),
        ast::ExprKind::Pid => Ok(ir::Val::Pid),
        ast::ExprKind::Env(inner) => Ok(ir::Val::Env(Box::new(lower_expr(*inner, ctx, sm, file)?))),
        ast::ExprKind::Uid => Ok(ir::Val::Uid),
        ast::ExprKind::Ppid => Ok(ir::Val::Ppid),
        ast::ExprKind::Pwd => Ok(ir::Val::Pwd),
        ast::ExprKind::SelfPid => Ok(ir::Val::SelfPid),
        ast::ExprKind::Argv0 => Ok(ir::Val::Argv0),
        ast::ExprKind::Argc => Ok(ir::Val::Argc),
        ast::ExprKind::EnvDot(name) => Ok(ir::Val::EnvDot(name)),
        ast::ExprKind::Input(e) => Ok(ir::Val::Input(Box::new(lower_expr(*e, ctx, sm, file)?))),
        ast::ExprKind::Confirm { prompt, default } => {
            let prompt_val = lower_expr(*prompt, ctx, sm, file)?;
            let default_bool = match default {
                None => false,
                Some(d) => {
                    if let ast::ExprKind::Bool(b) = d.node {
                        b
                    } else {
                        return Err(CompileError::new(sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            "confirm(default=...) must be a true/false literal",
                            d.span,
                        )));
                    }
                }
            };
            Ok(ir::Val::Confirm { prompt: Box::new(prompt_val), default: default_bool })
        }
        ast::ExprKind::Sudo { args, options } => {
            let (argv, allow_fail_span) = lower_sudo_command(args, options, ctx, sm, file)?;
            
            if let Some(span) = allow_fail_span {
                 return Err(CompileError::new(sm.format_diagnostic(
                    file,
                    opts.diag_base_dir.as_deref(),
                    "allow_fail is only valid on statement-form sudo(...); use capture(sudo(...), allow_fail=true) to allow failure during capture",
                    span,
                )));
            }
            
            Ok(ir::Val::Command(argv))
        }
        ast::ExprKind::Call { name, args } => {
            if name == "matches" {
                if args.len() != 2 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "matches() requires exactly 2 arguments (text, regex)",
                        e.span,
                    )));
                }
                let mut iter = args.into_iter();
                let text = Box::new(lower_expr(iter.next().unwrap(), ctx, sm, file)?);
                let regex = Box::new(lower_expr(iter.next().unwrap(), ctx, sm, file)?);
                Ok(ir::Val::Matches(text, regex))
            } else if name == "contains" {
                if args.len() != 2 {
                     return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "contains() requires exactly 2 arguments (list, value)",
                        e.span,
                    )));
                }
                let mut iter = args.into_iter();
                let list = Box::new(lower_expr(iter.next().unwrap(), ctx, sm, file)?);
                let needle = Box::new(lower_expr(iter.next().unwrap(), ctx, sm, file)?);
                Ok(ir::Val::Contains { list, needle })
            } else if name == "contains_line" {
                if args.len() != 2 {
                     return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "contains_line() requires exactly 2 arguments (text, needle)",
                        e.span,
                    )));
                }
                let mut iter = args.into_iter();
                let text = Box::new(lower_expr(iter.next().unwrap(), ctx, sm, file)?);
                let needle = Box::new(lower_expr(iter.next().unwrap(), ctx, sm, file)?);
                Ok(ir::Val::ContainsLine { text, needle })
            } else if name == "parse_args" {
                if !args.is_empty() {
                    return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "parse_args() takes no arguments", e.span)));
                }
                Ok(ir::Val::ParseArgs)
            } else if name == "load_envfile" {
                if args.len() != 1 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "load_envfile() requires exactly 1 argument (path)",
                        e.span,
                    )));
                }
                let path = lower_expr(args.into_iter().next().unwrap(), ctx, sm, file)?;
                Ok(ir::Val::LoadEnvfile(Box::new(path)))
            } else if name == "json_kv" {
                if args.len() != 1 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "json_kv() requires exactly 1 argument (pairs_blob)",
                        e.span,
                    )));
                }
                let blob = lower_expr(args.into_iter().next().unwrap(), ctx, sm, file)?;
                Ok(ir::Val::JsonKv(Box::new(blob)))
            } else if name == "which" {
                if args.len() != 1 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "which() requires exactly 1 argument (cmd)",
                        e.span,
                    )));
                }
                let arg = lower_expr(args.into_iter().next().unwrap(), ctx, sm, file)?;
                Ok(ir::Val::Which(Box::new(arg)))
            } else if name == "try_run" {
                return Err(CompileError::new(sm.format_diagnostic(
                    file,
                    opts.diag_base_dir.as_deref(),
                    "try_run() must be bound via let (e.g., let r = try_run(...))",
                    e.span,
                )));
            } else if name == "require" {
                return Err(CompileError::new(sm.format_diagnostic(
                    file,
                    opts.diag_base_dir.as_deref(),
                    "require() is a statement; use it as a standalone call",
                    e.span,
                )));
            } else if name == "read_file" {
                if args.len() != 1 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "read_file() requires exactly 1 argument (path)",
                        e.span,
                    )));
                }
                let arg = lower_expr(args.into_iter().next().unwrap(), ctx, sm, file)?;
                Ok(ir::Val::ReadFile(Box::new(arg)))
            } else if name == "write_file" {
                return Err(CompileError::new(sm.format_diagnostic(
                    file,
                    opts.diag_base_dir.as_deref(),
                    "write_file() is a statement, not an expression",
                    e.span,
                )));
            } else if name == "append_file" {
                return Err(CompileError::new(sm.format_diagnostic(
                    file,
                    opts.diag_base_dir.as_deref(),
                    "append_file() is a statement, not an expression",
                    e.span,
                )));
            } else if matches!(name.as_str(), "log_info" | "log_warn" | "log_error") {
                return Err(CompileError::new(sm.format_diagnostic(
                    file,
                    opts.diag_base_dir.as_deref(),
                    format!("{}() is a statement, not an expression", name).as_str(),
                    e.span,
                )));
            } else if name == "home" {
                if !args.is_empty() {
                    return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "home() takes no arguments", e.span)));
                }
                Ok(ir::Val::Home)
            } else if name == "path_join" {
                if args.is_empty() {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "path_join() requires at least 1 argument",
                        e.span,
                    )));
                }
                let lowered_args = args
                    .into_iter()
                    .map(|a| lower_expr(a, ctx, sm, file))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ir::Val::PathJoin(lowered_args))
            } else if name == "lines" {
                if args.len() != 1 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "lines() requires exactly 1 argument (text)",
                        e.span,
                    )));
                }
                let arg = lower_expr(args.into_iter().next().unwrap(), ctx, sm, file)?;
                Ok(ir::Val::Lines(Box::new(arg)))
            } else if name == "split" {
                if args.len() != 2 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "split() requires exactly 2 arguments (text, delimiter)",
                        e.span,
                    )));
                }
                let mut iter = args.into_iter();
                let s = lower_expr(iter.next().unwrap(), ctx, sm, file)?;
                let delim = lower_expr(iter.next().unwrap(), ctx, sm, file)?;
                Ok(ir::Val::Split {
                    s: Box::new(s),
                    delim: Box::new(delim),
                })
            } else if name == "save_envfile" {
                return Err(CompileError::new(sm.format_diagnostic(
                    file,
                    opts.diag_base_dir.as_deref(),
                    "save_envfile() is a statement; use it as a standalone call",
                    e.span,
                )));
            } else {
                let lowered_args = args
                    .into_iter()
                    .map(|a| lower_expr(a, ctx, sm, file))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ir::Val::Call {
                    name,
                    args: lowered_args,
                })
            }
        }
        ast::ExprKind::MapLiteral(entries) => {
            let lowered_entries = entries
                .into_iter()
                .map(|(k, v)| {
                     let v = lower_expr(v, ctx, sm, file)?;
                     Ok((k, v))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ir::Val::MapLiteral(lowered_entries))
        }
        ast::ExprKind::MapIndex { map, key } => Ok(ir::Val::MapIndex { map, key }),
        ast::ExprKind::Capture { expr, options } => {
            let expr_span = expr.span;
            let lowered_expr = lower_expr(*expr, ctx, sm, file)?;
            let mut allow_fail = false;
            let mut seen_allow_fail = false;
            for opt in options {
                if opt.name == "allow_fail" {
                    if seen_allow_fail {
                        return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "allow_fail specified more than once", opt.span)));
                    }
                    seen_allow_fail = true;
                    if let ast::ExprKind::Bool(b) = opt.value.node {
                        allow_fail = b;
                    } else {
                        return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "allow_fail must be true/false literal", opt.value.span)));
                    }
                } else {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        format!("Unknown option '{}'. Supported options: allow_fail", opt.name).as_str(),
                        opt.span,
                    )));
                }
            }

            if allow_fail && !ctx.in_let_rhs {
                 return Err(CompileError::new(sm.format_diagnostic(
                     file,
                     opts.diag_base_dir.as_deref(),
                     "capture(..., allow_fail=true) is only allowed in 'let' assignment (e.g. let res = capture(...))",
                     expr_span, // Use safe span
                 )));
            }

            Ok(ir::Val::Capture {
                value: Box::new(lowered_expr),
                allow_fail,
            })
        }
        ast::ExprKind::Sh { cmd, options } => {
            // Expression-form sh() only supports 'shell' option
            // (allow_fail is rejected at parse time with helpful message)
            let mut shell_expr = ast::Expr {
                node: ast::ExprKind::Literal("sh".to_string()),
                span: Span::new(0, 0),
            };
            let mut seen_shell = false;
            
            for opt in options {
                if opt.name == "shell" {
                    if seen_shell {
                        return Err(CompileError::new(sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            "shell specified more than once",
                            opt.span,
                        )));
                    }
                    seen_shell = true;
                    shell_expr = opt.value;
                } else {
                    // This shouldn't happen if parser is correct, but handle gracefully
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        &format!("unknown sh() option '{}' in expression context; only 'shell' is supported", opt.name),
                        opt.span,
                    )));
                }
            }
            
            // Build argv: [shell, "-c", cmd]
            let shell_val = lower_expr(shell_expr, ctx, sm, file)?;
            let dash_c_val = ir::Val::Literal("-c".to_string());
            let cmd_val = lower_expr(*cmd, ctx, sm, file)?;
            
            Ok(ir::Val::Command(vec![shell_val, dash_c_val, cmd_val]))
        }
    }
}
fn lower_sudo_call_args<'a>(
    run_call: &ast::RunCall,
    ctx: &mut LoweringContext<'a>,
    sm: &SourceMap,
    file: &str,
    opts: &'a LowerOptions,
) -> Result<(Vec<ir::Val>, bool), CompileError> {
    // Validate options via SudoSpec (validation only, duplicates parser check but safe)
    // Note: parser already validated, but we need to re-derive the flags deterministicly.
    // Parser constructed the AST as raw args/options.
    
    let spec = SudoSpec::from_options(&run_call.options)
        .map_err(|(msg, span)| CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), &msg, span)))?;

    let mut argv = Vec::new();
    argv.push(ir::Val::Literal("sudo".to_string()));

    // Deterministic flags
    for flag in spec.to_flags_argv() {
        argv.push(ir::Val::Literal(flag));
    }

    // Positional args
    for arg in &run_call.args {
        argv.push(lower_expr(arg.clone(), ctx, sm, file)?);
    }
    
    // Extract allow_fail boolean
    let allow_fail = spec.allow_fail.map(|(b, _)| b).unwrap_or(false);
    
    Ok((argv, allow_fail))
}
