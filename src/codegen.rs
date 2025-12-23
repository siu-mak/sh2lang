use crate::ir::{Function, Cmd, Val, RedirectTarget};

pub fn emit(funcs: &[Function]) -> String {
    let mut out = String::new();

    for (i, f) in funcs.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&format!("{}() {{\n", f.name));
        for (idx, param) in f.params.iter().enumerate() {
            out.push_str(&format!("  local {}=\"${{{}}}\"\n", param, idx + 1));
        }
        for cmd in &f.commands {
            emit_cmd(cmd, &mut out, 2);
        }
        out.push_str("}\n");
    }

    out.push_str("\nmain \"$@\"\n");
    out
}

fn emit_val(v: &Val) -> String {
    match v {
        Val::Literal(s) => format!("\"{}\"", s),
        Val::Var(s) => format!("\"${}\"", s),
        Val::Concat(l, r) => format!("{}{}", emit_val(l), emit_val(r)),
        Val::Command(args) => {
            let parts: Vec<String> = args.iter().map(emit_word).collect();
            format!("$( {} )", parts.join(" "))
        }
        Val::CommandPipe(segments) => {
             let seg_strs: Vec<String> = segments.iter().map(|seg| {
                 let words: Vec<String> = seg.iter().map(emit_word).collect();
                 words.join(" ")
             }).collect();
             format!("$( {} )", seg_strs.join(" | "))
        }
        Val::Len(inner) => {
             format!("$( printf \"%s\" {} | awk '{{ print length($0) }}' )", emit_val(inner))
        }
        Val::Arg(n) => format!("\"${}\"", n),
        Val::Index { list, index } => {
            match &**list {
                 Val::Var(name) => format!("\"${{{}[{}]}}\"", name, emit_index_expr(index)),
                 Val::List(elems) => {
                     let mut arr_str = String::new();
                     for (i, elem) in elems.iter().enumerate() {
                         if i > 0 { arr_str.push(' '); }
                         arr_str.push_str(&emit_word(elem));
                     }
                     // Force evaluation of index
                     format!("\"$( arr=({}); idx=$(( {} )); printf \"%s\" \"${{arr[idx]}}\" )\"", arr_str, emit_index_expr(index))
                 }
                 Val::Args => {
                     format!("\"$( arr=(\"$@\"); idx=$(( {} )); printf \"%s\" \"${{arr[idx]}}\" )\"", emit_index_expr(index))
                 }
                 _ => panic!("Index implemented only for variables and list literals"),
            }
        }
        Val::Join { list, sep } => {
             match &**list {
                 Val::Var(name) => {
                     // Arrays: "$( IFS=<sep>; printf "%s" "${name[*]}" )"
                     format!("\"$( IFS={}; printf \"%s\" \"${{{}[*]}}\" )\"", emit_val(sep), name)
                 }
                 Val::List(elems) => {
                     let mut arr_str = String::new();
                     for (i, elem) in elems.iter().enumerate() {
                         if i > 0 { arr_str.push(' '); }
                         arr_str.push_str(&emit_word(elem));
                     }
                     format!("\"$( arr=({}); IFS={}; printf \"%s\" \"${{arr[*]}}\" )\"", arr_str, emit_val(sep))
                 }
                 Val::Args => {
                     format!("\"$( IFS={}; printf \"%s\" \"$*\" )\"", emit_val(sep))
                 }
                 _ => panic!("Join implemented only for variables and list literals"),
             }
        }
        Val::Count(inner) => match &**inner {
            Val::List(elems) => format!("\"{}\"", elems.len()),
            Val::Var(name) => format!("\"${{#{}[@]}}\"", name),
            Val::Args => "\"$#\"".to_string(),
            _ => panic!("count(...) supports only list literals, list variables, and args"),
        },
        Val::Bool(_) => panic!("Cannot emit boolean value as string/word; booleans are only valid in conditions"),
        Val::Number(n) => format!("\"{}\"", n),
        Val::Status => "\"$?\"".to_string(),
        Val::Pid => "\"$!\"".to_string(),
        Val::Arith { .. } => format!("\"$(( {} ))\"", emit_arith_expr(v)),
        Val::Compare { .. } | Val::And(..) | Val::Or(..) | Val::Not(..) | Val::Exists(..) | Val::IsDir(..) | Val::IsFile(..) | Val::List(..) | Val::Args => panic!("Cannot emit boolean/list/args value as string"),
    }
}

