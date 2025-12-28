use crate::ast;
use crate::ir;
use std::collections::HashSet;

struct LoweringContext {
    run_results: HashSet<String>,
}

/// Lower a whole program (AST) into IR
pub fn lower(mut p: ast::Program) -> Vec<ir::Function> {
    let has_main = p.functions.iter().any(|f| f.name == "main");
    let has_top_level = !p.top_level.is_empty();

    if has_top_level {
        if has_main {
            panic!("Top-level statements are not allowed when `func main` is defined; move statements into main or remove main to use implicit main.");
        }
        // Synthesize main
        let main_func = ast::Function {
             name: "main".to_string(),
             params: vec![],
             body: p.top_level, // Move checks out
        };
        p.functions.push(main_func);
    } else if !has_main {
         panic!("No entrypoint: define `func main()` or add top-level statements.");
    }

    p.functions
        .into_iter()
        .map(lower_function)
        .collect()
}

/// Lower a single function
fn lower_function(f: ast::Function) -> ir::Function {
    let mut commands = Vec::new();
    let mut ctx = LoweringContext {
        run_results: HashSet::new(),
    };

    for stmt in f.body {
        lower_stmt(stmt, &mut commands, &mut ctx);
    }

    ir::Function {
        name: f.name,
        params: f.params,
        commands,
    }
}

