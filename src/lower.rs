use crate::ast;
use crate::ir;
use crate::span::SourceMap;
use crate::span::Span;
use std::collections::HashSet;

#[derive(Clone, Debug)]
struct LoweringContext<'a> {
    run_results: HashSet<String>,
    opts: &'a LowerOptions,
}

impl<'a> LoweringContext<'a> {
    fn new(opts: &'a LowerOptions) -> Self {
        Self {
            run_results: HashSet::new(),
            opts,
        }
    }

    fn insert(&mut self, name: &str) {
        self.run_results.insert(name.to_string());
    }

    fn remove(&mut self, name: &str) {
        self.run_results.remove(name);
    }

    fn intersection(&self, other: &Self) -> Self {
        let run_results = self
            .run_results
            .intersection(&other.run_results)
            .cloned()
            .collect();
        Self {
            run_results,
            opts: self.opts,
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
pub fn lower(p: ast::Program) -> Vec<ir::Function> {
    lower_with_options(p, &LowerOptions::default())
}

pub fn lower_with_options(p: ast::Program, opts: &LowerOptions) -> Vec<ir::Function> {
    let has_main = p.functions.iter().any(|f| f.name == "main");
    let has_top_level = !p.top_level.is_empty();

    let entry_file = &p.entry_file;
    let maps = &p.source_maps;

    let entry_sm = maps
        .get(entry_file)
        .expect("Missing source map for entry file");

    let mut ir_funcs = Vec::new();

    if has_top_level {
        if has_main {
            let snippet = entry_sm.format_diagnostic(entry_file, opts.diag_base_dir.as_deref(), "Top-level statements are not allowed when `func main` is defined; move statements into main or remove main to use implicit main.", p.span);
            panic!("{}", snippet);
        }
        // Synthesize main
        let main_func = ast::Function {
            name: "main".to_string(),
            params: vec![],
            body: p.top_level,
            span: p.span,
            file: entry_file.clone(),
        };

        for f in p.functions {
            let sm = maps.get(&f.file).expect("Missing source map");
            ir_funcs.push(lower_function(f, sm, opts));
        }

        let sm = maps.get(&main_func.file).expect("Missing source map");
        ir_funcs.push(lower_function(main_func, sm, opts));
    } else {
        if !has_main {
            panic!("No entrypoint: define `func main()` or add top-level statements.");
        }
        for f in p.functions {
            let sm = maps.get(&f.file).expect("Missing source map");
            ir_funcs.push(lower_function(f, sm, opts));
        }
    }

    ir_funcs
}

/// Lower a single function
fn lower_function(f: ast::Function, sm: &SourceMap, opts: &LowerOptions) -> ir::Function {
    let mut body = Vec::new();
    let mut ctx = LoweringContext::new(opts);

    for stmt in f.body {
        ctx = lower_stmt(stmt, &mut body, ctx, sm, &f.file, opts);
    }

    ir::Function {
        name: f.name,
        params: f.params,
        commands: body,
        file: f.file,
    }
}

/// Helper to lower a block of statements sequentially
fn lower_block<'a>(
    stmts: Vec<ast::Stmt>,
    out: &mut Vec<ir::Cmd>,
    mut ctx: LoweringContext<'a>,
    sm: &SourceMap,
    file: &str,
    opts: &'a LowerOptions,
) -> LoweringContext<'a> {
    for stmt in stmts {
        ctx = lower_stmt(stmt, out, ctx, sm, file, opts);
    }
    ctx
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

/// Lower one AST statement into IR commands. Returns the updated context after this statement.
fn lower_stmt<'a>(
    stmt: ast::Stmt,
    out: &mut Vec<ir::Cmd>,
    mut ctx: LoweringContext<'a>,
    sm: &SourceMap,
    file: &str,
    opts: &'a LowerOptions,
) -> LoweringContext<'a> {
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
                        panic!(
                            "{}",
                            sm.format_diagnostic(
                                file,
                                opts.diag_base_dir.as_deref(),
                                "try_run() requires at least 1 argument (cmd)",
                                value.span
                            )
                        );
                    }
                    let lowered_args = args
                        .clone()
                        .into_iter()
                        .map(|a| lower_expr(a, &mut ctx, sm, file))
                        .collect();
                    out.push(ir::Cmd::Assign(
                        name.clone(),
                        ir::Val::TryRun(lowered_args),
                        loc,
                    ));
                    ctx.insert(&name);
                    return ctx;
                }
            }
            out.push(ir::Cmd::Assign(
                name.clone(),
                lower_expr(value, &mut ctx, sm, file),
                loc,
            ));
            ctx.remove(&name);
            ctx
        }

        ast::StmtKind::Run(run_call) => {
            let ir_args = run_call
                .args
                .into_iter()
                .map(|a| lower_expr(a, &mut ctx, sm, file))
                .collect();
            out.push(ir::Cmd::Exec {
                args: ir_args,
                allow_fail: run_call.allow_fail,
                loc,
            });
            ctx
        }

        ast::StmtKind::Print(e) => {
            out.push(ir::Cmd::Print(lower_expr(e, &mut ctx, sm, file)));
            ctx
        }

        ast::StmtKind::PrintErr(e) => {
            out.push(ir::Cmd::PrintErr(lower_expr(e, &mut ctx, sm, file)));
            ctx
        }
        ast::StmtKind::If {
            cond,
            then_body,
            elifs,
            else_body,
        } => {
            let cond_val = lower_expr(cond, &mut ctx, sm, file);

            let mut t_cmds = Vec::new();
            let ctx_then = lower_block(then_body, &mut t_cmds, ctx.clone(), sm, file, opts);

            let mut lowered_elifs = Vec::new();
            let mut ctx_elifs = Vec::new();

            for elif in elifs {
                let mut body_cmds = Vec::new();
                let elif_cond = lower_expr(elif.cond, &mut ctx, sm, file); // Evaluate cond in original context
                let ctx_elif = lower_block(elif.body, &mut body_cmds, ctx.clone(), sm, file, opts);
                lowered_elifs.push((elif_cond, body_cmds));
                ctx_elifs.push(ctx_elif);
            }

            let mut e_cmds = Vec::new();
            let ctx_else = if let Some(body) = else_body {
                lower_block(body, &mut e_cmds, ctx.clone(), sm, file, opts)
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
            final_ctx
        }
        ast::StmtKind::Case { expr, arms } => {
            let expr_val = lower_expr(expr, &mut ctx, sm, file);
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

                let ctx_arm = lower_block(arm.body, &mut body_cmds, ctx.clone(), sm, file, opts);

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
                return ctx;
            }

            let mut final_ctx = arm_ctxs[0].clone();
            for c in arm_ctxs.iter().skip(1) {
                final_ctx = final_ctx.intersection(c);
            }

            if !has_wildcard {
                final_ctx = final_ctx.intersection(&ctx);
            }

            final_ctx
        }
        ast::StmtKind::While { cond, body } => {
            let cond_val = lower_expr(cond, &mut ctx, sm, file);
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(body, &mut lower_body, ctx.clone(), sm, file, opts);

            out.push(ir::Cmd::While {
                cond: cond_val,
                body: lower_body,
            });
            ctx.intersection(&ctx_body)
        }
        ast::StmtKind::For { var, items, body } => {
            let lowered_items = items
                .into_iter()
                .map(|i| lower_expr(i, &mut ctx, sm, file))
                .collect();
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(body, &mut lower_body, ctx.clone(), sm, file, opts);

            out.push(ir::Cmd::For {
                var,
                items: lowered_items,
                body: lower_body,
            });
            ctx.intersection(&ctx_body)
        }
        ast::StmtKind::Pipe(segments) => {
            let mut lowered_segments = Vec::new();
            for run_call in segments {
                let lowered_args = run_call
                    .args
                    .into_iter()
                    .map(|a| lower_expr(a, &mut ctx, sm, file))
                    .collect();
                lowered_segments.push((lowered_args, run_call.allow_fail));
            }
            out.push(ir::Cmd::Pipe(lowered_segments, loc));
            ctx
        }
        ast::StmtKind::Break => {
            out.push(ir::Cmd::Break);
            ctx
        }
        ast::StmtKind::Continue => {
            out.push(ir::Cmd::Continue);
            ctx
        }
        ast::StmtKind::Return(e) => {
            out.push(ir::Cmd::Return(
                e.map(|x| lower_expr(x, &mut ctx, sm, file)),
            ));
            ctx
        }
        ast::StmtKind::Exit(e) => {
            out.push(ir::Cmd::Exit(e.map(|x| lower_expr(x, &mut ctx, sm, file))));
            ctx
        }
        ast::StmtKind::WithEnv { bindings, body } => {
            let lowered_bindings = bindings
                .into_iter()
                .map(|(k, v)| (k, lower_expr(v, &mut ctx, sm, file)))
                .collect();
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(body, &mut lower_body, ctx.clone(), sm, file, opts);
            out.push(ir::Cmd::WithEnv {
                bindings: lowered_bindings,
                body: lower_body,
            });
            ctx_body
        }
        ast::StmtKind::AndThen { left, right } => {
            let mut lower_left = Vec::new();
            let ctx_left = lower_block(left, &mut lower_left, ctx.clone(), sm, file, opts);

            let mut lower_right = Vec::new();
            let ctx_right = lower_block(right, &mut lower_right, ctx_left.clone(), sm, file, opts);

            out.push(ir::Cmd::AndThen {
                left: lower_left,
                right: lower_right,
            });
            ctx_left.intersection(&ctx_right)
        }
        ast::StmtKind::OrElse { left, right } => {
            let mut lower_left = Vec::new();
            let ctx_left = lower_block(left, &mut lower_left, ctx.clone(), sm, file, opts);

            let mut lower_right = Vec::new();
            let ctx_right = lower_block(right, &mut lower_right, ctx_left.clone(), sm, file, opts);

            out.push(ir::Cmd::OrElse {
                left: lower_left,
                right: lower_right,
            });
            ctx_left.intersection(&ctx_right)
        }
        ast::StmtKind::WithCwd { path, body } => {
            let lowered_path = lower_expr(path, &mut ctx, sm, file);
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(body, &mut lower_body, ctx.clone(), sm, file, opts);
            out.push(ir::Cmd::WithCwd {
                path: lowered_path,
                body: lower_body,
            });
            ctx_body
        }
        ast::StmtKind::WithLog { path, append, body } => {
            let lowered_path = lower_expr(path, &mut ctx, sm, file);
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(body, &mut lower_body, ctx.clone(), sm, file, opts);
            out.push(ir::Cmd::WithLog {
                path: lowered_path,
                append,
                body: lower_body,
            });
            ctx_body
        }
        ast::StmtKind::Cd { path } => {
            out.push(ir::Cmd::Cd(lower_expr(path, &mut ctx, sm, file)));
            ctx
        }
        ast::StmtKind::Sh(s) => {
            out.push(ir::Cmd::Raw(s));
            ctx
        }
        ast::StmtKind::ShBlock(lines) => {
            for s in lines {
                out.push(ir::Cmd::Raw(s));
            }
            ctx
        }
        ast::StmtKind::Call { name, args } => {
            if name == "save_envfile" {
                if args.len() != 2 {
                    panic!(
                        "{}",
                        sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            "save_envfile() requires exactly 2 arguments (path, env_blob)",
                            stmt.span
                        )
                    );
                }
                let mut iter = args.into_iter();
                let path = lower_expr(iter.next().unwrap(), &mut ctx, sm, file);
                let env = lower_expr(iter.next().unwrap(), &mut ctx, sm, file);
                out.push(ir::Cmd::SaveEnvfile { path, env });
            } else if name == "load_envfile" {
                panic!("{}", sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "load_envfile() returns a value; use it in an expression (e.g., let m = load_envfile(\"env.meta\"))", stmt.span));
            } else if name == "which" {
                panic!("{}", sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "which() returns a value; use it in an expression (e.g., let p = which(\"cmd\"))", stmt.span));
            } else if name == "require" {
                if args.len() != 1 {
                    panic!(
                        "{}",
                        sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            "require() requires exactly one argument (cmd_list)",
                            stmt.span
                        )
                    );
                }
                let arg = &args[0];
                if let ast::ExprKind::List(elems) = &arg.node {
                    let mut valid_cmds = Vec::new();
                    for e in elems {
                        valid_cmds.push(lower_expr(e.clone(), &mut ctx, sm, file));
                    }
                    out.push(ir::Cmd::Require(valid_cmds));
                } else {
                    panic!(
                        "{}",
                        sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "require() expects a list literal", arg.span)
                    );
                }
            } else if name == "write_file" {
                if args.len() < 2 || args.len() > 3 {
                    panic!(
                        "{}",
                        sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            "write_file() requires 2 or 3 arguments (path, content, [append])",
                            stmt.span
                        )
                    );
                }
                let mut iter = args.into_iter();
                let path = lower_expr(iter.next().unwrap(), &mut ctx, sm, file);
                let content = lower_expr(iter.next().unwrap(), &mut ctx, sm, file);
                let append = if iter.len() > 0 {
                    let arg = iter.next().unwrap();
                    if let ast::ExprKind::Bool(b) = arg.node {
                        b
                    } else {
                        panic!(
                            "{}",
                            sm.format_diagnostic(
                                file,
                                opts.diag_base_dir.as_deref(),
                                "write_file() third argument must be a boolean literal",
                                arg.span
                            )
                        );
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
                panic!("{}", sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "read_file() returns a value; use it in an expression (e.g., let s = read_file(\"foo.txt\"))", stmt.span));
            } else if matches!(name.as_str(), "log_info" | "log_warn" | "log_error") {
                let level = match name.as_str() {
                    "log_info" => ir::LogLevel::Info,
                    "log_warn" => ir::LogLevel::Warn,
                    "log_error" => ir::LogLevel::Error,
                    _ => unreachable!(),
                };
                if args.is_empty() || args.len() > 2 {
                    panic!(
                        "{}",
                        sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            format!("{}() requires 1 or 2 arguments (msg, [timestamp])", name)
                                .as_str(),
                            stmt.span
                        )
                    );
                }
                let mut iter = args.into_iter();
                let msg = lower_expr(iter.next().unwrap(), &mut ctx, sm, file);
                let timestamp = if iter.len() > 0 {
                    let arg = iter.next().unwrap();
                    if let ast::ExprKind::Bool(b) = arg.node {
                        b
                    } else {
                        panic!(
                            "{}",
                            sm.format_diagnostic(
                                file,
                                opts.diag_base_dir.as_deref(),
                                format!("{}() second argument must be a boolean literal", name)
                                    .as_str(),
                                arg.span
                            )
                        );
                    }
                } else {
                    false
                };
                out.push(ir::Cmd::Log {
                    level,
                    msg,
                    timestamp,
                });
            } else if name == "home" {
                panic!(
                    "{}",
                    sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "home() returns a value; use it in an expression (e.g., let h = home())",
                        stmt.span
                    )
                );
            } else if name == "path_join" {
                panic!("{}", sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "path_join() returns a value; use it in an expression (e.g., let p = path_join(\"a\", \"b\"))", stmt.span));
            } else if name == "try_run" {
                panic!(
                    "{}",
                    sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "try_run() must be bound via let (e.g., let r = try_run(...))",
                        stmt.span
                    )
                );
            } else {
                let args = args
                    .iter()
                    .map(|e| lower_expr(e.clone(), &mut ctx, sm, file))
                    .collect();
                out.push(ir::Cmd::Call {
                    name: name.clone(),
                    args,
                });
            }
            ctx
        }
        ast::StmtKind::Subshell { body } => {
            let mut lower_body = Vec::new();
            lower_block(body, &mut lower_body, ctx.clone(), sm, file, opts);
            out.push(ir::Cmd::Subshell { body: lower_body });
            ctx
        }
        ast::StmtKind::Group { body } => {
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(body, &mut lower_body, ctx.clone(), sm, file, opts);
            out.push(ir::Cmd::Group { body: lower_body });
            ctx_body
        }
        ast::StmtKind::WithRedirect {
            stdout,
            stderr,
            stdin,
            body,
        } => {
            let mut lowered_body = Vec::new();
            let ctx_body = lower_block(body, &mut lowered_body, ctx.clone(), sm, file, opts);

            let lower_target = |t: ast::RedirectTarget, c: &mut LoweringContext| match t {
                ast::RedirectTarget::File { path, append } => ir::RedirectTarget::File {
                    path: lower_expr(path, c, sm, file),
                    append,
                },
                ast::RedirectTarget::HereDoc { content } => ir::RedirectTarget::HereDoc { content },
                ast::RedirectTarget::Stdout => ir::RedirectTarget::Stdout,
                ast::RedirectTarget::Stderr => ir::RedirectTarget::Stderr,
            };

            out.push(ir::Cmd::WithRedirect {
                stdout: stdout.map(|t| lower_target(t, &mut ctx)),
                stderr: stderr.map(|t| lower_target(t, &mut ctx)),
                stdin: stdin.map(|t| lower_target(t, &mut ctx)),
                body: lowered_body,
            });
            ctx_body
        }
        ast::StmtKind::Spawn { stmt } => {
            let mut lower_cmds = Vec::new();
            lower_stmt(*stmt, &mut lower_cmds, ctx.clone(), sm, file, opts);

            if lower_cmds.len() == 1 {
                out.push(ir::Cmd::Spawn(Box::new(lower_cmds.remove(0))));
            } else {
                out.push(ir::Cmd::Spawn(Box::new(ir::Cmd::Group {
                    body: lower_cmds,
                })));
            }
            ctx
        }
        ast::StmtKind::Wait(expr) => {
            out.push(ir::Cmd::Wait(
                expr.map(|e| lower_expr(e, &mut ctx, sm, file)),
            ));
            ctx
        }
        ast::StmtKind::TryCatch {
            try_body,
            catch_body,
        } => {
            let mut lower_try = Vec::new();
            let ctx_try = lower_block(try_body, &mut lower_try, ctx.clone(), sm, file, opts);

            let mut lower_catch = Vec::new();
            let ctx_catch = lower_block(catch_body, &mut lower_catch, ctx.clone(), sm, file, opts);

            out.push(ir::Cmd::TryCatch {
                try_body: lower_try,
                catch_body: lower_catch,
            });
            ctx_try.intersection(&ctx_catch)
        }
        ast::StmtKind::Export { name, value } => {
            out.push(ir::Cmd::Export {
                name,
                value: value.map(|v| lower_expr(v, &mut ctx, sm, file)),
            });
            ctx
        }
        ast::StmtKind::Unset { name } => {
            out.push(ir::Cmd::Unset(name));
            ctx
        }
        ast::StmtKind::Source { path } => {
            out.push(ir::Cmd::Source(lower_expr(path, &mut ctx, sm, file)));
            ctx
        }
        ast::StmtKind::Exec(args) => {
            out.push(ir::Cmd::ExecReplace(
                args.into_iter()
                    .map(|a| lower_expr(a, &mut ctx, sm, file))
                    .collect(),
                loc,
            ));
            ctx
        }
        ast::StmtKind::Set { target, value } => {
            match target {
                ast::LValue::Var(name) => {
                    out.push(ir::Cmd::Assign(
                        name,
                        lower_expr(value, &mut ctx, sm, file),
                        loc,
                    ));
                }
                ast::LValue::Env(name) => {
                    if matches!(&value.node, ast::ExprKind::List(_) | ast::ExprKind::Args) {
                        panic!("{}", sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "set env.<NAME> requires a scalar string/number; lists/args are not supported", stmt.span));
                    }

                    let val = lower_expr(value, &mut ctx, sm, file);

                    if matches!(&val, ir::Val::List(_) | ir::Val::Args) {
                        panic!("{}", sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "set env.<NAME> requires a scalar string/number; lists/args are not supported", stmt.span));
                    }

                    out.push(ir::Cmd::Export {
                        name,
                        value: Some(val),
                    });
                }
            }
            ctx
        }
        ast::StmtKind::PipeBlocks { segments } => {
            let mut lower_segments = Vec::new();
            for seg in segments {
                let mut lowered = Vec::new();
                lower_block(seg, &mut lowered, ctx.clone(), sm, file, opts);
                lower_segments.push(lowered);
            }
            out.push(ir::Cmd::PipeBlocks(lower_segments, loc));
            ctx
        }
        ast::StmtKind::ForMap {
            key_var,
            val_var,
            map,
            body,
        } => {
            let mut lower_body = Vec::new();
            let ctx_body = lower_block(body, &mut lower_body, ctx.clone(), sm, file, opts);
            out.push(ir::Cmd::ForMap {
                key_var,
                val_var,
                map,
                body: lower_body,
            });
            ctx.intersection(&ctx_body)
        }
    }
}