fn emit_word(v: &Val) -> String {
    match v {
        Val::Literal(s) => format!("\"{}\"", s),
        Val::Var(s) => format!("\"${}\"", s),
        Val::Concat(l, r) => format!("{}{}", emit_word(l), emit_word(r)),
        Val::Command(args) => {
            let parts: Vec<String> = args.iter().map(emit_word).collect();
            format!("$( {} )", parts.join(" "))
        }
        Val::CommandPipe(segments) => {
             let seg_strs: Vec<String> = segments.iter().map(|seg| {
                 let words: Vec<String> = seg.iter().map(emit_word).collect();
                 words.join(" ")
             }).collect();
             format!("$( {} )", seg_strs.join(" | "))
        }
        Val::Len(inner) => {
             format!("$( printf \"%s\" {} | awk '{{ print length($0) }}' )", emit_val(inner))
        }
        Val::Arg(n) => format!("\"${}\"", n),
        Val::Index { list, index } => emit_val(&Val::Index { list: list.clone(), index: index.clone() }),
        Val::Join { list, sep } => emit_val(&Val::Join { list: list.clone(), sep: sep.clone() }),
        Val::Count(inner) => emit_val(&Val::Count(inner.clone())),
        Val::Bool(_) => panic!("Cannot emit boolean value as string/word; booleans are only valid in conditions"),
        Val::Number(n) => format!("\"{}\"", n),
        Val::Status => "\"$?\"".to_string(),
        Val::Pid => "\"$!\"".to_string(),
        Val::Arith { .. } => format!("\"$(( {} ))\"", emit_arith_expr(v)),
        Val::Args => "\"$@\"".into(),
        Val::Compare { .. } | Val::And(..) | Val::Or(..) | Val::Not(..) | Val::Exists(..) | Val::IsDir(..) | Val::IsFile(..) | Val::List(..) => panic!("Cannot emit boolean/list value as command word"),
    }
}

fn emit_cond(v: &Val) -> String {
    match v {
        Val::Compare { left, op, right } => {
            let (op_str, is_numeric) = match op {
                crate::ir::CompareOp::Eq => ("=", false),
                crate::ir::CompareOp::NotEq => ("!=", false),
                crate::ir::CompareOp::Lt => ("-lt", true),
                crate::ir::CompareOp::Le => ("-le", true),
                crate::ir::CompareOp::Gt => ("-gt", true),
                crate::ir::CompareOp::Ge => ("-ge", true),
            };
            if is_numeric {
                 // For numeric, operands can be arith expressions or just numbers.
                 // emit_val returns quoted strings, which [ ... ] handles nicely for -lt etc.
                 // e.g. [ "1" -lt "2" ]
                format!("[ {} {} {} ]", emit_val(left), op_str, emit_val(right))
            } else {
                format!("[ {} {} {} ]", emit_val(left), op_str, emit_val(right))
            }
        }
        Val::And(left, right) => {
            let mut l_str = emit_cond(left);
            let mut r_str = emit_cond(right);
            // Wrap left if Or (for clarity/spec, even if bash left-associativity makes it implicit)
            // (A || B) && C -> ( A || B ) && C
            if let Val::Or(..) = **left {
                l_str = format!("( {} )", l_str);
            }
            // If right is Or, we must wrap it because && > || in sh2c but equal in bash (left-associative).
            // A && (B || C) -> A && B || C (bash interprets as (A&&B)||C).
            if let Val::Or(..) = **right {
                r_str = format!("( {} )", r_str);
            }
            format!("{} && {}", l_str, r_str)
        }
        Val::Or(left, right) => {
            let l_str = emit_cond(left);
            let mut r_str = emit_cond(right);
            // If right is And, we must wrap it because && > || in sh2c but equal in bash.
            // A || B && C -> A || B && C (bash interprets as (A||B)&&C). We want A || (B&&C).
            if let Val::And(..) = **right {
                r_str = format!("( {} )", r_str);
            }
            format!("{} || {}", l_str, r_str)
        }
        Val::Not(expr) => {
            let inner = emit_cond(expr);
            // If inner is binary, wrap it. ! (A && B) -> ! A && B (bash interprets as (!A) && B).
            match **expr {
                Val::And(..) | Val::Or(..) => format!("! ( {} )", inner),
                _ => format!("! {}", inner),
            }
        }
        Val::Exists(path) => {
            format!("[ -e {} ]", emit_val(path))
        }
        Val::IsDir(path) => {
            format!("[ -d {} ]", emit_val(path))
        }
        Val::IsFile(path) => {
            format!("[ -f {} ]", emit_val(path))
        }
        Val::Bool(true) => "true".to_string(),
        Val::Bool(false) => "false".to_string(),
        // Legacy "is set" behavior for direct values
        v => format!("[ -n {} ]", emit_val(v)),
    }
}

