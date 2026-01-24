use crate::ast::*;

pub fn format_program(program: &Program) -> String {
    let mut out = String::new();

    // Imports
    for (i, imp) in program.imports.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&format!("import \"{}\"", imp));
    }
    if !program.imports.is_empty() {
        out.push('\n');
    }

    // Functions
    for (i, func) in program.functions.iter().enumerate() {
        if i > 0 || !program.imports.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(&format_function(func));
    }

    // Top-level statements

    
    // Ensure single trailing newline
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn format_function(func: &Function) -> String {
    let params = func.params.join(", ");
    let body = format_block(&func.body, 1, true);
    format!("func {}({}) {{\n{}\n}}", func.name, params, body)
}

fn indent_str(depth: usize) -> String {
    "    ".repeat(depth)
}

fn format_block(stmts: &[Stmt], depth: usize, force_newline: bool) -> String {
    if stmts.is_empty() && !force_newline {
         return String::new();
    }
    let indent = indent_str(depth);
    let mut out = String::new();
    for (i, stmt) in stmts.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&indent);
        out.push_str(&format_stmt(stmt, depth));
    }
    out
}



// Check if a statement is "simple atomic" for chaining
fn is_simple_atom(stmt: &Stmt) -> bool {
    match &stmt.node {
        StmtKind::Let { .. } |
        StmtKind::Run(_) |
        StmtKind::Exec(_) |
        StmtKind::Print(_) |
        StmtKind::PrintErr(_) |
        StmtKind::Call { .. } |
        StmtKind::Set { .. } |
        StmtKind::Return(_) |
        StmtKind::Exit(_) |
        StmtKind::Break |
        StmtKind::Continue |
        StmtKind::Cd { .. } |
        StmtKind::Export { .. } |
        StmtKind::Unset { .. } |
        StmtKind::Source { .. } |
        StmtKind::Sh(_) |
        StmtKind::Wait(_) => true,
        _ => false,
    }
}



