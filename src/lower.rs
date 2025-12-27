use crate::ast;
use crate::ir;

/// Lower a whole program (AST) into IR
pub fn lower(p: ast::Program) -> Vec<ir::Function> {
    p.functions
        .into_iter()
        .map(lower_function)
        .collect()
}

/// Lower a single function
fn lower_function(f: ast::Function) -> ir::Function {
    let mut commands = Vec::new();

    for stmt in f.body {
        lower_stmt(stmt, &mut commands);
    }

    ir::Function {
        name: f.name,
        params: f.params,
        commands,
    }
}

/// Lower one AST statement into IR commands
fn lower_stmt(stmt: ast::Stmt, out: &mut Vec<ir::Cmd>) {
    match stmt {
        ast::Stmt::Let { name, value } => {
            out.push(ir::Cmd::Assign(name, lower_expr(value)));
        }

        ast::Stmt::Run(run_call) => {
            let ir_args = run_call.args.into_iter().map(lower_expr).collect();
            out.push(ir::Cmd::Exec { args: ir_args, allow_fail: run_call.allow_fail });
        }

        ast::Stmt::Print(e) => {
            out.push(ir::Cmd::Print(lower_expr(e)));
        }

        ast::Stmt::PrintErr(e) => {
            out.push(ir::Cmd::PrintErr(lower_expr(e)));
        }
        ast::Stmt::If { cond, then_body, elifs, else_body } => {
            let mut t_cmds = Vec::new();
            for s in then_body {
                lower_stmt(s, &mut t_cmds);
            }
            
            let mut lowered_elifs = Vec::new();
            for elif in elifs {
                let mut body_cmds = Vec::new();
                for s in elif.body {
                    lower_stmt(s, &mut body_cmds);
                }
                lowered_elifs.push((lower_expr(elif.cond), body_cmds));
            }

            let mut e_cmds = Vec::new();
            if let Some(body) = else_body {
                for s in body {
                    lower_stmt(s, &mut e_cmds);
                }
            }

            out.push(ir::Cmd::If {
                cond: lower_expr(cond),
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
                    lower_stmt(s, &mut body_cmds);
                }
                
                let patterns = arm.patterns.into_iter().map(|p| match p {
                    ast::Pattern::Literal(s) => ir::Pattern::Literal(s),
                    ast::Pattern::Glob(s) => ir::Pattern::Glob(s),
                    ast::Pattern::Wildcard => ir::Pattern::Wildcard,
                }).collect();
                
                lower_arms.push((patterns, body_cmds));
            }
            out.push(ir::Cmd::Case {
                expr: lower_expr(expr),
                arms: lower_arms,
            });
        }
        ast::Stmt::While { cond, body } => {
            let mut lower_body = Vec::new();
            for s in body {
                lower_stmt(s, &mut lower_body);
            }
            out.push(ir::Cmd::While {
                cond: lower_expr(cond),
                body: lower_body,
            });
        }
        ast::Stmt::For { var, items, body } => {
            let mut lower_body = Vec::new();
            for s in body {
                lower_stmt(s, &mut lower_body);
            }
            let lowered_items = items.into_iter().map(lower_expr).collect();
            out.push(ir::Cmd::For {
                var,
                items: lowered_items,
                body: lower_body,
            });
        }
        ast::Stmt::Pipe(segments) => {
            let mut lowered_segments = Vec::new();
            for run_call in segments {
                let lowered_args = run_call.args.into_iter().map(lower_expr).collect();
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
             out.push(ir::Cmd::Return(e.map(lower_expr)));
        }
        ast::Stmt::Exit(e) => {
             out.push(ir::Cmd::Exit(e.map(lower_expr)));
        }
        ast::Stmt::WithEnv { bindings, body } => {
            let lowered_bindings = bindings.into_iter().map(|(k, v)| (k, lower_expr(v))).collect();
            let mut lower_body = Vec::new();
            for s in body {
                lower_stmt(s, &mut lower_body);
            }
            out.push(ir::Cmd::WithEnv {
                bindings: lowered_bindings,
                body: lower_body,
            });
        }
        ast::Stmt::AndThen { left, right } => {
            let mut lower_left = Vec::new();
            for s in left {
                lower_stmt(s, &mut lower_left);
            }
            let mut lower_right = Vec::new();
            for s in right {
                lower_stmt(s, &mut lower_right);
            }
            out.push(ir::Cmd::AndThen { left: lower_left, right: lower_right });
        }
        ast::Stmt::OrElse { left, right } => {
            let mut lower_left = Vec::new();
            for s in left {
                lower_stmt(s, &mut lower_left);
            }
            let mut lower_right = Vec::new();
            for s in right {
                lower_stmt(s, &mut lower_right);
            }
            out.push(ir::Cmd::OrElse { left: lower_left, right: lower_right });
        }
        ast::Stmt::WithCwd { path, body } => {
            let lowered_path = lower_expr(path);
            let mut lower_body = Vec::new();
            for s in body {
                lower_stmt(s, &mut lower_body);
            }
            out.push(ir::Cmd::WithCwd {
                path: lowered_path,
                body: lower_body,
            });
        }
        ast::Stmt::WithLog { path, append, body } => {
            let lowered_path = lower_expr(path);
            let mut lower_body = Vec::new();
            for s in body {
                lower_stmt(s, &mut lower_body);
            }
            out.push(ir::Cmd::WithLog {
                path: lowered_path,
                append,
                body: lower_body,
            });
        }
        ast::Stmt::Cd { path } => {
            out.push(ir::Cmd::Cd(lower_expr(path)));
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
                 let path = lower_expr(iter.next().unwrap());
                 let env = lower_expr(iter.next().unwrap());
                 out.push(ir::Cmd::SaveEnvfile { path, env });
            } else if name == "load_envfile" {
                 panic!("load_envfile() returns a value; use it in an expression (e.g., let m = load_envfile(\"env.meta\"))");
            } else {
                let args = args.iter().map(|e| lower_expr(e.clone())).collect();
                out.push(ir::Cmd::Call { name: name.clone(), args });
            }
        }
        ast::Stmt::Subshell { body } => {
            let mut lowered = Vec::new();
            for s in body {
                lower_stmt(s, &mut lowered);
            }
            out.push(ir::Cmd::Subshell { body: lowered });
        }
        ast::Stmt::Group { body } => {
            let mut lowered = Vec::new();
            for s in body {
                lower_stmt(s, &mut lowered);
            }
            out.push(ir::Cmd::Group { body: lowered });
        }
        ast::Stmt::WithRedirect { stdout, stderr, stdin, body } => {
             let mut lowered_body = Vec::new();
             for s in body {
                 lower_stmt(s, &mut lowered_body);
             }
             
             let lower_target = |t: ast::RedirectTarget| match t {
                 ast::RedirectTarget::File { path, append } => ir::RedirectTarget::File { path: lower_expr(path), append },
                 ast::RedirectTarget::HereDoc { content } => ir::RedirectTarget::HereDoc { content },
                 ast::RedirectTarget::Stdout => ir::RedirectTarget::Stdout,
                 ast::RedirectTarget::Stderr => ir::RedirectTarget::Stderr,
             };
             
             out.push(ir::Cmd::WithRedirect {
                  stdout: stdout.map(lower_target),
                 stderr: stderr.map(lower_target),
                 stdin: stdin.map(lower_target),
                 body: lowered_body,
             });
        }
        ast::Stmt::Spawn { stmt } => {
            let mut lower_cmds = Vec::new();
            lower_stmt(*stmt, &mut lower_cmds);
            
            if lower_cmds.len() == 1 {
                 out.push(ir::Cmd::Spawn(Box::new(lower_cmds.remove(0))));
            } else {
                 out.push(ir::Cmd::Spawn(Box::new(ir::Cmd::Group { body: lower_cmds })));
            }
        }
        ast::Stmt::Wait(expr) => {
            out.push(ir::Cmd::Wait(expr.map(lower_expr)));
        }
        ast::Stmt::TryCatch { try_body, catch_body } => {
            let mut lower_try = Vec::new();
            for s in try_body {
                lower_stmt(s, &mut lower_try);
            }
            let mut lower_catch = Vec::new();
            for s in catch_body {
                lower_stmt(s, &mut lower_catch);
            }
            out.push(ir::Cmd::TryCatch { try_body: lower_try, catch_body: lower_catch });
        }
        ast::Stmt::Export { name, value } => {
            out.push(ir::Cmd::Export {
                name,
                value: value.map(lower_expr),
            });
        }
        ast::Stmt::Unset { name } => {
            out.push(ir::Cmd::Unset(name));
        }
        ast::Stmt::Source { path } => {
            out.push(ir::Cmd::Source(lower_expr(path)));
        }
        ast::Stmt::Exec(args) => {
            out.push(ir::Cmd::ExecReplace(args.into_iter().map(lower_expr).collect()));
        }
        ast::Stmt::Set { target, value } => {
             match target {
                 ast::LValue::Var(name) => {
                     out.push(ir::Cmd::Assign(name, lower_expr(value)));
                 }
                  ast::LValue::Env(name) => {
                      // 1. Pre-check for obvious invalid types to give good errors before lowering
                      if matches!(&value, ast::Expr::List(_) | ast::Expr::Args) {
                          panic!("set env.<NAME> requires a scalar string/number; lists/args are not supported");
                      }

                      let val = lower_expr(value);

                      // 2. Post-check for types that lowered into lists (e.g. variable references that happen to be lists, though lower_expr usually emits Var for those, type check happens at runtime for some, but if we can detect usage of list-like constructs here)
                      // Actually, Val::List comes from literal lists. Val::Args comes from `args`. 
                      // Lowering generally preserves structure.
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
                    lower_stmt(s, &mut lowered);
                }
                lower_segments.push(lowered);
            }
            out.push(ir::Cmd::PipeBlocks(lower_segments));
        }
    }
}