fn lower_expr<'a>(e: ast::Expr, ctx: &mut LoweringContext<'a>, sm: &SourceMap, file: &str) -> ir::Val {
    let opts = ctx.opts; // Get opts from context for diagnostic formatting
    match e.node {
        ast::ExprKind::Literal(s) => ir::Val::Literal(s),
        ast::ExprKind::Var(s) => ir::Val::Var(s),
        ast::ExprKind::Concat(l, r) => ir::Val::Concat(
            Box::new(lower_expr(*l, ctx, sm, file)),
            Box::new(lower_expr(*r, ctx, sm, file)),
        ),
        ast::ExprKind::Arith { left, op, right } => {
            if matches!(op, ast::ArithOp::Add) {
                let l_is_lit = matches!(left.node, ast::ExprKind::Literal(_));
                let r_is_lit = matches!(right.node, ast::ExprKind::Literal(_));
                if l_is_lit || r_is_lit {
                    return ir::Val::Concat(
                        Box::new(lower_expr(*left, ctx, sm, file)),
                        Box::new(lower_expr(*right, ctx, sm, file)),
                    );
                }
            }

            let op = match op {
                ast::ArithOp::Add => ir::ArithOp::Add,
                ast::ArithOp::Sub => ir::ArithOp::Sub,
                ast::ArithOp::Mul => ir::ArithOp::Mul,
                ast::ArithOp::Div => ir::ArithOp::Div,
                ast::ArithOp::Mod => ir::ArithOp::Mod,
            };
            ir::Val::Arith {
                left: Box::new(lower_expr(*left, ctx, sm, file)),
                op,
                right: Box::new(lower_expr(*right, ctx, sm, file)),
            }
        }
        ast::ExprKind::Compare { left, op, right } => {
            let op = match op {
                ast::CompareOp::Eq => ir::CompareOp::Eq,
                ast::CompareOp::NotEq => ir::CompareOp::NotEq,
                ast::CompareOp::Lt => ir::CompareOp::Lt,
                ast::CompareOp::Le => ir::CompareOp::Le,
                ast::CompareOp::Gt => ir::CompareOp::Gt,
                ast::CompareOp::Ge => ir::CompareOp::Ge,
            };
            ir::Val::Compare {
                left: Box::new(lower_expr(*left, ctx, sm, file)),
                op,
                right: Box::new(lower_expr(*right, ctx, sm, file)),
            }
        }
        ast::ExprKind::And(left, right) => ir::Val::And(
            Box::new(lower_expr(*left, ctx, sm, file)),
            Box::new(lower_expr(*right, ctx, sm, file)),
        ),
        ast::ExprKind::Or(left, right) => ir::Val::Or(
            Box::new(lower_expr(*left, ctx, sm, file)),
            Box::new(lower_expr(*right, ctx, sm, file)),
        ),
        ast::ExprKind::Not(expr) => ir::Val::Not(Box::new(lower_expr(*expr, ctx, sm, file))),
        ast::ExprKind::Exists(path) => ir::Val::Exists(Box::new(lower_expr(*path, ctx, sm, file))),
        ast::ExprKind::IsDir(path) => ir::Val::IsDir(Box::new(lower_expr(*path, ctx, sm, file))),
        ast::ExprKind::IsFile(path) => ir::Val::IsFile(Box::new(lower_expr(*path, ctx, sm, file))),
        ast::ExprKind::IsSymlink(path) => {
            ir::Val::IsSymlink(Box::new(lower_expr(*path, ctx, sm, file)))
        }
        ast::ExprKind::IsExec(path) => ir::Val::IsExec(Box::new(lower_expr(*path, ctx, sm, file))),
        ast::ExprKind::IsReadable(path) => {
            ir::Val::IsReadable(Box::new(lower_expr(*path, ctx, sm, file)))
        }
        ast::ExprKind::IsWritable(path) => {
            ir::Val::IsWritable(Box::new(lower_expr(*path, ctx, sm, file)))
        }
        ast::ExprKind::IsNonEmpty(path) => {
            ir::Val::IsNonEmpty(Box::new(lower_expr(*path, ctx, sm, file)))
        }
        ast::ExprKind::BoolStr(inner) => {
            ir::Val::BoolStr(Box::new(lower_expr(*inner, ctx, sm, file)))
        }
        ast::ExprKind::Len(expr) => ir::Val::Len(Box::new(lower_expr(*expr, ctx, sm, file))),
        ast::ExprKind::Arg(n) => ir::Val::Arg(n),
        ast::ExprKind::Index { list, index } => ir::Val::Index {
            list: Box::new(lower_expr(*list, ctx, sm, file)),
            index: Box::new(lower_expr(*index, ctx, sm, file)),
        },
        ast::ExprKind::Field { base, name } => {
            let b = lower_expr(*base, ctx, sm, file);

            match name.as_str() {
                "flags" => ir::Val::ArgsFlags(Box::new(b)),
                "positionals" => ir::Val::ArgsPositionals(Box::new(b)),
                "status" | "stdout" | "stderr" => {
                    if let ir::Val::Var(vname) = &b {
                        if ctx.run_results.contains(vname) {
                            ir::Val::Var(format!("{}__{}", vname, name))
                        } else {
                            panic!("{}", sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), format!(".{} is only valid on try_run() results (bind via let)", name).as_str(), e.span));
                        }
                    } else {
                        panic!("{}", sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), format!("Field access '{}' only supported on variables (e.g. r.status)", name).as_str(), e.span));
                    }
                }
                _ => panic!("{}", sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), format!("Unknown field '{}'. Supported: status, stdout, stderr, flags, positionals.", name).as_str(), e.span)),
            }
        }
        ast::ExprKind::Join { list, sep } => ir::Val::Join {
            list: Box::new(lower_expr(*list, ctx, sm, file)),
            sep: Box::new(lower_expr(*sep, ctx, sm, file)),
        },
        ast::ExprKind::Count(inner) => ir::Val::Count(Box::new(lower_expr(*inner, ctx, sm, file))),
        ast::ExprKind::Bool(b) => ir::Val::Bool(b),
        ast::ExprKind::Number(n) => ir::Val::Number(n),
        ast::ExprKind::Command(args) => {
            let lowered_args = args
                .into_iter()
                .map(|a| lower_expr(a, ctx, sm, file))
                .collect();
            ir::Val::Command(lowered_args)
        }
        ast::ExprKind::CommandPipe(segments) => {
            let lowered_segments = segments
                .into_iter()
                .map(|seg| {
                    seg.into_iter()
                        .map(|a| lower_expr(a, ctx, sm, file))
                        .collect()
                })
                .collect();
            ir::Val::CommandPipe(lowered_segments)
        }
        ast::ExprKind::List(exprs) => {
            let lowered_exprs = exprs
                .into_iter()
                .map(|e| lower_expr(e, ctx, sm, file))
                .collect();
            ir::Val::List(lowered_exprs)
        }
        ast::ExprKind::Args => ir::Val::Args,
        ast::ExprKind::Status => ir::Val::Status,
        ast::ExprKind::Pid => ir::Val::Pid,
        ast::ExprKind::Env(inner) => ir::Val::Env(Box::new(lower_expr(*inner, ctx, sm, file))),
        ast::ExprKind::Uid => ir::Val::Uid,
        ast::ExprKind::Ppid => ir::Val::Ppid,
        ast::ExprKind::Pwd => ir::Val::Pwd,
        ast::ExprKind::SelfPid => ir::Val::SelfPid,
        ast::ExprKind::Argv0 => ir::Val::Argv0,
        ast::ExprKind::Argc => ir::Val::Argc,
        ast::ExprKind::EnvDot(name) => ir::Val::EnvDot(name),
        ast::ExprKind::Input(e) => ir::Val::Input(Box::new(lower_expr(*e, ctx, sm, file))),
        ast::ExprKind::Confirm(e) => ir::Val::Confirm(Box::new(lower_expr(*e, ctx, sm, file))),
        ast::ExprKind::Call { name, args } => {
            if name == "matches" {
                if args.len() != 2 {
                    panic!(
                        "{}",
                        sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            "matches() requires exactly 2 arguments (text, regex)",
                            e.span
                        )
                    );
                }
                let mut iter = args.into_iter();
                let text = Box::new(lower_expr(iter.next().unwrap(), ctx, sm, file));
                let regex = Box::new(lower_expr(iter.next().unwrap(), ctx, sm, file));
                ir::Val::Matches(text, regex)
            } else if name == "parse_args" {
                if !args.is_empty() {
                    panic!(
                        "{}",
                        sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "parse_args() takes no arguments", e.span)
                    );
                }
                ir::Val::ParseArgs
            } else if name == "load_envfile" {
                if args.len() != 1 {
                    panic!(
                        "{}",
                        sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            "load_envfile() requires exactly 1 argument (path)",
                            e.span
                        )
                    );
                }
                let path = lower_expr(args.into_iter().next().unwrap(), ctx, sm, file);
                ir::Val::LoadEnvfile(Box::new(path))
            } else if name == "json_kv" {
                if args.len() != 1 {
                    panic!(
                        "{}",
                        sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            "json_kv() requires exactly 1 argument (pairs_blob)",
                            e.span
                        )
                    );
                }
                let blob = lower_expr(args.into_iter().next().unwrap(), ctx, sm, file);
                ir::Val::JsonKv(Box::new(blob))
            } else if name == "which" {
                if args.len() != 1 {
                    panic!(
                        "{}",
                        sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            "which() requires exactly 1 argument (cmd)",
                            e.span
                        )
                    );
                }
                let arg = lower_expr(args.into_iter().next().unwrap(), ctx, sm, file);
                ir::Val::Which(Box::new(arg))
            } else if name == "try_run" {
                panic!(
                    "{}",
                    sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "try_run() must be bound via let (e.g., let r = try_run(...))",
                        e.span
                    )
                );
            } else if name == "require" {
                panic!(
                    "{}",
                    sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "require() is a statement, not an expression",
                        e.span
                    )
                );
            } else if name == "read_file" {
                if args.len() != 1 {
                    panic!(
                        "{}",
                        sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            "read_file() requires exactly 1 argument (path)",
                            e.span
                        )
                    );
                }
                let arg = lower_expr(args.into_iter().next().unwrap(), ctx, sm, file);
                ir::Val::ReadFile(Box::new(arg))
            } else if name == "write_file" {
                panic!(
                    "{}",
                    sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "write_file() is a statement, not an expression",
                        e.span
                    )
                );
            } else if matches!(name.as_str(), "log_info" | "log_warn" | "log_error") {
                panic!(
                    "{}",
                    sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        format!("{}() is a statement, not an expression", name).as_str(),
                        e.span
                    )
                );
            } else if name == "home" {
                if !args.is_empty() {
                    panic!(
                        "{}",
                        sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "home() takes no arguments", e.span)
                    );
                }
                ir::Val::Home
            } else if name == "path_join" {
                if args.is_empty() {
                    panic!(
                        "{}",
                            sm.format_diagnostic(
                                file,
                                opts.diag_base_dir.as_deref(),
                                "path_join() requires at least 1 argument",
                                e.span
                            )                  );
                }
                let lowered_args = args
                    .into_iter()
                    .map(|a| lower_expr(a, ctx, sm, file))
                    .collect();
                ir::Val::PathJoin(lowered_args)
            } else if name == "save_envfile" {
                panic!(
                    "{}",
                    sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "save_envfile() is a statement; use it as a standalone call",
                        e.span
                    )
                );
            } else {
                let lowered_args = args
                    .into_iter()
                    .map(|a| lower_expr(a, ctx, sm, file))
                    .collect();
                ir::Val::Call {
                    name,
                    args: lowered_args,
                }
            }
        }
        ast::ExprKind::MapLiteral(entries) => {
            let lowered_entries = entries
                .into_iter()
                .map(|(k, v)| (k, lower_expr(v, ctx, sm, file)))
                .collect();
            ir::Val::MapLiteral(lowered_entries)
        }
        ast::ExprKind::MapIndex { map, key } => ir::Val::MapIndex { map, key },
    }
}
