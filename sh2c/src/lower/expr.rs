use crate::ast;
use crate::builtins;
use crate::ir;
use crate::span::{Span, SourceMap};
use crate::error::CompileError;
use super::{LoweringContext, LowerOptions, resolve_span};
use super::sudo::{lower_run_call_args, lower_sudo_command};

fn validate_arg_index_expr(
    expr: &ast::Expr,
    sm: &SourceMap,
    file: &str,
    opts: &LowerOptions,
) -> Result<(), CompileError> {
    match &expr.node {
        ast::ExprKind::Number(_) | ast::ExprKind::Var(_) | ast::ExprKind::Arith { .. } => Ok(()),
        // Grouping/Parens are implicit in AST structure or handled by parser
        
        k => {
            let kind_name = match k {
                ast::ExprKind::Literal(_) => "StringLiteral",
                ast::ExprKind::Call { .. } => "Call",
                ast::ExprKind::Arg(_) => "Arg", // Nested arg
                ast::ExprKind::List(_) => "List",
                ast::ExprKind::MapLiteral(_) => "MapLiteral",
                ast::ExprKind::Capture { .. } => "Capture",
                ast::ExprKind::Command(_) => "Command",
                ast::ExprKind::Compare { .. } | ast::ExprKind::And(..) | ast::ExprKind::Or(..) | ast::ExprKind::Not(..) => "BooleanExpression",
                _ => "InvalidExpression", 
            };
            
            Err(CompileError::new(sm.format_diagnostic(
                file,
                opts.diag_base_dir.as_deref(),
                &format!(
                    "arg(expr) index must be an integer expression (variable, number, or arithmetic), got {}",
                    kind_name
                ),
                expr.span,
            )))
        }
    }
}