// Refactored: format_stmt returns content *without* leading indentation
// The caller adds indentation unless it's inline.
fn format_stmt(stmt: &Stmt, depth: usize) -> String {
    match &stmt.node {
        StmtKind::Let { name, value } => format!("let {} = {}", name, format_expr(value)),
        StmtKind::Run(call) => format_run_call(call),
        StmtKind::Exec(args) => {
            let parts: Vec<String> = args.iter().map(format_expr).collect();
            format!("exec({})", parts.join(", "))
        }
        StmtKind::Print(e) => format!("print({})", format_expr(e)),
        StmtKind::PrintErr(e) => format!("print_err({})", format_expr(e)),
        StmtKind::If { cond, then_body, elifs, else_body } => {
            let mut s = format!("if {} {{\n{}\n", format_expr(cond), format_block(then_body, depth + 1, false));
            let indent = indent_str(depth);
            for elif in elifs {
                s.push_str(&format!("{}}} elif {} {{\n{}\n", indent, format_expr(&elif.cond), format_block(&elif.body, depth + 1, false)));
            }
            if let Some(else_b) = else_body {
                s.push_str(&format!("{}}} else {{\n{}\n", indent, format_block(else_b, depth + 1, false)));
            }
            s.push_str(&format!("{}}}", indent));
            s
        }
        StmtKind::While { cond, body } => {
            format!("while {} {{\n{}\n{}}}", format_expr(cond), format_block(body, depth + 1, false), indent_str(depth))
        }
        StmtKind::For { var, items, body } => {
            let items_str: Vec<String> = items.iter().map(format_expr).collect();
            let items_joined = if items.len() == 1 { items_str[0].clone() } else { format!("({})", items_str.join(", ")) };
            format!("for {} in {} {{\n{}\n{}}}", var, items_joined, format_block(body, depth + 1, false), indent_str(depth))
        }
        StmtKind::ForMap { key_var, val_var, map, body } => {
            format!("for ({}, {}) in {} {{\n{}\n{}}}", key_var, val_var, map, format_block(body, depth + 1, false), indent_str(depth))
        }
        StmtKind::TryCatch { try_body, catch_body } => {
            format!("try {{\n{}\n{}}} catch {{\n{}\n{}}}",
                format_block(try_body, depth + 1, false),
                indent_str(depth),
                format_block(catch_body, depth + 1, false),
                indent_str(depth)
            )
        }
        StmtKind::Pipe(segments) => {
            let parts: Vec<String> = segments.iter().map(|c| format_run_call(c)).collect();
            parts.join(" | ")
        }
        StmtKind::PipeBlocks { segments } => {
            let parts: Vec<String> = segments.iter().map(|seg| {
                format!("{{\n{}\n{}}}", format_block(seg, depth + 1, false), indent_str(depth))
            }).collect();
            parts.join(" | ")
        }
        StmtKind::Return(opt) => {
             match opt {
                 Some(e) => format!("return {}", format_expr(e)),
                 None => "return".to_string(),
             }
        }
        StmtKind::Exit(opt) => {
             match opt {
                 Some(e) => format!("exit {}", format_expr(e)),
                 None => "exit".to_string(),
             }
        }
        StmtKind::Break => "break".to_string(),
        StmtKind::Continue => "continue".to_string(),
        StmtKind::Cd { path } => format!("cd({})", format_expr(path)),
        StmtKind::Export { name, value } => {
            match value {
                Some(v) => format!("export(\"{}\", {})", name, format_expr(v)),
                None => format!("export(\"{}\")", name),
            }
        }
        StmtKind::Unset { name } => format!("unset(\"{}\")", name),
        StmtKind::Source { path } => format!("source({})", format_expr(path)),
        StmtKind::Sh(expr) => format!("sh({})", format_expr(expr)),
        StmtKind::ShBlock(lines) => {
             // sh { "line1", "line2" }
             let joined = lines.iter().map(|l| format!("\"{}\"", sh_escape(l))).collect::<Vec<_>>().join(", ");
             format!("sh {{ {} }}", joined)
        }
        StmtKind::Call { name, args } => {
            let parts: Vec<String> = args.iter().map(format_expr).collect();
             format!("{}({})", name, parts.join(", "))
        }
        StmtKind::AndThen { left, right } => {
             format_chain(left, right, "&&", depth)
        }
        StmtKind::OrElse { left, right } => {
             format_chain(left, right, "||", depth)
        }
        StmtKind::Set { target, value } => {
             match target {
                 LValue::Var(v) => format!("set {} = {}", v, format_expr(value)),
                 LValue::Env(v) => format!("set env.{} = {}", v, format_expr(value)),
             }
        }
        StmtKind::Case { expr, arms } => {
            let mut s = format!("case {} {{\n", format_expr(expr));
            let indent = indent_str(depth);
            let inner_indent = indent_str(depth + 1);
            for arm in arms {
                 let pats: Vec<String> = arm.patterns.iter().map(|p| match p {
                     Pattern::Literal(s) => format!("\"{}\"", sh_escape(s)),
                     Pattern::Glob(s) => format!("glob(\"{}\")", sh_escape(s)),
                     Pattern::Wildcard => "_".to_string(),
                 }).collect();
                 let pats_str = pats.join(" | ");
                 s.push_str(&format!("{}{} => {{\n{}\n{}}}\n", inner_indent, pats_str, format_block(&arm.body, depth + 2, false), inner_indent));
            }
            s.push_str(&format!("{}}}", indent));
            s
        }
        StmtKind::WithEnv { bindings, body } => {
             let binds: Vec<String> = bindings.iter().map(|(k, v)| format!("{} = {}", k, format_expr(v))).collect();
             format!("with env {{ {} }} {{\n{}\n{}}}", binds.join(", "), format_block(body, depth + 1, false), indent_str(depth))
        }
        StmtKind::WithCwd { path, body } => {
            format!("with cwd({}) {{\n{}\n{}}}", format_expr(path), format_block(body, depth + 1, false), indent_str(depth))
        }
        StmtKind::WithLog { path, append, body } => {
             let opts = if *append { ", append=true" } else { "" };
             format!("with log({}{}) {{\n{}\n{}}}", format_expr(path), opts, format_block(body, depth + 1, false), indent_str(depth))
        }
        StmtKind::WithRedirect { stdout, stderr, stdin, body } => {
            let mut opts = Vec::new();
            if let Some(t) = stdout {
                opts.push(format!("stdout: {}", format_redirect_target(t)));
            }
            if let Some(t) = stderr {
                opts.push(format!("stderr: {}", format_redirect_target(t)));
            }
             if let Some(t) = stdin {
                opts.push(format!("stdin: {}", format_redirect_target(t)));
            }
            format!("with redirect {{ {} }} {{\n{}\n{}}}", opts.join(", "), format_block(body, depth + 1, false), indent_str(depth))
        }
        StmtKind::Subshell { body } => {
            format!("subshell {{\n{}\n{}}}", format_block(body, depth + 1, false), indent_str(depth))
        }
        StmtKind::Group { body } => {
            format!("group {{\n{}\n{}}}", format_block(body, depth + 1, false), indent_str(depth))
        }
        StmtKind::Spawn { stmt } => {
             // Spawn wraps a single stmt, but that stmt effectively can be a group/block
             // If inner is a group, emit spawn { ... }
             // If inner is a Run, emit spawn run(...)
             match &stmt.node {
                 StmtKind::Group { body } => format!("spawn {{\n{}\n{}}}", format_block(body, depth+1, false), indent_str(depth)),
                 _ => format!("spawn {}", format_stmt(stmt, depth)),
             }
        }
        StmtKind::Wait(opt) => {
            match opt {
                Some(e) => format!("wait({})", format_expr(e)),
                None => "wait".to_string(),
            }
        }
    }
}