/// Lower one AST statement into IR commands
/// Lower one AST statement into IR commands
fn lower_stmt(stmt: ast::Stmt, out: &mut Vec<ir::Cmd>, ctx: &mut LoweringContext) {
    match stmt {
        ast::Stmt::Let { name, value } => {
            // Special handling for try_run to allow it ONLY during strict let-binding lowering.
            // This prevents "let x = 1 + try_run(...)" but allows "let x = try_run(...)".
            if let ast::Expr::Call { name: func_name, args } = &value {
                if func_name == "try_run" {
                    if args.is_empty() {
                        panic!("try_run() requires at least 1 argument (cmd)");
                    }
                    // lower args using ctx
                    let lowered_args = args.clone().into_iter().map(|a| lower_expr(a, ctx)).collect();
                    out.push(ir::Cmd::Assign(name.clone(), ir::Val::TryRun(lowered_args)));
                    ctx.run_results.insert(name);
                    return;
                }
            }
            out.push(ir::Cmd::Assign(name.clone(), lower_expr(value, ctx)));
            ctx.run_results.remove(&name);
        }

        ast::Stmt::Run(run_call) => {
            let ir_args = run_call.args.into_iter().map(|a| lower_expr(a, ctx)).collect();
            out.push(ir::Cmd::Exec { args: ir_args, allow_fail: run_call.allow_fail });
        }

        ast::Stmt::Print(e) => {
            out.push(ir::Cmd::Print(lower_expr(e, ctx)));
        }

        ast::Stmt::PrintErr(e) => {
            out.push(ir::Cmd::PrintErr(lower_expr(e, ctx)));
        }
        ast::Stmt::If { cond, then_body, elifs, else_body } => {
            let mut t_cmds = Vec::new();
            for s in then_body {
                lower_stmt(s, &mut t_cmds, ctx);
            }
            
            let mut lowered_elifs = Vec::new();
            for elif in elifs {
                let mut body_cmds = Vec::new();
                for s in elif.body {
                    lower_stmt(s, &mut body_cmds, ctx);
                }
                lowered_elifs.push((lower_expr(elif.cond, ctx), body_cmds));
            }

            let mut e_cmds = Vec::new();
            if let Some(body) = else_body {
                for s in body {
                    lower_stmt(s, &mut e_cmds, ctx);
                }
            }

            out.push(ir::Cmd::If {
                cond: lower_expr(cond, ctx),
                then_body: t_cmds,
                elifs: lowered_elifs,
                else_body: e_cmds,
            });
        }
        ast::Stmt::Case { expr, arms } => {
            let mut lower_arms = Vec::new();
            for arm in arms {
                let mut body_cmds = Vec::new();
                for s in arm.body {
                    lower_stmt(s, &mut body_cmds, ctx);
                }
                
                let patterns = arm.patterns.into_iter().map(|p| match p {
                    ast::Pattern::Literal(s) => ir::Pattern::Literal(s),
                    ast::Pattern::Glob(s) => ir::Pattern::Glob(s),
                    ast::Pattern::Wildcard => ir::Pattern::Wildcard,
                }).collect();
                
                lower_arms.push((patterns, body_cmds));
            }
            out.push(ir::Cmd::Case {
                expr: lower_expr(expr, ctx),
                arms: lower_arms,
            });
        }
        ast::Stmt::While { cond, body } => {
            let mut lower_body = Vec::new();
            for s in body {
                lower_stmt(s, &mut lower_body, ctx);
            }
            out.push(ir::Cmd::While {
                cond: lower_expr(cond, ctx),
                body: lower_body,
            });
        }
        ast::Stmt::For { var, items, body } => {
            let mut lower_body = Vec::new();
            for s in body {
                lower_stmt(s, &mut lower_body, ctx);
            }
            let lowered_items = items.into_iter().map(|i| lower_expr(i, ctx)).collect();
            out.push(ir::Cmd::For {
                var,
                items: lowered_items,
                body: lower_body,
            });
        }
        ast::Stmt::Pipe(segments) => {
            let mut lowered_segments = Vec::new();
            for run_call in segments {
                let lowered_args = run_call.args.into_iter().map(|a| lower_expr(a, ctx)).collect();
                lowered_segments.push((lowered_args, run_call.allow_fail));
            }
            out.push(ir::Cmd::Pipe(lowered_segments));
        }
        ast::Stmt::Break => {
            out.push(ir::Cmd::Break);
        }
        ast::Stmt::Continue => {
            out.push(ir::Cmd::Continue);
        }
        ast::Stmt::Return(e) => {
             out.push(ir::Cmd::Return(e.map(|x| lower_expr(x, ctx))));
        }
        ast::Stmt::Exit(e) => {
             out.push(ir::Cmd::Exit(e.map(|x| lower_expr(x, ctx))));
        }
        ast::Stmt::WithEnv { bindings, body } => {
            let lowered_bindings = bindings.into_iter().map(|(k, v)| (k, lower_expr(v, ctx))).collect();
            let mut lower_body = Vec::new();
            for s in body {
                lower_stmt(s, &mut lower_body, ctx);
            }
            out.push(ir::Cmd::WithEnv {
                bindings: lowered_bindings,
                body: lower_body,
            });
        }
        ast::Stmt::AndThen { left, right } => {
            let mut lower_left = Vec::new();
            for s in left {
                lower_stmt(s, &mut lower_left, ctx);
            }
            let mut lower_right = Vec::new();
            for s in right {
                lower_stmt(s, &mut lower_right, ctx);
            }
            out.push(ir::Cmd::AndThen { left: lower_left, right: lower_right });
        }
        ast::Stmt::OrElse { left, right } => {
            let mut lower_left = Vec::new();
            for s in left {
                lower_stmt(s, &mut lower_left, ctx);
            }
            let mut lower_right = Vec::new();
            for s in right {
                lower_stmt(s, &mut lower_right, ctx);
            }
            out.push(ir::Cmd::OrElse { left: lower_left, right: lower_right });
        }
        ast::Stmt::WithCwd { path, body } => {
            let lowered_path = lower_expr(path, ctx);
            let mut lower_body = Vec::new();
            for s in body {
                lower_stmt(s, &mut lower_body, ctx);
            }
            out.push(ir::Cmd::WithCwd {
                path: lowered_path,
                body: lower_body,
            });
        }
        ast::Stmt::WithLog { path, append, body } => {
            let lowered_path = lower_expr(path, ctx);
            let mut lower_body = Vec::new();
            for s in body {
                lower_stmt(s, &mut lower_body, ctx);
            }
            out.push(ir::Cmd::WithLog {
                path: lowered_path,
                append,
                body: lower_body,
            });
        }
        ast::Stmt::Cd { path } => {
            out.push(ir::Cmd::Cd(lower_expr(path, ctx)));
        }
        ast::Stmt::Sh(s) => {
            out.push(ir::Cmd::Raw(s));
        }
        ast::Stmt::ShBlock(lines) => {
            for s in lines {
                out.push(ir::Cmd::Raw(s));
            }
        }
        ast::Stmt::Call { name, args } => {
            if name == "save_envfile" {
                 if args.len() != 2 {
                     panic!("save_envfile() requires exactly 2 arguments (path, env_blob)");
                 }
                 let mut iter = args.into_iter();
                 let path = lower_expr(iter.next().unwrap(), ctx);
                 let env = lower_expr(iter.next().unwrap(), ctx);
                 out.push(ir::Cmd::SaveEnvfile { path, env });
            } else if name == "load_envfile" {
                 panic!("load_envfile() returns a value; use it in an expression (e.g., let m = load_envfile(\"env.meta\"))");
            } else if name == "which" {
                 panic!("which() returns a value; use it in an expression (e.g., let p = which(\"cmd\"))");
            } else if name == "require" {
                 if args.len() != 1 {
                     panic!("require() requires exactly one argument (cmd_list)");
                 }
                 let arg = &args[0];
                 if let ast::Expr::List(elems) = arg {
                     let mut valid_cmds = Vec::new();
                     for e in elems {
                         valid_cmds.push(lower_expr(e.clone(), ctx));
                     }
                     out.push(ir::Cmd::Require(valid_cmds));
                 } else {
                     panic!("require() expects a list literal");
                 }
            } else if name == "write_file" {
                 if args.len() < 2 || args.len() > 3 {
                     panic!("write_file() requires 2 or 3 arguments (path, content, [append])");
                 }
                 let mut iter = args.into_iter();
                 let path = lower_expr(iter.next().unwrap(), ctx);
                 let content = lower_expr(iter.next().unwrap(), ctx);
                 let append = if iter.len() > 0 {
                     if let ast::Expr::Bool(b) = iter.next().unwrap() {
                         b
                     } else {
                         panic!("write_file() third argument must be a boolean literal");
                     }
                 } else {
                     false
                 };
                 out.push(ir::Cmd::WriteFile { path, content, append });
            } else if name == "read_file" {
                 panic!("read_file() returns a value; use it in an expression (e.g., let s = read_file(\"foo.txt\"))");
            } else if matches!(name.as_str(), "log_info" | "log_warn" | "log_error") {
                 let level = match name.as_str() {
                     "log_info" => ir::LogLevel::Info,
                     "log_warn" => ir::LogLevel::Warn,
                     "log_error" => ir::LogLevel::Error,
                     _ => unreachable!(),
                 };
                 if args.is_empty() || args.len() > 2 {
                     panic!("{}() requires 1 or 2 arguments (msg, [timestamp])", name);
                 }
                 let mut iter = args.into_iter();
                 let msg = lower_expr(iter.next().unwrap(), ctx);
                 let timestamp = if iter.len() > 0 {
                     if let ast::Expr::Bool(b) = iter.next().unwrap() {
                         b
                     } else {
                         panic!("{}() second argument must be a boolean literal", name);
                     }
                 } else {
                     false
                 };
                 out.push(ir::Cmd::Log { level, msg, timestamp });
            } else if name == "home" {
                 panic!("home() returns a value; use it in an expression (e.g., let h = home())");
            } else if name == "path_join" {
                 panic!("path_join() returns a value; use it in an expression (e.g., let p = path_join(\"a\", \"b\"))");
            } else if name == "try_run" {
                 panic!("try_run() must be bound via let (e.g., let r = try_run(...))");
            } else {
                let args = args.iter().map(|e| lower_expr(e.clone(), ctx)).collect();
                out.push(ir::Cmd::Call { name: name.clone(), args });
            }
        }
        ast::Stmt::Subshell { body } => {
            let mut lowered = Vec::new();
            for s in body {
                lower_stmt(s, &mut lowered, ctx);
            }
            out.push(ir::Cmd::Subshell { body: lowered });
        }
        ast::Stmt::Group { body } => {
            let mut lowered = Vec::new();
            for s in body {
                lower_stmt(s, &mut lowered, ctx);
            }
            out.push(ir::Cmd::Group { body: lowered });
        }
        ast::Stmt::WithRedirect { stdout, stderr, stdin, body } => {
             let mut lowered_body = Vec::new();
             for s in body {
                 lower_stmt(s, &mut lowered_body, ctx);
             }
             
             let lower_target = |t: ast::RedirectTarget, c: &mut LoweringContext| match t {
                 ast::RedirectTarget::File { path, append } => ir::RedirectTarget::File { path: lower_expr(path, c), append },
                 ast::RedirectTarget::HereDoc { content } => ir::RedirectTarget::HereDoc { content },
                 ast::RedirectTarget::Stdout => ir::RedirectTarget::Stdout,
                 ast::RedirectTarget::Stderr => ir::RedirectTarget::Stderr,
             };
             
             out.push(ir::Cmd::WithRedirect {
                  stdout: stdout.map(|t| lower_target(t, ctx)),
                 stderr: stderr.map(|t| lower_target(t, ctx)),
                 stdin: stdin.map(|t| lower_target(t, ctx)),
                 body: lowered_body,
             });
        }
        ast::Stmt::Spawn { stmt } => {
            let mut lower_cmds = Vec::new();
            lower_stmt(*stmt, &mut lower_cmds, ctx);
            
            if lower_cmds.len() == 1 {
                 out.push(ir::Cmd::Spawn(Box::new(lower_cmds.remove(0))));
            } else {
                 out.push(ir::Cmd::Spawn(Box::new(ir::Cmd::Group { body: lower_cmds })));
            }
        }
        ast::Stmt::Wait(expr) => {
            out.push(ir::Cmd::Wait(expr.map(|e| lower_expr(e, ctx))));
        }
        ast::Stmt::TryCatch { try_body, catch_body } => {
            let mut lower_try = Vec::new();
            for s in try_body {
                lower_stmt(s, &mut lower_try, ctx);
            }
            let mut lower_catch = Vec::new();
            for s in catch_body {
                lower_stmt(s, &mut lower_catch, ctx);
            }
            out.push(ir::Cmd::TryCatch { try_body: lower_try, catch_body: lower_catch });
        }
        ast::Stmt::Export { name, value } => {
            out.push(ir::Cmd::Export {
                name,
                value: value.map(|v| lower_expr(v, ctx)),
            });
        }
        ast::Stmt::Unset { name } => {
            out.push(ir::Cmd::Unset(name));
        }
        ast::Stmt::Source { path } => {
            out.push(ir::Cmd::Source(lower_expr(path, ctx)));
        }
        ast::Stmt::Exec(args) => {
            out.push(ir::Cmd::ExecReplace(args.into_iter().map(|a| lower_expr(a, ctx)).collect()));
        }
        ast::Stmt::Set { target, value } => {
             match target {
                 ast::LValue::Var(name) => {
                     out.push(ir::Cmd::Assign(name, lower_expr(value, ctx)));
                 }
                  ast::LValue::Env(name) => {
                      if matches!(&value, ast::Expr::List(_) | ast::Expr::Args) {
                          panic!("set env.<NAME> requires a scalar string/number; lists/args are not supported");
                      }

                      let val = lower_expr(value, ctx);

                      if matches!(&val, ir::Val::List(_) | ir::Val::Args) {
                           panic!("set env.<NAME> requires a scalar string/number; lists/args are not supported");
                      }
                      
                      out.push(ir::Cmd::Export {
                          name,
                          value: Some(val),
                      });
                  }
             }
        }
        ast::Stmt::PipeBlocks { segments } => {
            let mut lower_segments = Vec::new();
            for seg in segments {
                let mut lowered = Vec::new();
                for s in seg {
                    lower_stmt(s, &mut lowered, ctx);
                }
                lower_segments.push(lowered);
            }
            out.push(ir::Cmd::PipeBlocks(lower_segments));
        }
        ast::Stmt::ForMap { key_var, val_var, map, body } => {
            let mut lower_body = Vec::new();
            for s in body {
                lower_stmt(s, &mut lower_body, ctx);
            }
            out.push(ir::Cmd::ForMap {
                key_var,
                val_var,
                map,
                body: lower_body,
            });
        }
    }
}