pub(super) fn lower_expr<'a>(e: ast::Expr, out: &mut Vec<ir::Cmd>, ctx: &mut LoweringContext<'a>, sm: &SourceMap, file: &str) -> Result<ir::Val, CompileError> {
    let opts = ctx.opts(); // Get opts from context for diagnostic formatting
    match e.node {
        ast::ExprKind::Literal(s) => Ok(ir::Val::Literal(s)),
        ast::ExprKind::Var(s) => {
            if ctx.is_bool_var(&s) {
                Ok(ir::Val::BoolVar(s))
            } else {
                Ok(ir::Val::Var(s))
            }
        }
        ast::ExprKind::QualifiedCall { .. } => unreachable!("QualifiedCall should be resolved by loader before lowering"),
        ast::ExprKind::QualifiedCommandWord { .. } => unreachable!("QualifiedCommandWord should be resolved by loader before lowering"),
        ast::ExprKind::Concat(l, r) => Ok(ir::Val::Concat(
            Box::new(lower_expr(*l, out, ctx, sm, file)?),
            Box::new(lower_expr(*r, out, ctx, sm, file)?),
        )),
        ast::ExprKind::Arith { left, op, right } => {
            if matches!(op, ast::ArithOp::Add) {
                let l_is_lit = matches!(left.node, ast::ExprKind::Literal(_));
                let r_is_lit = matches!(right.node, ast::ExprKind::Literal(_));
                if l_is_lit || r_is_lit {
                    return Ok(ir::Val::Concat(
                        Box::new(lower_expr(*left, out, ctx, sm, file)?),
                        Box::new(lower_expr(*right, out, ctx, sm, file)?),
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
                left: Box::new(lower_expr(*left, out, ctx, sm, file)?),
                op,
                right: Box::new(lower_expr(*right, out, ctx, sm, file)?),
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

                let pred_val = lower_expr(pred_expr, out, ctx, sm, file)?;
                
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
                    left: Box::new(lower_expr(*left, out, ctx, sm, file)?),
                    op,
                    right: Box::new(lower_expr(*right, out, ctx, sm, file)?),
                })
            }
        }
        ast::ExprKind::And(left, right) => Ok(ir::Val::And(
            Box::new(lower_expr(*left, out, ctx, sm, file)?),
            Box::new(lower_expr(*right, out, ctx, sm, file)?),
        )),
        ast::ExprKind::Or(left, right) => Ok(ir::Val::Or(
            Box::new(lower_expr(*left, out, ctx, sm, file)?),
            Box::new(lower_expr(*right, out, ctx, sm, file)?),
        )),
        ast::ExprKind::Not(expr) => Ok(ir::Val::Not(Box::new(lower_expr(*expr, out, ctx, sm, file)?))),
        ast::ExprKind::Exists(path) => Ok(ir::Val::Exists(Box::new(lower_expr(*path, out, ctx, sm, file)?))),
        ast::ExprKind::IsDir(path) => Ok(ir::Val::IsDir(Box::new(lower_expr(*path, out, ctx, sm, file)?))),
        ast::ExprKind::IsFile(path) => Ok(ir::Val::IsFile(Box::new(lower_expr(*path, out, ctx, sm, file)?))),
        ast::ExprKind::IsSymlink(path) => {
            Ok(ir::Val::IsSymlink(Box::new(lower_expr(*path, out, ctx, sm, file)?)))
        }
        ast::ExprKind::IsExec(path) => Ok(ir::Val::IsExec(Box::new(lower_expr(*path, out, ctx, sm, file)?))),
        ast::ExprKind::IsReadable(path) => {
            Ok(ir::Val::IsReadable(Box::new(lower_expr(*path, out, ctx, sm, file)?)))
        }
        ast::ExprKind::IsWritable(path) => {
            Ok(ir::Val::IsWritable(Box::new(lower_expr(*path, out, ctx, sm, file)?)))
        }
        ast::ExprKind::IsNonEmpty(path) => {
            Ok(ir::Val::IsNonEmpty(Box::new(lower_expr(*path, out, ctx, sm, file)?)))
        }
        ast::ExprKind::BoolStr(inner) => {
            Ok(ir::Val::BoolStr(Box::new(lower_expr(*inner, out, ctx, sm, file)?)))
        }
        ast::ExprKind::Len(expr) => Ok(ir::Val::Len(Box::new(lower_expr(*expr, out, ctx, sm, file)?))),
        ast::ExprKind::Arg(expr) => {
            validate_arg_index_expr(&expr, sm, file, ctx.opts())?;

            let index_val = lower_expr(*expr, out, ctx, sm, file)?;
            
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
            list: Box::new(lower_expr(*list, out, ctx, sm, file)?),
            index: Box::new(lower_expr(*index, out, ctx, sm, file)?),
        }),
        ast::ExprKind::Field { base, name } => {
            let b = lower_expr(*base, out, ctx, sm, file)?;

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
            list: Box::new(lower_expr(*list, out, ctx, sm, file)?),
            sep: Box::new(lower_expr(*sep, out, ctx, sm, file)?),
        }),
        ast::ExprKind::Count(inner) => Ok(ir::Val::Count(Box::new(lower_expr(*inner, out, ctx, sm, file)?))),
        ast::ExprKind::Bool(b) => Ok(ir::Val::Bool(b)),
        ast::ExprKind::Number(n) => Ok(ir::Val::Number(n)),
        ast::ExprKind::Command(args) => {
            // Ticket 10: sh("cmd") shorthand in command substitution.
            // The parser emits ExprKind::Sh as a single arg for sh("cmd") shorthand
            // (with LParen). Delegate directly to the existing Sh lowering which
            // correctly produces [sh, -c, cmd].
            //
            // Bare `sh file` (no parens) produces [Literal("sh"), Literal("file")]
            // and is NOT affected — it lowers as a normal command.
            if args.len() == 1 {
                if matches!(&args[0].node, ast::ExprKind::Sh { .. }) {
                    return lower_expr(args.into_iter().next().unwrap(), out, ctx, sm, file);
                }
            }
            let lowered_args = args
                .into_iter()
                .map(|a| lower_expr(a, out, ctx, sm, file))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ir::Val::Command(lowered_args))
        }
        ast::ExprKind::CommandPipe(segments) => {
            let lowered_segments = segments
                .into_iter()
                .map(|seg| {
                    seg.into_iter()
                        .map(|a| lower_expr(a, out, ctx, sm, file))
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ir::Val::CommandPipe(lowered_segments))
        }
        ast::ExprKind::List(exprs) => {
            let lowered_exprs = exprs
                .into_iter()
                .map(|e| lower_expr(e, out, ctx, sm, file))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ir::Val::List(lowered_exprs))
        }
        ast::ExprKind::Args => Ok(ir::Val::Args),
        ast::ExprKind::Status => Ok(ir::Val::Status),
        ast::ExprKind::Pid => Ok(ir::Val::Pid),
        ast::ExprKind::Env(inner) => Ok(ir::Val::Env(Box::new(lower_expr(*inner, out, ctx, sm, file)?))),
        ast::ExprKind::Uid => Ok(ir::Val::Uid),
        ast::ExprKind::Ppid => Ok(ir::Val::Ppid),
        ast::ExprKind::Pwd => Ok(ir::Val::Pwd),
        ast::ExprKind::SelfPid => Ok(ir::Val::SelfPid),
        ast::ExprKind::Argv0 => Ok(ir::Val::Argv0),
        ast::ExprKind::Argc => Ok(ir::Val::Argc),
        ast::ExprKind::EnvDot(name) => Ok(ir::Val::EnvDot(name)),
        ast::ExprKind::Input(e) => Ok(ir::Val::Input(Box::new(lower_expr(*e, out, ctx, sm, file)?))),
        ast::ExprKind::Confirm { prompt, default } => {
            let prompt_val = lower_expr(*prompt, out, ctx, sm, file)?;
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
            let (argv, allow_fail_span) = lower_sudo_command(args, options, out, ctx, sm, file)?;
            
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
        ast::ExprKind::Run(run_call) => {
            // Validate options as safety net
            for opt in &run_call.options {
                if opt.name == "allow_fail" {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "Internal error: allow_fail should be hoisted to capture; please report this bug",
                        opt.span,
                    )));
                } else {
                     return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        &format!("unknown run() option '{}'", opt.name),
                        opt.span,
                    )));
                }
            }

            let lowered_args = run_call.args
                .into_iter()
                .map(|a| lower_expr(a, out, ctx, sm, file))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ir::Val::Command(lowered_args))
        }

        ast::ExprKind::Call { name, args, options } => {
            if name == "argv" {
                if !args.is_empty() {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "argv() takes no arguments",
                        e.span,
                    )));
                }
                Ok(ir::Val::Args)

            } else if name == "matches" {
                if args.len() != 2 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "matches() requires exactly 2 arguments (text, regex)",
                        e.span,
                    )));
                }
                let mut iter = args.into_iter();
                let text = Box::new(lower_expr(iter.next().unwrap(), out, ctx, sm, file)?);
                let regex = Box::new(lower_expr(iter.next().unwrap(), out, ctx, sm, file)?);
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
                let list = Box::new(lower_expr(iter.next().unwrap(), out, ctx, sm, file)?);
                let needle = Box::new(lower_expr(iter.next().unwrap(), out, ctx, sm, file)?);
                
                // Static Dispatch:
                // 1. Check if haystack is definitely a list (Literal, Split, Lines, or known list var).
                let is_list = match &*list {
                    ir::Val::List(_) | ir::Val::Split { .. } | ir::Val::Lines(_) => true,
                    ir::Val::Var(n) => ctx.is_list_var(n),
                    _ => false,
                };

                // 2. Validate needle is a scalar (string/bool/num), not a list/map.
                if matches!(&*needle, ir::Val::List(_) | ir::Val::Args) {
                     return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "contains() needle must be a scalar value (string), found list",
                        e.span,
                    )));
                }

                if is_list {
                    // Materialize list (Path B change)
                    let list_val = if let ir::Val::Var(_) = *list {
                         list
                    } else {
                        // Generate unique temp name
                        ctx.tmp_counter += 1;
                        let tmp_name = format!("__sh2_tmp_list_{}", ctx.tmp_counter);
                        ctx.insert_list_var(&tmp_name);
                        
                        out.push(ir::Cmd::Assign(
                            tmp_name.clone(),
                            *list,
                            None
                        ));
                        
                        Box::new(ir::Val::Var(tmp_name))
                    };
                    Ok(ir::Val::ContainsList { list: list_val, needle })
                } else {
                    Ok(ir::Val::ContainsSubstring { haystack: list, needle })
                }
            } else if name == "contains_line" {
                if args.len() != 2 {
                     return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "contains_line() requires exactly 2 arguments (file, needle)",
                        e.span,
                    )));
                }
                let mut iter = args.into_iter();
                let file_val = Box::new(lower_expr(iter.next().unwrap(), out, ctx, sm, file)?);
                let needle = Box::new(lower_expr(iter.next().unwrap(), out, ctx, sm, file)?);
                Ok(ir::Val::ContainsLine { file: file_val, needle })
            } else if name == "starts_with" {
                if args.len() != 2 {
                     return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "starts_with() requires exactly 2 arguments (text, prefix)",
                        e.span,
                    )));
                }
                let mut iter = args.into_iter();
                let text = Box::new(lower_expr(iter.next().unwrap(), out, ctx, sm, file)?);
                let prefix = Box::new(lower_expr(iter.next().unwrap(), out, ctx, sm, file)?);
                Ok(ir::Val::StartsWith { text, prefix })
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
                let path = lower_expr(args.into_iter().next().unwrap(), out, ctx, sm, file)?;
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
                let blob = lower_expr(args.into_iter().next().unwrap(), out, ctx, sm, file)?;
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
                let arg = lower_expr(args.into_iter().next().unwrap(), out, ctx, sm, file)?;
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
                let arg = lower_expr(args.into_iter().next().unwrap(), out, ctx, sm, file)?;
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
                    .map(|a| lower_expr(a, out, ctx, sm, file))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ir::Val::PathJoin(lowered_args))
            } else if name == "lines" {
                if args.len() != 1 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "lines() requires exactly 1 argument (the source)",
                        e.span,
                    )));
                }
                let arg = lower_expr(args.into_iter().next().unwrap(), out, ctx, sm, file)?;
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
                let s = lower_expr(iter.next().unwrap(), out, ctx, sm, file)?;
                let delim = lower_expr(iter.next().unwrap(), out, ctx, sm, file)?;
                Ok(ir::Val::Split {
                    s: Box::new(s),
                    delim: Box::new(delim),
                })
            } else if name == "glob" {
                if args.len() != 1 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "glob() requires exactly 1 argument (pattern)",
                        e.span,
                    )));
                }
                let arg = lower_expr(args.into_iter().next().unwrap(), out, ctx, sm, file)?;
                Ok(ir::Val::Glob(Box::new(arg)))
            } else if name == "find_files" {
                // Ensure no positional arguments
                if !args.is_empty() {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "find_files() does not accept positional arguments. Use named arguments: find_files(dir=\"...\", name=\"...\")",
                        e.span,
                    )));
                }

                // Process options
                let mut dir = None;
                let mut name_arg = None;

                for opt in options {
                    match opt.name.as_str() {
                        "dir" => {
                            if dir.is_some() {
                                return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "Duplicate argument 'dir'", opt.span)));
                            }
                            dir = Some(lower_expr(opt.value, out, ctx, sm, file)?);
                        }
                        "name" => {
                            if name_arg.is_some() {
                                return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "Duplicate argument 'name'", opt.span)));
                            }
                            name_arg = Some(lower_expr(opt.value, out, ctx, sm, file)?);
                        }
                        _ => {
                            return Err(CompileError::new(sm.format_diagnostic(
                                file, 
                                opts.diag_base_dir.as_deref(), 
                                format!("Unknown argument '{}'. Supported: dir, name", opt.name).as_str(), 
                                opt.span
                            )));
                        }
                    }
                }

                // Apply defaults
                let dir_val = dir.unwrap_or(ir::Val::Literal(".".to_string()));
                let name_val = name_arg.unwrap_or(ir::Val::Literal("*".to_string()));

                Ok(ir::Val::FindFiles {
                    dir: Box::new(dir_val),
                    name: Box::new(name_val),
                })
            } else if name == "spawn" {
                // spawn(run(...)) or spawn(sudo(...)) - returns PID
                if args.len() != 1 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "spawn() requires exactly 1 argument: run(...) or sudo(...)",
                        e.span,
                    )));
                }
                if !options.is_empty() {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "spawn() does not accept named arguments",
                        options[0].span,
                    )));
                }
                
                let inner_expr = args.into_iter().next().unwrap();
                let loc = Some(resolve_span(inner_expr.span, sm, file, opts.diag_base_dir.as_deref()));
                
                // Validate it's run() or sudo()
                match &inner_expr.node {
                    ast::ExprKind::Run(run_call) => {
                        // LEVERAGE lower_run_call_args to handle/reject options like 'shell' or 'allow_fail'
                        // We clone ctx because lower_run_call_args mutates it (e.g. for dynamic args?) 
                        // but generally we want side effects (like defined vars? no run call args are expressions).
                        // Actually lower_run_call_args takes &mut ctx.
                        
                        let (lowered_args, allow_fail) = lower_run_call_args(run_call, out, ctx, sm, file, opts)?;
                        
                        // Reject allow_fail in spawn() because it's ambiguous/redundant with wait checking
                        if allow_fail {
                             return Err(CompileError::new(sm.format_diagnostic(
                                file,
                                opts.diag_base_dir.as_deref(),
                                "allow_fail=true is not supported in spawn(); use wait(pid, allow_fail=true) instead",
                                run_call.options.iter().find(|o| o.name == "allow_fail").map(|o| o.span).unwrap_or(inner_expr.span),
                            )));
                        }
                        
                        Ok(ir::Val::Spawn { args: lowered_args, loc })
                    }
                    ast::ExprKind::Sudo { args: sudo_args, options: sudo_opts } => {
                        // Lower sudo command (reuse existing helper logic)
                        let (argv, allow_fail_span) = lower_sudo_command(
                            sudo_args.clone(),
                            sudo_opts.clone(),
                            out,
                            ctx,
                            sm,
                            file,
                        )?;
                        
                        // Reject allow_fail in sudo inside spawn
                        if let Some(span) = allow_fail_span {
                            return Err(CompileError::new(sm.format_diagnostic(
                                file,
                                opts.diag_base_dir.as_deref(),
                                "allow_fail=true is not supported in spawn(); use wait(pid, allow_fail=true) instead",
                                span,
                            )));
                        }
                        
                        Ok(ir::Val::Spawn { args: argv, loc })
                    }
                    _ => {
                        Err(CompileError::new(sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            "spawn() only accepts run(...) or sudo(...) as its argument",
                            inner_expr.span,
                        )))
                    }
                }
            } else if name == "wait" {
                // wait(pid) or wait(pid, allow_fail=true)
                if args.len() != 1 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "wait() requires exactly 1 positional argument (the PID)",
                        e.span,
                    )));
                }
                
                // Location for wait() call itself
                let loc = Some(resolve_span(e.span, sm, file, opts.diag_base_dir.as_deref()));

                // Process options
                let mut allow_fail = false;
                for opt in options {
                    match opt.name.as_str() {
                        "allow_fail" => {
                            match &opt.value.node {
                                ast::ExprKind::Bool(b) => {
                                    allow_fail = *b;
                                }
                                _ => {
                                    return Err(CompileError::new(sm.format_diagnostic(
                                        file,
                                        opts.diag_base_dir.as_deref(),
                                        "allow_fail must be a boolean literal (true or false)",
                                        opt.value.span,
                                    )));
                                }
                            }
                        }
                        _ => {
                            return Err(CompileError::new(sm.format_diagnostic(
                                file,
                                opts.diag_base_dir.as_deref(),
                                &format!("Unknown option '{}'. Supported: allow_fail", opt.name),
                                opt.span,
                            )));
                        }
                    }
                }
                
                let pid_val = lower_expr(args.into_iter().next().unwrap(), out, ctx, sm, file)?;
                Ok(ir::Val::Wait {
                    pid: Box::new(pid_val),
                    allow_fail,
                    loc,
                })
            } else if name == "wait_all" {
                // wait_all(pids) or wait_all(pids, allow_fail=true)
                // Lowers to loop IR that waits for all PIDs and returns first non-zero exit code
                if args.len() != 1 {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        "wait_all() requires exactly 1 positional argument (a list of PIDs)",
                        e.span,
                    )));
                }
                
                let loc = Some(resolve_span(e.span, sm, file, opts.diag_base_dir.as_deref()));
                
                // Process options
                let mut allow_fail = false;
                for opt in options {
                    match opt.name.as_str() {
                        "allow_fail" => {
                            match &opt.value.node {
                                ast::ExprKind::Bool(b) => {
                                    allow_fail = *b;
                                }
                                _ => {
                                    return Err(CompileError::new(sm.format_diagnostic(
                                        file,
                                        opts.diag_base_dir.as_deref(),
                                        "allow_fail must be a boolean literal (true or false)",
                                        opt.value.span,
                                    )));
                                }
                            }
                        }
                        _ => {
                            return Err(CompileError::new(sm.format_diagnostic(
                                file,
                                opts.diag_base_dir.as_deref(),
                                &format!("Unknown option '{}'. Supported: allow_fail", opt.name),
                                opt.span,
                            )));
                        }
                    }
                }
                
                // Lower the pids argument (must be a list)
                let pids_expr = args.into_iter().next().unwrap();
                let pids_val = lower_expr(pids_expr.clone(), out, ctx, sm, file)?;
                
                // Validate it's a list - target-aware check
                match &pids_expr.node {
                    ast::ExprKind::List(_) => { /* ok for all targets */ }
                    ast::ExprKind::Var(_) | ast::ExprKind::Call { .. } => {
                        // Only allow on Bash target - POSIX requires inline list literals
                        if opts.target == crate::codegen::TargetShell::Posix {
                            return Err(CompileError::new(sm.format_diagnostic(
                                file,
                                opts.diag_base_dir.as_deref(),
                                "wait_all() on --target posix requires an inline list literal, e.g. wait_all([p1, p2])",
                                pids_expr.span,
                            )));
                        }
                        // Bash: assume ok - could be a list variable or list-returning call
                    }
                    _ => {
                        return Err(CompileError::new(sm.format_diagnostic(
                            file,
                            opts.diag_base_dir.as_deref(),
                            "wait_all() requires a list of PIDs",
                            pids_expr.span,
                        )));
                    }
                }
                
                // Return a special WaitAll value that codegen will handle
                Ok(ir::Val::WaitAll {
                    pids: Box::new(pids_val),
                    allow_fail,
                    loc,
                })
            } else if name == "save_envfile" {
                return Err(CompileError::new(sm.format_diagnostic(
                    file,
                    opts.diag_base_dir.as_deref(),
                    "save_envfile() is a statement; use it as a standalone call",
                    e.span,
                )));
            } else {
                // Validate: must be user-defined or a prelude helper
                // SAFETY: EXPR_BUILTINS should have been handled earlier in this match chain.
                // If one reaches here, it's an internal compiler bug.
                if builtins::is_expr_builtin(&name) {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        &format!(
                            "internal error: builtin `{}` not lowered correctly \
                             (please report this bug)",
                            name
                        ),
                        e.span,
                    )));
                }
                if !ctx.user_funcs.contains(&name) && !builtins::is_prelude_helper(&name) {
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        &format!(
                            "unknown function `{}` (use run(\"{}\", ...) for external commands, \
                             or define func {}(...) {{ ... }})",
                            name, name, name
                        ),
                        e.span,
                    )));
                }
                let lowered_args = args
                    .into_iter()
                    .map(|a| lower_expr(a, out, ctx, sm, file))
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
                     let v = lower_expr(v, out, ctx, sm, file)?;
                     Ok((k, v))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ir::Val::MapLiteral(lowered_entries))
        }
        ast::ExprKind::MapIndex { map, key } => Ok(ir::Val::MapIndex { map, key }),
        ast::ExprKind::Capture { expr, options } => {
            let expr_span = expr.span; // Capture span before move
            // Path C: Hoisting is handled by parser for nested run(...) inside $(...).
            // However, we preserve the context check or explicit logic if needed.
            // Since Parser flattens run() into Command w/ hoisted options,
            // `expr` here will be Command, not Call.
            // So we just process explicit capture options.

            let lowered_expr = lower_expr(*expr, out, ctx, sm, file)?;

            let mut allow_fail = false;
            let mut seen_allow_fail = false;
            let mut allow_fail_span = None;

            for opt in options {
                if opt.name == "allow_fail" {
                    if seen_allow_fail {
                        return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "allow_fail specified more than once", opt.span)));
                    }
                    seen_allow_fail = true;
                    allow_fail_span = Some(opt.span);
                    if let ast::ExprKind::Bool(b) = opt.value.node {
                        allow_fail = b;
                    } else {
                        return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "allow_fail must be a boolean literal", opt.value.span)));
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
                     "capture(..., allow_fail=true) is only allowed in 'let' assignment to ensure exit status can be preserved",
                     allow_fail_span.unwrap_or(expr_span),
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
            let mut args_val = None;
            
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
                } else if opt.name == "args" {
                    // Handle args
                     let val = lower_expr(opt.value, out, ctx, sm, file)?;
                     if matches!(val, ir::Val::Args) {
                         args_val = Some(val);
                     } else {
                          return Err(CompileError::new(sm.format_diagnostic(
                              file,
                              opts.diag_base_dir.as_deref(),
                              "args= must be actual arguments (args() or argv())",
                              opt.span,
                          )));
                     }
                } else {
                    // This shouldn't happen if parser is correct, but handle gracefully
                    return Err(CompileError::new(sm.format_diagnostic(
                        file,
                        opts.diag_base_dir.as_deref(),
                        &format!("unknown sh() option '{}' in expression context; only 'shell' and 'args' are supported", opt.name),
                        opt.span,
                    )));
                }
            }
            
            // Build argv: [shell, "-c", cmd] + optional [target_name, args]
            let shell_val = lower_expr(shell_expr, out, ctx, sm, file)?;
            let dash_c_val = ir::Val::Literal("-c".to_string());
            let cmd_val = lower_expr(*cmd, out, ctx, sm, file)?;
            
            let mut cmd_vec = vec![shell_val, dash_c_val, cmd_val];
            
            if let Some(args) = args_val {
                 let target_literal = match ctx.opts().target {
                     crate::codegen::TargetShell::Bash => "bash",
                     crate::codegen::TargetShell::Posix => "sh",
                 };
                 cmd_vec.push(ir::Val::Literal(target_literal.to_string()));
                 cmd_vec.push(args);
            }
            
            Ok(ir::Val::Command(cmd_vec))
        }
    }
}