fn emit_index_expr(v: &Val) -> String {
    emit_arith_expr(v)
}

fn emit_arith_expr(v: &Val) -> String {
    match v {
        Val::Literal(s) => s.clone(),
        Val::Number(n) => n.to_string(),
        Val::Var(s) => s.clone(), // Bare variable for arithmetic context
        Val::Arg(n) => format!("${}", n), // $1 etc
        Val::Status => "$?".to_string(),
        Val::Pid => "$!".to_string(),
        Val::Arith { left, op, right } => {
            let op_str = match op {
                crate::ir::ArithOp::Add => "+",
                crate::ir::ArithOp::Sub => "-",
                crate::ir::ArithOp::Mul => "*",
                crate::ir::ArithOp::Div => "/",
                crate::ir::ArithOp::Mod => "%",
            };
            format!("( {} {} {} )", emit_arith_expr(left), op_str, emit_arith_expr(right))
        }
        Val::Command(..) | Val::CommandPipe(..) | Val::Len(..) | Val::Count(..) => {
             // For complex types that emit $(...), we can just delegate to emit_val BUT strict quote removal logic is needed?
             // emit_val returns $(...) unquoted (based on previous check). 
             // Wait, emit_val returns `format!("$( {} )", ...)` for Command.
             // So it creates a String containing `$( ... )`. NO QUOTES.
             // So we can use emit_val result directly.
             emit_val(v)
        }
        _ => panic!("Unsupported type in arithmetic expression"),
    }
}