fn format_redirect_target(t: &RedirectTarget) -> String {
    match t {
         RedirectTarget::Stdout => "stdout".to_string(),
         RedirectTarget::Stderr => "stderr".to_string(),
         RedirectTarget::File { path, append } => {
             if *append {
                 format!("file({}, append=true)", format_expr(path))
             } else {
                 format!("file({})", format_expr(path))
             }
         }
         RedirectTarget::HereDoc { content } => {
             format!("heredoc(\"{}\")", sh_escape(content))
         }
    }
}

fn format_run_call(call: &RunCall) -> String {
    let mut parts: Vec<String> = call.args.iter().map(format_expr).collect();
    for opt in &call.options {
         parts.push(format!("{} = {}", opt.name, format_expr(&opt.value)));
    }
    format!("run({})", parts.join(", "))
}

fn format_expr(expr: &Expr) -> String {
    format_expr_prec(&expr.node, 0)
}

fn format_expr_prec(kind: &ExprKind, min_prec: u8) -> String {
    // Precedence levels:
    // 0: lowest
    // 1: ||
    // 2: &&
    // 3: compare (==, !=, <, etc)
    // 4: concat (&)
    // 5: add/sub
    // 6: mul/div/mod
    // 7: prefix (!, -)
    // 8: suffix/call/atom

    match kind {
        ExprKind::Or(l, r) => {
            wrap_parens(min_prec, 1, format!("{} || {}", format_expr_prec(&l.node, 1), format_expr_prec(&r.node, 1 + 1))) 
            // Bash/sh logical ops are left-associative usually.
        }
        ExprKind::And(l, r) => {
             wrap_parens(min_prec, 2, format!("{} && {}", format_expr_prec(&l.node, 2), format_expr_prec(&r.node, 2 + 1)))
        }
        ExprKind::Compare { left, op, right } => {
            let op_str = match op {
                CompareOp::Eq => "==",
                CompareOp::NotEq => "!=",
                CompareOp::Lt => "<",
                CompareOp::Le => "<=",
                CompareOp::Gt => ">",
                CompareOp::Ge => ">=",
            };
            wrap_parens(min_prec, 3, format!("{} {} {}", format_expr_prec(&left.node, 3), op_str, format_expr_prec(&right.node, 3 + 1)))
        }
        ExprKind::Concat(l, r) => {
             wrap_parens(min_prec, 4, format!("{} & {}", format_expr_prec(&l.node, 4), format_expr_prec(&r.node, 4 + 1)))
        }
        ExprKind::Arith { left, op, right } => {
             let (prec, op_str) = match op {
                 ArithOp::Add => (5, "+"),
                 ArithOp::Sub => (5, "-"),
                 ArithOp::Mul => (6, "*"),
                 ArithOp::Div => (6, "/"),
                 ArithOp::Mod => (6, "%"),
             };
             wrap_parens(min_prec, prec, format!("{} {} {}", format_expr_prec(&left.node, prec), op_str, format_expr_prec(&right.node, prec + 1)))
        }
        ExprKind::Not(e) => {
            wrap_parens(min_prec, 7, format!("!{}", format_expr_prec(&e.node, 7)))
        }
        ExprKind::Literal(s) => format!("\"{}\"", sh_escape(s)),
        ExprKind::Var(s) => s.clone(),
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Number(n) => n.to_string(),
        ExprKind::List(items) => {
            let parts: Vec<String> = items.iter().map(format_expr).collect();
            format!("[{}]", parts.join(", "))
        }
        ExprKind::MapLiteral(entries) => {
             let parts: Vec<String> = entries.iter().map(|(k, v)| format!("\"{}\": {}", sh_escape(k), format_expr(v))).collect();
             format!("{{ {} }}", parts.join(", "))
        }
        ExprKind::Call { name, args } => {
             let parts: Vec<String> = args.iter().map(format_expr).collect();
             format!("{}({})", name, parts.join(", "))
        }
        ExprKind::Command(args) => {
             // $(...) uses Command. Parser expects `capture(run(...))` or `capture(run(...) | ...)`
             let parts: Vec<String> = args.iter().map(format_expr).collect();
             format!("capture(run({}))", parts.join(", "))
        }
        ExprKind::CommandPipe(segs) => {
             // pipe segments: each segment is a run() call separated by |
             let seg_strs: Vec<String> = segs.iter().map(|s| {
                 let args: Vec<String> = s.iter().map(format_expr).collect();
                 format!("run({})", args.join(", ")) 
             }).collect();
             format!("capture({})", seg_strs.join(" | "))
        }
        ExprKind::Arg(expr) => format!("arg({})", format_expr(expr)),
        ExprKind::Env(e) => format!("env({})", format_expr(e)),
        ExprKind::Exists(e) => format!("exists({})", format_expr(e)),
        ExprKind::IsDir(e) => format!("is_dir({})", format_expr(e)),
        ExprKind::IsFile(e) => format!("is_file({})", format_expr(e)),
        ExprKind::Len(e) => format!("len({})", format_expr(e)),
        ExprKind::BoolStr(e) => format!("bool_str({})", format_expr(e)),
        ExprKind::Count(e) => format!("count({})", format_expr(e)),
        ExprKind::Input(e) => format!("input({})", format_expr(e)),
        ExprKind::Index { list, index } => {
            // Suffix precedence 8
            wrap_parens(min_prec, 8, format!("{}[{}]", format_expr_prec(&list.node, 8), format_expr(index)))
        }
        ExprKind::Field { base, name } => {
             // Suffix precedence 8
            wrap_parens(min_prec, 8, format!("{}.{}", format_expr_prec(&base.node, 8), name))
        }
        ExprKind::Join { list, sep } => {
            format!("join({}, {})", format_expr(list), format_expr(sep))
        }
        ExprKind::EnvDot(name) => format!("env.{}", name),
        ExprKind::IsSymlink(e) => format!("is_symlink({})", format_expr(e)),
        ExprKind::IsExec(e) => format!("is_exec({})", format_expr(e)),
        ExprKind::IsReadable(e) => format!("is_readable({})", format_expr(e)),
        ExprKind::IsWritable(e) => format!("is_writable({})", format_expr(e)),
        ExprKind::IsNonEmpty(e) => format!("is_non_empty({})", format_expr(e)),

        ExprKind::Uid => "uid()".to_string(),
        ExprKind::Ppid => "ppid()".to_string(),
        ExprKind::Pwd => "pwd()".to_string(),
        ExprKind::SelfPid => "self_pid()".to_string(),
        ExprKind::Argv0 => "argv0()".to_string(),
        ExprKind::Argc => "argc()".to_string(),
        ExprKind::Status => "status()".to_string(),
        ExprKind::Args => "args".to_string(),
        ExprKind::Pid => "pid()".to_string(),
        ExprKind::Capture { expr, options } => {
            let inner_str = if let ExprKind::Command(args) = &expr.node {
                 let parts: Vec<String> = args.iter().map(format_expr).collect();
                 format!("run({})", parts.join(", "))
            } else if let ExprKind::CommandPipe(segs) = &expr.node {
                 let seg_strs: Vec<String> = segs.iter().map(|s| {
                     let args: Vec<String> = s.iter().map(format_expr).collect();
                     format!("run({})", args.join(", ")) 
                 }).collect();
                 seg_strs.join(" | ")
            } else {
                 format_expr(expr)
            };

            let mut parts = vec![inner_str];
            for opt in options {
                parts.push(format!("{} = {}", opt.name, format_expr(&opt.value)));
            }
            format!("capture({})", parts.join(", "))
        },
        ExprKind::Sh { cmd, options } => {
            let mut parts = vec![format_expr(cmd)];
            for opt in options {
                parts.push(format!("{}={}", opt.name, format_expr(&opt.value)));
            }
            format!("sh({})", parts.join(", "))
        }
        ExprKind::Confirm { prompt, default } => {
            match default {
                Some(d) => format!("confirm({}, default={})", format_expr(prompt), format_expr(d)),
                None => format!("confirm({})", format_expr(prompt)),
            }
        }
        _ => panic!("Formatting unimplemented for ExprKind: {:?}", kind),
    }
}

