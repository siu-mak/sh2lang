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

        ast::Stmt::Run(args) => {
            let ir_args = args.into_iter().map(lower_expr).collect();
            out.push(ir::Cmd::Exec(ir_args));
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
            for args in segments {
                let lowered_args = args.into_iter().map(lower_expr).collect();
                lowered_segments.push(lowered_args);
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
            let args = args.iter().map(|e| lower_expr(e.clone())).collect();
            out.push(ir::Cmd::Call { name: name.clone(), args });
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
        ast::Stmt::AndThen { left, right } => {
            let mut l_cmds = Vec::new();
            lower_stmt(*left, &mut l_cmds);
            let mut r_cmds = Vec::new();
            lower_stmt(*right, &mut r_cmds);
            out.push(ir::Cmd::AndThen { left: l_cmds, right: r_cmds });
        }
        ast::Stmt::OrElse { left, right } => {
            let mut l_cmds = Vec::new();
            lower_stmt(*left, &mut l_cmds);
            let mut r_cmds = Vec::new();
            lower_stmt(*right, &mut r_cmds);
            out.push(ir::Cmd::OrElse { left: l_cmds, right: r_cmds });
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
        ast::Expr::Len(expr) => {
            ir::Val::Len(Box::new(lower_expr(*expr)))
        }
        ast::Expr::Arg(n) => ir::Val::Arg(n),
        ast::Expr::Index { list, index } => ir::Val::Index {
            list: Box::new(lower_expr(*list)),
            index: Box::new(lower_expr(*index)),
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
    }
}