fn emit_cmd(cmd: &Cmd, out: &mut String, indent: usize) {
    let pad = " ".repeat(indent);

    match cmd {
        Cmd::Assign(name, val) => {
            out.push_str(&pad);
            if let Val::List(elems) = val {
                out.push_str(name);
                out.push_str("=(");
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 { out.push(' '); }
                    out.push_str(&emit_word(elem));
                }
                out.push_str(")\n");
            } else if let Val::Args = val {
                out.push_str(name);
                out.push_str("=(\"$@\")\n");
            } else {
                out.push_str(name);
                out.push('=');
                out.push_str(&emit_val(val));
                out.push('\n');
            }
        }
        Cmd::Exec(args) => {
            out.push_str(&pad);
            let shell_args: Vec<String> = args.iter().map(emit_word).collect();
            out.push_str(&shell_args.join(" "));
            out.push('\n');
        }
        Cmd::ExecReplace(args) => {
            out.push_str(&pad);
            out.push_str("exec ");
            let shell_args: Vec<String> = args.iter().map(emit_word).collect();
            out.push_str(&shell_args.join(" "));
            out.push('\n');
        }
        Cmd::Print(val) => {
            out.push_str(&pad);
            out.push_str("echo ");
            match val {
                Val::Args => out.push_str(&emit_word(val)),
                _ => out.push_str(&emit_val(val)),
            }
            out.push('\n');
        }
        Cmd::PrintErr(val) => {
            out.push_str(&pad);
            out.push_str("echo ");
            match val {
                Val::Args => out.push_str(&emit_word(val)),
                _ => out.push_str(&emit_val(val)),
            }
            out.push_str(" >&2\n");
        }
        Cmd::If { cond, then_body, elifs, else_body } => {
            let cond_str = emit_cond(cond);
            out.push_str(&format!("{pad}if {cond_str}; then\n"));
            for c in then_body {
                emit_cmd(c, out, indent + 2);
            }

            for (cond, body) in elifs {
                let cond_str = emit_cond(cond);
                out.push_str(&format!("{pad}elif {cond_str}; then\n"));
                for c in body {
                    emit_cmd(c, out, indent + 2);
                }
            }

            if !else_body.is_empty() {
                out.push_str(&format!("{pad}else\n"));
                for c in else_body {
                    emit_cmd(c, out, indent + 2);
                }
            }

            out.push_str(&format!("{pad}fi\n"));
        }
        Cmd::Pipe(segments) => {
             out.push_str(&pad);
             let cmds: Vec<String> = segments.iter().map(|args| {
                 let parts: Vec<String> = args.iter().map(emit_word).collect();
                 parts.join(" ")
             }).collect();
             out.push_str(&cmds.join(" | "));
             out.push('\n');
        }
        Cmd::Case { expr, arms } => {
            out.push_str(&format!("{}case {} in\n", pad, emit_val(expr)));
            for (patterns, body) in arms {
                out.push_str(&pad);
                out.push_str("  ");
                let pat_strs: Vec<String> = patterns.iter().map(|p| match p {
                    crate::ir::Pattern::Literal(s) => format!("\"{}\"", s),
                    crate::ir::Pattern::Wildcard => "*".to_string(),
                }).collect();
                out.push_str(&pat_strs.join("|"));
                out.push_str(")\n");
                
                for cmd in body {
                    emit_cmd(cmd, out, indent + 4);
                }
                out.push_str(&format!("{}  ;;\n", pad));
            }
            out.push_str(&format!("{}esac\n", pad));
        }

        Cmd::While { cond, body } => {
            let cond_str = emit_cond(cond);
            out.push_str(&format!("{pad}while {cond_str}; do\n"));
            for c in body {
                emit_cmd(c, out, indent + 2);
            }
            out.push_str(&format!("{pad}done\n"));
        }
        Cmd::For { var, items, body } => {
             out.push_str(&format!("{}for {} in", pad, var));
             for item in items {
                match item {
                    Val::List(elems) => {
                        for elem in elems {
                            out.push(' ');
                            out.push_str(&emit_val(elem));
                        }
                    }
                    Val::Args => {
                         out.push_str(" \"$@\"");
                    }
                    Val::Var(name) => {
                        out.push_str(&format!(" \"${{{}[@]}}\"", name));
                    }
                    _ => {
                        out.push(' ');
                        out.push_str(&emit_val(item));
                    }
                }
             }
             out.push_str("; do\n");
             for c in body {
                 emit_cmd(c, out, indent + 2);
             }
             out.push_str(&format!("{}done\n", pad));
        }
        Cmd::Break => {
            out.push_str(&format!("{pad}break\n"));
        }
        Cmd::Continue => {
            out.push_str(&format!("{pad}continue\n"));
        }
        Cmd::Return(val) => {
             if let Some(v) = val {
                 out.push_str(&format!("{pad}return {}\n", emit_val(v)));
             } else {
                 out.push_str(&format!("{pad}return\n"));
             }
        }
        Cmd::Exit(val) => {
             if let Some(v) = val {
                 out.push_str(&format!("{pad}exit {}\n", emit_val(v)));
             } else {
                 out.push_str(&format!("{pad}exit\n"));
             }
        }
        Cmd::WithEnv { bindings, body } => {
            // Check for single Exec optimization
            if body.len() == 1 {
                if let Cmd::Exec(args) = &body[0] {
                    out.push_str(&pad);
                    for (k, v) in bindings {
                        out.push_str(&format!("{}={} ", k, emit_val(v)));
                    }
                    let shell_args: Vec<String> = args.iter().map(emit_word).collect();
                    out.push_str(&shell_args.join(" "));
                    out.push('\n');
                    return;
                }
            }

            // General case: Subshell
            out.push_str(&format!("{pad}(\n"));
            for (k, v) in bindings {
                out.push_str(&format!("{}  {}={}\n", pad, k, emit_val(v)));
            }
            for cmd in body {
                emit_cmd(cmd, out, indent + 2);
            }
            out.push_str(&format!("{pad})\n"));
        }
        Cmd::WithCwd { path, body } => {
            out.push_str(&format!("{pad}(\n"));
            out.push_str(&format!("{pad}  cd {}\n", emit_val(path)));
            for cmd in body {
                emit_cmd(cmd, out, indent + 2);
            }
            out.push_str(&format!("{pad})\n"));
        }
        Cmd::Cd(path) => {
            out.push_str(&pad);
            out.push_str("cd ");
            out.push_str(&emit_val(path));
            out.push('\n');
        }
        Cmd::Raw(s) => {
            out.push_str(&pad);
            out.push_str(s);
            out.push('\n');
        }
        Cmd::Call { name, args } => {
            out.push_str(&pad);
            out.push_str(name);
            for arg in args {
                out.push(' ');
                out.push_str(&emit_word(arg));
            }
            out.push('\n');
        }
        Cmd::Subshell { body } => {
            out.push_str(&format!("{pad}(\n"));
            for cmd in body {
                emit_cmd(cmd, out, indent + 2);
            }
            out.push_str(&format!("{pad})\n"));
        }
        Cmd::Group { body } => {
            out.push_str(&format!("{pad}{{\n"));
            for cmd in body {
                emit_cmd(cmd, out, indent + 2);
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        Cmd::WithRedirect { stdout, stderr, stdin, body } => {
            out.push_str(&format!("{pad}{{\n"));
            for cmd in body {
                 emit_cmd(cmd, out, indent + 2);
            }
            out.push_str(&format!("{pad}}}")); // No newline yet, redirections follow
            
            if let Some(target) = stdin {
                match target {
                    RedirectTarget::File { path, .. } => {
                        out.push_str(&format!(" < {}", emit_val(path)));
                    }
                    _ => panic!("stdin redirected to something invalid (only file supported)"),
                }
            }

            // Determine emission order: standard (stdout then stderr) or swapped
            let mut emit_stdout_first = true;
            if let Some(stdout_target) = &stdout {
                if let Some(stderr_target) = &stderr {
                    if matches!(stdout_target, RedirectTarget::Stderr)
                        && matches!(stderr_target, RedirectTarget::File { .. })
                    {
                        emit_stdout_first = false;
                    }
                    if matches!(stdout_target, RedirectTarget::Stderr)
                        && matches!(stderr_target, RedirectTarget::Stdout)
                    {
                         panic!("Cyclic redirection: stdout to stderr AND stderr to stdout is not supported");
                    }
                }
            }

            let emit_stdout = |out: &mut String| {
                if let Some(target) = &stdout {
                    match target {
                        RedirectTarget::File { path, append } => {
                            let op = if *append { ">>" } else { ">" };
                            out.push_str(&format!(" {} {}", op, emit_val(path)));
                        }
                        RedirectTarget::Stderr => {
                            out.push_str(" 1>&2");
                        }
                        RedirectTarget::Stdout => {
                            // no-op
                        }
                    }
                }
            };

            let emit_stderr = |out: &mut String| {
                if let Some(target) = &stderr {
                    match target {
                        RedirectTarget::File { path, append } => {
                             let op = if *append { ">>" } else { ">" };
                             out.push_str(&format!(" 2{} {}", op, emit_val(path)));
                        }
                        RedirectTarget::Stdout => {
                             out.push_str(" 2>&1");
                        }
                        RedirectTarget::Stderr => {
                            // no-op
                        }
                    }
                }
            };

            if emit_stdout_first {
                emit_stdout(out);
                emit_stderr(out);
            } else {
                emit_stderr(out);
                emit_stdout(out);
            }
            out.push('\n');
        }
        Cmd::Spawn(cmd) => {
             // Emit inner command to a temp buffer to handle trailing newline
             let mut inner_out = String::new();
             emit_cmd(cmd, &mut inner_out, indent);
             
             // Trim trailing newline if present
             if inner_out.ends_with('\n') {
                 inner_out.pop();
             }
             
             out.push_str(&inner_out);
             out.push_str(" &\n");
        }
        Cmd::Wait(opt) => {
             match opt {
                 Some(val) => {
                     match val {
                         crate::ir::Val::List(elems) => {
                             out.push_str(&format!("{pad}wait"));
                             for elem in elems {
                                 out.push(' ');
                                 out.push_str(&emit_word(elem));
                             }
                             out.push('\n');
                         }
                         _ => {
                             out.push_str(&format!("{pad}wait {}\n", emit_word(val)));
                         }
                     }
                 }
                 None => out.push_str(&format!("{pad}wait\n")),
             }
        }
        Cmd::TryCatch { try_body, catch_body } => {
            out.push_str(&format!("{pad}if ! (\n"));
            for cmd in try_body {
                emit_cmd(cmd, out, indent + 2);
            }
            out.push_str(&format!("{pad}); then\n"));
            for cmd in catch_body {
                emit_cmd(cmd, out, indent + 2);
            }
            out.push_str(&format!("{pad}fi\n"));
        }
        Cmd::AndThen { left, right } => {
            out.push_str(&format!("{pad}{{\n"));
            for cmd in left {
                emit_cmd(cmd, out, indent + 2);
            }
            out.push_str(&format!("{pad}}} && {{\n"));
            for cmd in right {
                emit_cmd(cmd, out, indent + 2);
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        Cmd::OrElse { left, right } => {
            out.push_str(&format!("{pad}{{\n"));
            for cmd in left {
                emit_cmd(cmd, out, indent + 2);
            }
            out.push_str(&format!("{pad}}} || {{\n"));
            for cmd in right {
                emit_cmd(cmd, out, indent + 2);
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        Cmd::Export { name, value } => {
            out.push_str(&pad);
            out.push_str("export ");
            out.push_str(name);
            if let Some(v) = value {
                out.push('=');
                out.push_str(&emit_val(v));
            }
            out.push('\n');
        }
        Cmd::Unset(name) => {
            out.push_str(&pad);
            out.push_str("unset ");
            out.push_str(name);
            out.push('\n');
        }
        Cmd::Source(path) => {
            out.push_str(&pad);
            out.push_str(". ");
            out.push_str(&emit_word(path));
            out.push('\n');
        }
    }
}