fn format_chain(left: &[Stmt], right: &[Stmt], op: &str, depth: usize) -> String {
    let is_simple = left.len() == 1 && right.len() == 1 
        && is_simple_atom(&left[0]) && is_simple_atom(&right[0]);

    if is_simple {
        return format!("{} {} {}", format_stmt(&left[0], depth), op, format_stmt(&right[0], depth));
    }

    let mut out = String::new();
    
    // Left side
    for (i, s) in left.iter().enumerate() {
        if i > 0 { out.push_str("\n"); out.push_str(&indent_str(depth)); }
        out.push_str(&format_stmt(s, depth));
    }
    
    out.push('\n');
    out.push_str(&indent_str(depth));
    out.push_str(op);
    out.push(' ');
    
    // Right side
    for (i, s) in right.iter().enumerate() {
        if i > 0 { out.push_str("\n"); out.push_str(&indent_str(depth)); }
        out.push_str(&format_stmt(s, depth));
    }
    
    out
}

fn wrap_parens(min_prec: u8, op_prec: u8, s: String) -> String {
    if op_prec < min_prec {
        format!("({})", s)
    } else {
        s
    }
}

fn sh_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            // $ is special in interpolated strings, but here we escape generically for "..."
            // If the string is intended to be interpolated, we might need to handle $.
            // But ExprKind::Literal usually implies raw data unless parsed from interpolated.
            // If it's a literal, we escape $ to match sh2 string logic if it allows $?
            // "foo$bar" -> is that var sub?
            // In sh2, "..." handles escapes.
            '$' => out.push_str("\\$"), 
            _ => out.push(c),
        }
    }
    out
}