fn lower_expr(e: ast::Expr, ctx: &mut LoweringContext) -> ir::Val {
    match e {
        ast::Expr::Literal(s) => ir::Val::Literal(s),
        ast::Expr::Var(s) => ir::Val::Var(s),
        ast::Expr::Concat(l, r) => ir::Val::Concat(Box::new(lower_expr(*l, ctx)), Box::new(lower_expr(*r, ctx))),
        ast::Expr::Arith { left, op, right } => {
            // HACK: If op is Add and either side is a string literal, lower to Concat to preserve legacy string behavior.
            // This is a static heuristic because we don't have types.
            if matches!(op, ast::ArithOp::Add) {
                let l_is_lit = matches!(*left, ast::Expr::Literal(_));
                let r_is_lit = matches!(*right, ast::Expr::Literal(_));
                if l_is_lit || r_is_lit {
                    return ir::Val::Concat(Box::new(lower_expr(*left, ctx)), Box::new(lower_expr(*right, ctx)));
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
                left: Box::new(lower_expr(*left, ctx)),
                op,
                right: Box::new(lower_expr(*right, ctx)),
            }
        }
        ast::Expr::Compare { left, op, right } => {
            let op = match op {
                ast::CompareOp::Eq => ir::CompareOp::Eq,
                ast::CompareOp::NotEq => ir::CompareOp::NotEq,
                ast::CompareOp::Lt => ir::CompareOp::Lt,
                ast::CompareOp::Le => ir::CompareOp::Le,
                ast::CompareOp::Gt => ir::CompareOp::Gt,
                ast::CompareOp::Ge => ir::CompareOp::Ge,
            };
            ir::Val::Compare {
                left: Box::new(lower_expr(*left, ctx)),
                op,
                right: Box::new(lower_expr(*right, ctx)),
            }
        }
        ast::Expr::And(left, right) => {
            ir::Val::And(Box::new(lower_expr(*left, ctx)), Box::new(lower_expr(*right, ctx)))
        }
        ast::Expr::Or(left, right) => {
            ir::Val::Or(Box::new(lower_expr(*left, ctx)), Box::new(lower_expr(*right, ctx)))
        }
        ast::Expr::Not(expr) => {
            ir::Val::Not(Box::new(lower_expr(*expr, ctx)))
        }
        ast::Expr::Exists(path) => {
            ir::Val::Exists(Box::new(lower_expr(*path, ctx)))
        }
        ast::Expr::IsDir(path) => {
            ir::Val::IsDir(Box::new(lower_expr(*path, ctx)))
        }
        ast::Expr::IsFile(path) => {
            ir::Val::IsFile(Box::new(lower_expr(*path, ctx)))
        }
        ast::Expr::IsSymlink(path) => {
            ir::Val::IsSymlink(Box::new(lower_expr(*path, ctx)))
        }
        ast::Expr::IsExec(path) => {
            ir::Val::IsExec(Box::new(lower_expr(*path, ctx)))
        }
        ast::Expr::IsReadable(path) => {
            ir::Val::IsReadable(Box::new(lower_expr(*path, ctx)))
        }
        ast::Expr::IsWritable(path) => {
            ir::Val::IsWritable(Box::new(lower_expr(*path, ctx)))
        }
        ast::Expr::IsNonEmpty(path) => {
            ir::Val::IsNonEmpty(Box::new(lower_expr(*path, ctx)))
        }
        ast::Expr::BoolStr(inner) => {
            ir::Val::BoolStr(Box::new(lower_expr(*inner, ctx)))
        }
        ast::Expr::Len(expr) => {
            ir::Val::Len(Box::new(lower_expr(*expr, ctx)))
        }
        ast::Expr::Arg(n) => ir::Val::Arg(n),
        ast::Expr::Index { list, index } => ir::Val::Index {
            list: Box::new(lower_expr(*list, ctx)),
            index: Box::new(lower_expr(*index, ctx)),
        },
        ast::Expr::Field { base, name } => {
            let b = lower_expr(*base, ctx);

            match name.as_str() {
                "flags" => ir::Val::ArgsFlags(Box::new(b)),
                "positionals" => ir::Val::ArgsPositionals(Box::new(b)),
                "status" | "stdout" | "stderr" => {
                    if let ir::Val::Var(vname) = &b {
                        if ctx.run_results.contains(vname) {
                            ir::Val::Var(format!("{}__{}", vname, name))
                        } else {
                            panic!(".{} is only valid on try_run() results (bind via let)", name);
                        }
                    } else {
                        panic!("Field access '{}' only supported on variables (e.g. r.status)", name);
                    }
                }
                _ => panic!("Unknown field '{}'. Supported: status, stdout, stderr, flags, positionals.", name),
            }
        },
        ast::Expr::Join { list, sep } => ir::Val::Join {
            list: Box::new(lower_expr(*list, ctx)),
            sep: Box::new(lower_expr(*sep, ctx)),
        },
        ast::Expr::Count(inner) => ir::Val::Count(Box::new(lower_expr(*inner, ctx))),
        ast::Expr::Bool(b) => ir::Val::Bool(b),
        ast::Expr::Number(n) => ir::Val::Number(n),
        ast::Expr::Command(args) => {
            let lowered_args = args.into_iter().map(|a| lower_expr(a, ctx)).collect();
            ir::Val::Command(lowered_args)
        }
        ast::Expr::CommandPipe(segments) => {
            let lowered_segments = segments.into_iter()
                .map(|seg| seg.into_iter().map(|a| lower_expr(a, ctx)).collect())
                .collect();
            ir::Val::CommandPipe(lowered_segments)
        }
        ast::Expr::List(exprs) => {
            let lowered_exprs = exprs.into_iter().map(|e| lower_expr(e, ctx)).collect();
            ir::Val::List(lowered_exprs)
        }
        ast::Expr::Args => ir::Val::Args,
        ast::Expr::Status => ir::Val::Status,
        ast::Expr::Pid => ir::Val::Pid,
        ast::Expr::Env(inner) => ir::Val::Env(Box::new(lower_expr(*inner, ctx))),
        ast::Expr::Uid => ir::Val::Uid,
        ast::Expr::Ppid => ir::Val::Ppid,
        ast::Expr::Pwd => ir::Val::Pwd,
        ast::Expr::SelfPid => ir::Val::SelfPid,
        ast::Expr::Argv0 => ir::Val::Argv0,
        ast::Expr::Argc => ir::Val::Argc,
        ast::Expr::EnvDot(name) => ir::Val::EnvDot(name),
        ast::Expr::Input(e) => ir::Val::Input(Box::new(lower_expr(*e, ctx))),
        ast::Expr::Confirm(e) => ir::Val::Confirm(Box::new(lower_expr(*e, ctx))),
        ast::Expr::Call { name, args } => {
            if name == "matches" {
                if args.len() != 2 {
                    panic!("matches() requires exactly 2 arguments (text, regex)");
                }
                let mut iter = args.into_iter();
                let text = Box::new(lower_expr(iter.next().unwrap(), ctx));
                let regex = Box::new(lower_expr(iter.next().unwrap(), ctx));
                ir::Val::Matches(text, regex)
            } else if name == "parse_args" {
                if !args.is_empty() {
                    panic!("parse_args() takes no arguments");
                }
                ir::Val::ParseArgs
            } else if name == "load_envfile" {
                if args.len() != 1 {
                    panic!("load_envfile() requires exactly 1 argument (path)");
                }
                let path = lower_expr(args.into_iter().next().unwrap(), ctx);
                ir::Val::LoadEnvfile(Box::new(path))
            } else if name == "json_kv" {
                if args.len() != 1 {
                    panic!("json_kv() requires exactly 1 argument (pairs_blob)");
                }
                let blob = lower_expr(args.into_iter().next().unwrap(), ctx);
                ir::Val::JsonKv(Box::new(blob))
            } else if name == "which" {
                if args.len() != 1 {
                    panic!("which() requires exactly 1 argument (cmd)");
                }
                let arg = lower_expr(args.into_iter().next().unwrap(), ctx);
                ir::Val::Which(Box::new(arg))
            } else if name == "try_run" {
                 panic!("try_run() must be bound via let (e.g., let r = try_run(...))");
            } else if name == "require" {
                panic!("require() is a statement, not an expression");
            } else if name == "read_file" {
                 if args.len() != 1 {
                     panic!("read_file() requires exactly 1 argument (path)");
                 }
                 let arg = lower_expr(args.into_iter().next().unwrap(), ctx);
                 ir::Val::ReadFile(Box::new(arg))
            } else if name == "write_file" {
                 panic!("write_file() is a statement, not an expression");
            } else if matches!(name.as_str(), "log_info" | "log_warn" | "log_error") {
                 panic!("{}() is a statement, not an expression", name);
            } else if name == "home" {
                if !args.is_empty() {
                    panic!("home() takes no arguments");
                }
                ir::Val::Home
            } else if name == "path_join" {
                if args.is_empty() {
                    panic!("path_join() requires at least 1 argument");
                }
                let lowered_args = args.into_iter().map(|a| lower_expr(a, ctx)).collect();
                ir::Val::PathJoin(lowered_args)
            } else if name == "save_envfile" {
                 panic!("save_envfile() is a statement; use it as a standalone call");
            } else {
                let lowered_args = args.into_iter().map(|a| lower_expr(a, ctx)).collect();
                ir::Val::Call { name, args: lowered_args }
            }
        }
        ast::Expr::MapLiteral(entries) => {
            let lowered_entries = entries.into_iter().map(|(k, v)| (k, lower_expr(v, ctx))).collect();
            ir::Val::MapLiteral(lowered_entries)
        }
        ast::Expr::MapIndex { map, key } => {
            ir::Val::MapIndex { map, key }
        }
    }
}
