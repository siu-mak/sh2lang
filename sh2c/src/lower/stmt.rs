use crate::ast::{self, Spanned};
use crate::ir;
use crate::span::SourceMap;
use crate::error::CompileError;
use super::{LoweringContext, LowerOptions, lower_block, resolve_span};
use super::expr::lower_expr;
use super::sudo::{lower_run_call_args, lower_sudo_call_args};


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
                    | "contains_line"
                    | "confirm"
            )
        }
        _ => false,
    }
}

/// Lower one AST statement into IR commands. Returns the updated context after this statement.
pub(super) fn lower_stmt<'a>(
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
                args, options: _ } = &value.node
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
                        .map(|a| lower_expr(a, out, &mut ctx, sm, file))
                        .collect::<Result<Vec<_>, _>>()?;
                    out.push(ir::Cmd::Assign(
                        name.node.clone(),
                        ir::Val::TryRun(lowered_args),
                        loc,
                    ));
                    ctx.insert(&name.node);
                    return Ok(ctx);
                }
            }
            // Check if RHS is a boolean expression and track it
            let is_bool = is_bool_expr(&value);
            
            ctx.in_let_rhs = true;
            let val_ir = lower_expr(value, out, &mut ctx, sm, file)?;
            ctx.in_let_rhs = false;
            
            let is_capture_result = matches!(&val_ir, ir::Val::Capture { allow_fail: true, .. });
            let is_list = matches!(&val_ir, ir::Val::List(_) | ir::Val::Split { .. } | ir::Val::Lines(_));

            out.push(ir::Cmd::Assign(
                name.node.clone(),
                val_ir,
                loc,
            ));
            ctx.remove(&name.node);
            if is_capture_result {
                ctx.insert(&name.node);
            }
            if is_list {
                ctx.insert_list_var(&name.node);
            }
            if is_bool {
                ctx.insert_bool_var(&name.node);
            }
            Ok(ctx)
        }

        ast::StmtKind::Run(run_call) => {
            let ir_args = run_call
                .args
                .into_iter()
                .map(|a| lower_expr(a, out, &mut ctx, sm, file))
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
                        return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "allow_fail must be a boolean literal", opt.value.span)));
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
            if let ast::ExprKind::Call { name, args: _, options: _ } = &e.node {
                if name == "split" {
                     return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "Cannot emit boolean/list value as string",
                        e.span,
                    )));
                }
            }
            let val = lower_expr(e.clone(), out, &mut ctx, sm, file)?;
            out.push(ir::Cmd::Print(val));
            Ok(ctx)
        }

        ast::StmtKind::PrintErr(e) => {
            let val = lower_expr(e.clone(), out, &mut ctx, sm, file)?;
            out.push(ir::Cmd::PrintErr(val));
            Ok(ctx)
        }
        ast::StmtKind::If {
            cond,
            then_body,
            elifs,
            else_body,
        } => {
            let cond_val = lower_expr(cond, out, &mut ctx, sm, file)?;

            let mut t_cmds = Vec::new();
            let ctx_then = lower_block(&then_body, &mut t_cmds, ctx.clone(), sm, file, opts)?;

            let mut lowered_elifs = Vec::new();
            let mut ctx_elifs = Vec::new();

            for elif in elifs {
                let mut body_cmds = Vec::new();
                let elif_cond = lower_expr(elif.cond.clone(), out, &mut ctx, sm, file)?; // Evaluate cond in original context
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
            let expr_val = lower_expr(expr, out, &mut ctx, sm, file)?;
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
            let cond_val = lower_expr(cond, out, &mut ctx, sm, file)?;
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(&body, &mut lower_body, ctx.clone(), sm, file, opts)?;
            out.push(ir::Cmd::While {
                cond: cond_val,
                body: lower_body,
            });
            Ok(ctx.intersection(&ctx_body))
        }
        ast::StmtKind::For { var, iterable, body } => {
            let ir_iterable = match iterable {
                ast::ForIterable::List(items) => {
                     let lowered_items = items
                        .into_iter()
                        .map(|i| lower_expr(i, out, &mut ctx, sm, file))
                        .collect::<Result<Vec<_>, _>>()?;
                     ir::ForIterable::List(lowered_items)
                },
                ast::ForIterable::Range(start, end) => {
                    let start_val = lower_expr(*start, out, &mut ctx, sm, file)?;
                    let end_val = lower_expr(*end, out, &mut ctx, sm, file)?;
                    ir::ForIterable::Range(start_val, end_val)
                }
                ast::ForIterable::StdinLines => ir::ForIterable::StdinLines,
                ast::ForIterable::Find0(spec) => {
                    let dir_val = match spec.dir {
                        Some(d) => lower_expr(d, out, &mut ctx, sm, file)?,
                        None => ir::Val::Literal(".".to_string()),
                    };
                    let name_val = match spec.name {
                        Some(n) => Some(Box::new(lower_expr(n, out, &mut ctx, sm, file)?)),
                        None => None,
                    };
                    let type_val = match spec.type_filter {
                        Some(t) => {
                            // Validate type is literal "f" or "d"
                            match &t.node {
                                ast::ExprKind::Literal(s) if s == "f" || s == "d" => {},
                                _ => {
                                    return Err(CompileError::new(sm.format_diagnostic(
                                        file,
                                        opts.diag_base_dir.as_deref(),
                                        "find0() type must be literal \"f\" (files) or \"d\" (directories)",
                                        t.span,
                                    )));
                                }
                            }
                            Some(Box::new(lower_expr(t, out, &mut ctx, sm, file)?))
                        }
                        None => None,
                    };
                    let maxdepth_val = match spec.maxdepth {
                        Some(m) => {
                            match &m.node {
                                // Strict validation: maxdepth must be a non-negative integer literal.
                                // Note: `ast::ExprKind::Number` contains a `u32`, so it is strictly non-negative.
                                // Negative values (like `-1`) are parsed as unary expressions (not `Number` literals)
                                // and are correctly rejected here by the catch-all arm.
                                ast::ExprKind::Number(_) => {
                                    // Valid: literal non-negative integer.
                                }
                                _ => {
                                    // Any expression (variable, calculation, negative value) is rejected.
                                    // We use the span of the value expression `m` to point exactly at the invalid argument.
                                    return Err(CompileError::new(sm.format_diagnostic(
                                        file,
                                        opts.diag_base_dir.as_deref(),
                                        "find0() maxdepth must be a non-negative integer literal",
                                        m.span,
                                    )));
                                }
                            }
                            Some(Box::new(lower_expr(m, out, &mut ctx, sm, file)?))
                        }
                        None => None,
                    };
                    ir::ForIterable::Find0 {
                        dir: Box::new(dir_val),
                        name: name_val,
                        type_filter: type_val,
                        maxdepth: maxdepth_val,
                    }
                }
            };
            
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(&body, &mut lower_body, ctx.clone(), sm, file, opts)?;

            out.push(ir::Cmd::For {
                var: var.node,
                iterable: ir_iterable,
                body: lower_body,
            });
            Ok(ctx.intersection(&ctx_body))
        }
        ast::StmtKind::Pipe(segments) => {
            // Check if the pipeline ends with an `each_line` segment
            let last_idx = segments.len() - 1;
            let mut each_line_seg = None;
            
            for (i, seg) in segments.iter().enumerate() {
                if let ast::PipeSegment::EachLine(var, body) = &seg.node {
                    if i != last_idx {
                        let msg = "each_line must be the last segment of a pipeline";
                        let loc = resolve_span(seg.span, sm, file, opts.diag_base_dir.as_deref());
                        return Err(CompileError::new(msg).with_location(loc));
                    }
                    each_line_seg = Some((var, body));
                }
            }

            if let Some((var, body)) = each_line_seg {
                // Determine producer segments (all but the last)
                let producer_segments = &segments[0..last_idx];
                if producer_segments.is_empty() {
                     let loc = resolve_span(segments[last_idx].span, sm, file, opts.diag_base_dir.as_deref());
                     return Err(CompileError::new("each_line requires a producer").with_location(loc));
                }
                
                // Lower the producer pipeline into a single Cmd
                // We synthesize a StmtKind::Pipe for the producer part and recursively lower it
                let mut producer_out = Vec::new();
                let producer_stmt = ast::Stmt {
                    node: ast::StmtKind::Pipe(producer_segments.to_vec()),
                    span: segments[0].span.merge(segments[last_idx-1].span),
                };
                
                // Note: producer runs in a subshell (pipeline), so we ignore its context changes
                lower_stmt(producer_stmt, &mut producer_out, ctx.clone(), sm, file, opts)?;
                
                let producer_cmd = if producer_out.len() == 1 {
                    producer_out.pop().unwrap()
                } else {
                    ir::Cmd::Group { body: producer_out }
                };

                // Lower the each_line body
                // The body runs in the current context (process substitution), so changes persist
                let mut body_cmds = Vec::new();
                
                // Track loop variable if we were doing strict var checking, but for now just register it locally
                // Note: `lower_block` consumes ctx and returns a new one.
                // We want the changes in body to propagate.
                // However, sh2 rules for `for` loops (and similar) usually mean the loop var is visible?
                // `each_line` var is scoped to the loop? 
                // In Bash `while read x; do ... done`, x remains set after loop.
                // If we want sh2 to be cleaner, maybe we scope it? 
                // But sh2 `for` loops leak loop vars too (classic shell behavior).
                // Let's stick to Bash behavior: variables set in body persist. Loop var persists.
                
                // We should add `var` to context so `try_run` checks know it exists?
                // But `ctx` tracks *defined* variables.
                // Yes, declare it defined.
                ctx.insert(&var.node);
                
                let ctx_after_body = lower_block(body, &mut body_cmds, ctx, sm, file, opts)?;
                
                out.push(ir::Cmd::PipeEachLine {
                    producer: Box::new(producer_cmd),
                    var: var.node.clone(),
                    body: body_cmds,
                });
                
                Ok(ctx_after_body)
            } else {
                // Optimization: if all segments are Run(...) or Sudo(...), use ir::Cmd::Pipe
                
                let all_cmds = segments.iter().all(|s| matches!(s.node, ast::PipeSegment::Run(_) | ast::PipeSegment::Sudo(_)));
    
                if all_cmds {
                    // Pure command pipeline optimization path
                    let mut lowered_segments = Vec::new();
                    for seg in segments {
                         match &seg.node {
                            ast::PipeSegment::Run(run_call) => {
                                 let (args, allow_fail) = lower_run_call_args(run_call, out, &mut ctx, sm, file, opts)?;
                                 lowered_segments.push((args, allow_fail));
                            }
                            ast::PipeSegment::Sudo(run_call) => {
                                 let (args, allow_fail) = lower_sudo_call_args(run_call, out, &mut ctx, sm, file, opts)?;
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
                                let (args, allow_fail) = lower_run_call_args(run_call, out, &mut seg_ctx, sm, file, opts)?;
                                
                                block_cmds.push(ir::Cmd::Exec {
                                    args,
                                    allow_fail,
                                    loc: seg_loc, 
                                });
                            }
                            ast::PipeSegment::Sudo(run_call) => {
                                let mut seg_ctx = ctx.clone();
                                let (args, allow_fail) = lower_sudo_call_args(run_call, out, &mut seg_ctx, sm, file, opts)?;
                                
                                block_cmds.push(ir::Cmd::Exec {
                                    args,
                                    allow_fail,
                                    loc: seg_loc, 
                                });
                            }
                            ast::PipeSegment::EachLine(..) => unreachable!("EachLine handled above"),
                        }
                        lower_segments.push(block_cmds);
                    }
                    out.push(ir::Cmd::PipeBlocks(lower_segments, loc));
                }
                Ok(ctx)
            }
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
            let val = e.map(|x| lower_expr(x, out, &mut ctx, sm, file)).transpose()?;
            out.push(ir::Cmd::Return(val));
            Ok(ctx)
        }
        ast::StmtKind::Exit(e) => {
            let val = e.map(|x| lower_expr(x, out, &mut ctx, sm, file)).transpose()?;
            out.push(ir::Cmd::Exit(val));
            Ok(ctx)
        }
        ast::StmtKind::WithEnv { bindings, body } => {
            let lowered_bindings = bindings
                .into_iter()
                .map(|(k, v)| {
                     let v = lower_expr(v, out, &mut ctx, sm, file)?;
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
            let lowered_path = lower_expr(path, out, &mut ctx, sm, file)?;
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(&body, &mut lower_body, ctx.clone(), sm, file, opts)?;
            out.push(ir::Cmd::WithCwd {
                path: lowered_path,
                body: lower_body,
            });
            Ok(ctx_body)
        }
        ast::StmtKind::WithLog { path, append, body } => {
            let lowered_path = lower_expr(path, out, &mut ctx, sm, file)?;
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
            let path_val = lower_expr(path, out, &mut ctx, sm, file)?;
            out.push(ir::Cmd::Cd(path_val));
            Ok(ctx)
        }
        ast::StmtKind::Sh(expr) => {
            if let ast::ExprKind::Sh { cmd, options } = &expr.node {
                // Extract options: shell=, args=, allow_fail= (no-op).
                let mut shell_expr: Option<&ast::Expr> = None;
                let mut args_expr: Option<&ast::CallOption> = None;

                for opt in options {
                    match opt.name.as_str() {
                        "shell" => shell_expr = Some(&opt.value),
                        "args" => args_expr = Some(opt),
                        "allow_fail" => {} // no-op: probe never fails-fast
                        _ => {} // parser already rejects unknown options
                    }
                }

                // Lower args= if present; must resolve to Val::Args.
                let args_val = if let Some(args_opt) = args_expr {
                    let val = lower_expr(args_opt.value.clone(), out, &mut ctx, sm, file)?;
                    if !matches!(val, ir::Val::Args) {
                        return Err(CompileError::new(sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            "args= must be args() or argv()",
                            args_opt.span,
                        )));
                    }
                    Some(val)
                } else {
                    None
                };

                // Determine cmd value based on shell selection.
                // Default shell ("sh" or unset): lower cmd directly as a string value.
                // The probe helper already invokes `bash -c` / `sh -c` at runtime.
                // Custom shell: wrap as Val::Command([shell, "-c", cmd]).
                let is_custom_shell = shell_expr.is_some_and(|e| {
                    !matches!(&e.node, ast::ExprKind::Literal(s) if s == "sh")
                });

                let cmd_val = if is_custom_shell {
                    let shell_val = lower_expr(shell_expr.unwrap().clone(), out, &mut ctx, sm, file)?;
                    let cmd_val = lower_expr(*cmd.clone(), out, &mut ctx, sm, file)?;
                    ir::Val::Command(vec![shell_val, ir::Val::Literal("-c".to_string()), cmd_val])
                } else {
                    lower_expr(*cmd.clone(), out, &mut ctx, sm, file)?
                };

                out.push(ir::Cmd::Raw { cmd: cmd_val, args: args_val, loc });
            } else {
                return Err(CompileError::new(sm.format_diagnostic(
                    file,
                    opts.diag_base_dir.as_deref(),
                    "internal: StmtKind::Sh expected ExprKind::Sh",
                    expr.span,
                )));
            }
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
                let path = lower_expr(iter.next().unwrap(), out, &mut ctx, sm, file)?;
                let env = lower_expr(iter.next().unwrap(), out, &mut ctx, sm, file)?;
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
                        valid_cmds.push(lower_expr(e.clone(), out, &mut ctx, sm, file)?);
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
                let path = lower_expr(iter.next().unwrap(), out, &mut ctx, sm, file)?;
                let content = lower_expr(iter.next().unwrap(), out, &mut ctx, sm, file)?;
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
                let path = lower_expr(iter.next().unwrap(), out, &mut ctx, sm, file)?;
                let content = lower_expr(iter.next().unwrap(), out, &mut ctx, sm, file)?;
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
                let msg = lower_expr(iter.next().unwrap(), out, &mut ctx, sm, file)?;
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
                    cmd_args.push(lower_expr(a, out, &mut ctx, sm, file)?);
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

            let lower_output_target = |t: ast::RedirectOutputTarget, out: &mut Vec<ir::Cmd>, c: &mut LoweringContext| -> Result<ir::RedirectOutputTarget, CompileError> {
                 Ok(match t {
                    ast::RedirectOutputTarget::File { path, append } => ir::RedirectOutputTarget::File {
                        path: lower_expr(path, out, c, sm, file)?,
                        append,
                    },
                    ast::RedirectOutputTarget::ToStdout => ir::RedirectOutputTarget::ToStdout,
                    ast::RedirectOutputTarget::ToStderr => ir::RedirectOutputTarget::ToStderr,
                    ast::RedirectOutputTarget::InheritStdout => ir::RedirectOutputTarget::InheritStdout,
                    ast::RedirectOutputTarget::InheritStderr => ir::RedirectOutputTarget::InheritStderr,
                })
            };

            let lower_input_target = |t: ast::RedirectInputTarget, out: &mut Vec<ir::Cmd>, c: &mut LoweringContext| -> Result<ir::RedirectInputTarget, CompileError> {
                 Ok(match t {
                    ast::RedirectInputTarget::File { path } => ir::RedirectInputTarget::File {
                        path: lower_expr(path, out, c, sm, file)?,
                    },
                    ast::RedirectInputTarget::HereDoc { content } => ir::RedirectInputTarget::HereDoc { content },
                })
            };

            let lower_output_vec = |targets: Vec<Spanned<ast::RedirectOutputTarget>>, out: &mut Vec<ir::Cmd>, c: &mut LoweringContext| -> Result<Vec<ir::RedirectOutputTarget>, CompileError> {
                targets.into_iter().map(|spanned| lower_output_target(spanned.node, out, c)).collect()
            };

            let stdout_val = stdout.map(|targets| lower_output_vec(targets, out, &mut ctx)).transpose()?;
            let stderr_val = stderr.map(|targets| lower_output_vec(targets, out, &mut ctx)).transpose()?;
            let stdin_val = stdin.map(|t| lower_input_target(t, out, &mut ctx)).transpose()?;
            
            out.push(ir::Cmd::WithRedirect {
                stdout: stdout_val,
                stderr: stderr_val,
                stdin: stdin_val,
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
            let wait_val = expr.map(|e| lower_expr(e, out, &mut ctx, sm, file)).transpose()?;
            out.push(ir::Cmd::Wait(wait_val));
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
            let export_val = value.map(|v| lower_expr(v, out, &mut ctx, sm, file)).transpose()?;
            out.push(ir::Cmd::Export {
                name,
                value: export_val,
            });
            Ok(ctx)
        }
        ast::StmtKind::Unset { name } => {
            out.push(ir::Cmd::Unset(name));
            Ok(ctx)
        }
        ast::StmtKind::Source { path } => {
            let source_val = lower_expr(path, out, &mut ctx, sm, file)?;
            out.push(ir::Cmd::Source(source_val));
            Ok(ctx)
        }
        ast::StmtKind::Exec(args) => {
            let exec_args = args.into_iter()
                .map(|a| lower_expr(a, out, &mut ctx, sm, file))
                .collect::<Result<Vec<_>, _>>()?;
            out.push(ir::Cmd::ExecReplace(exec_args, loc));
            Ok(ctx)
        }
        ast::StmtKind::Set { target, value } => {
            match target {
                ast::LValue::Var(name) => {
                    let val = lower_expr(value, out, &mut ctx, sm, file)?;
                    
                    // List Inference
                    let is_list = match &val {
                        ir::Val::List(_) | ir::Val::Split { .. } | ir::Val::Lines(_) => true,
                        ir::Val::Var(n) => ctx.is_list_var(n),
                        _ => false,
                    };
                    
                    if is_list {
                        ctx.insert_list_var(&name.node);
                    }

                    out.push(ir::Cmd::Assign(
                        name.node.clone(),
                        val,
                        loc,
                    ));
                }
                ast::LValue::Env(name) => {
                    if matches!(&value.node, ast::ExprKind::List(_) | ast::ExprKind::Args) {
                        return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "set env.<NAME> requires a scalar string/number; lists/args are not supported", stmt.span)));
                    }

                    let val = lower_expr(value, out, &mut ctx, sm, file)?;

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
                key_var: key_var.node,
                val_var: val_var.node,
                map,
                body: lower_body,
            });
            Ok(ctx.intersection(&ctx_body))
        }
        ast::StmtKind::QualifiedCall { .. } => {
            unreachable!("QualifiedCall should be resolved by loader before lowering")
        }
    }
}