fn lower_expr(e: ast::Expr) -> ir::Val {
    match e {
        ast::Expr::Literal(s) => ir::Val::Literal(s),
        ast::Expr::Var(s) => ir::Val::Var(s),
        ast::Expr::Concat(l, r) => ir::Val::Concat(Box::new(lower_expr(*l)), Box::new(lower_expr(*r))),
        ast::Expr::Arith { left, op, right } => {
            // HACK: If op is Add and either side is a string literal, lower to Concat to preserve legacy string behavior.
            // This is a static heuristic because we don't have types.
            if matches!(op, ast::ArithOp::Add) {
                let l_is_lit = matches!(*left, ast::Expr::Literal(_));
                let r_is_lit = matches!(*right, ast::Expr::Literal(_));
                if l_is_lit || r_is_lit {
                    return ir::Val::Concat(Box::new(lower_expr(*left)), Box::new(lower_expr(*right)));
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
                left: Box::new(lower_expr(*left)),
                op,
                right: Box::new(lower_expr(*right)),
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
                left: Box::new(lower_expr(*left)),
                op,
                right: Box::new(lower_expr(*right)),
            }
        }
        ast::Expr::And(left, right) => {
            ir::Val::And(Box::new(lower_expr(*left)), Box::new(lower_expr(*right)))
        }
        ast::Expr::Or(left, right) => {
            ir::Val::Or(Box::new(lower_expr(*left)), Box::new(lower_expr(*right)))
        }
        ast::Expr::Not(expr) => {
            ir::Val::Not(Box::new(lower_expr(*expr)))
        }
        ast::Expr::Exists(path) => {
            ir::Val::Exists(Box::new(lower_expr(*path)))
        }
        ast::Expr::IsDir(path) => {
            ir::Val::IsDir(Box::new(lower_expr(*path)))
        }
        ast::Expr::IsFile(path) => {
            ir::Val::IsFile(Box::new(lower_expr(*path)))
        }
        ast::Expr::IsSymlink(path) => {
            ir::Val::IsSymlink(Box::new(lower_expr(*path)))
        }
        ast::Expr::IsExec(path) => {
            ir::Val::IsExec(Box::new(lower_expr(*path)))
        }
        ast::Expr::IsReadable(path) => {
            ir::Val::IsReadable(Box::new(lower_expr(*path)))
        }
        ast::Expr::IsWritable(path) => {
            ir::Val::IsWritable(Box::new(lower_expr(*path)))
        }
        ast::Expr::IsNonEmpty(path) => {
            ir::Val::IsNonEmpty(Box::new(lower_expr(*path)))
        }
        ast::Expr::BoolStr(inner) => {
            ir::Val::BoolStr(Box::new(lower_expr(*inner)))
        }
        ast::Expr::Len(expr) => {
            ir::Val::Len(Box::new(lower_expr(*expr)))
        }
        ast::Expr::Arg(n) => ir::Val::Arg(n),
        ast::Expr::Index { list, index } => ir::Val::Index {
            list: Box::new(lower_expr(*list)),
            index: Box::new(lower_expr(*index)),
        },
        ast::Expr::Field { base, name } => {
            let b = lower_expr(*base);
            match name.as_str() {
                "flags" => ir::Val::ArgsFlags(Box::new(b)),
                "positionals" => ir::Val::ArgsPositionals(Box::new(b)),
                _ => panic!("Unknown field '{}'. Only 'flags' and 'positionals' are supported on args object.", name),
            }
        },
        ast::Expr::Join { list, sep } => ir::Val::Join {
            list: Box::new(lower_expr(*list)),
            sep: Box::new(lower_expr(*sep)),
        },
        ast::Expr::Count(inner) => ir::Val::Count(Box::new(lower_expr(*inner))),
        ast::Expr::Bool(b) => ir::Val::Bool(b),
        ast::Expr::Number(n) => ir::Val::Number(n),
        ast::Expr::Command(args) => {
            let lowered_args = args.into_iter().map(lower_expr).collect();
            ir::Val::Command(lowered_args)
        }
        ast::Expr::CommandPipe(segments) => {
            let lowered_segments = segments.into_iter()
                .map(|seg| seg.into_iter().map(lower_expr).collect())
                .collect();
            ir::Val::CommandPipe(lowered_segments)
        }
        ast::Expr::List(exprs) => {
            let lowered_exprs = exprs.into_iter().map(lower_expr).collect();
            ir::Val::List(lowered_exprs)
        }
        ast::Expr::Args => ir::Val::Args,
        ast::Expr::Status => ir::Val::Status,
        ast::Expr::Pid => ir::Val::Pid,
        ast::Expr::Env(inner) => ir::Val::Env(Box::new(lower_expr(*inner))),
        ast::Expr::Uid => ir::Val::Uid,
        ast::Expr::Ppid => ir::Val::Ppid,
        ast::Expr::Pwd => ir::Val::Pwd,
        ast::Expr::SelfPid => ir::Val::SelfPid,
        ast::Expr::Argv0 => ir::Val::Argv0,
        ast::Expr::Argc => ir::Val::Argc,
        ast::Expr::EnvDot(name) => ir::Val::EnvDot(name),
        ast::Expr::Input(e) => ir::Val::Input(Box::new(lower_expr(*e))),
        ast::Expr::Confirm(e) => ir::Val::Confirm(Box::new(lower_expr(*e))),
        ast::Expr::Call { name, args } => {
            if name == "matches" {
                if args.len() != 2 {
                    panic!("matches() requires exactly 2 arguments (text, regex)");
                }
                let mut iter = args.into_iter();
                let text = Box::new(lower_expr(iter.next().unwrap()));
                let regex = Box::new(lower_expr(iter.next().unwrap()));
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
                let path = lower_expr(args.into_iter().next().unwrap());
                ir::Val::LoadEnvfile(Box::new(path))
            } else if name == "json_kv" {
                if args.len() != 1 {
                    panic!("json_kv() requires exactly 1 argument (pairs_blob)");
                }
                let blob = lower_expr(args.into_iter().next().unwrap());
                ir::Val::JsonKv(Box::new(blob))
            } else if name == "save_envfile" {
                 panic!("save_envfile() is a statement; use it as a standalone call");
            } else {
                let lowered_args = args.into_iter().map(lower_expr).collect();
                ir::Val::Call { name, args: lowered_args }
            }
        }
    }
}
