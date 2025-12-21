use crate::ir::{Function, Cmd, Val};

pub fn emit(funcs: &[Function]) -> String {
    let mut out = String::new();

    for f in funcs {
        out.push_str(&format!("{}() {{\n", f.name));
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
        Val::Compare { .. } | Val::And(..) | Val::Or(..) | Val::Not(..) | Val::List(..) | Val::Args => panic!("Cannot emit boolean/list/args value as string"),
        Val::Command(args) => {
            let parts: Vec<String> = args.iter().map(emit_val).collect();
            format!("$( {} )", parts.join(" "))
        }
    }
}

fn emit_cond(v: &Val) -> String {
    match v {
        Val::Compare { left, op, right } => {
            let op_str = match op {
                crate::ir::CompareOp::Eq => "=",
                crate::ir::CompareOp::NotEq => "!=",
            };
            format!("[ {} {} {} ]", emit_val(left), op_str, emit_val(right))
        }
        Val::And(left, right) => {
            format!("{} && {}", emit_cond(left), emit_cond(right))
        }
        Val::Or(left, right) => {
            format!("{} || {}", emit_cond(left), emit_cond(right))
        }
        Val::Not(expr) => {
            format!("! {}", emit_cond(expr))
        }
        // Legacy "is set" behavior for direct values
        v => format!("[ -n {} ]", emit_val(v)),
    }
}

fn emit_cmd(cmd: &Cmd, out: &mut String, indent: usize) {
    let pad = " ".repeat(indent);

    match cmd {
        Cmd::Assign(name, val) => {
            out.push_str(&pad);
            out.push_str(name);
            out.push('=');
            out.push_str(&emit_val(val));
            out.push('\n');
        }
        Cmd::Exec(args) => {
            out.push_str(&pad);
            let shell_args: Vec<String> = args.iter().map(emit_val).collect();
            out.push_str(&shell_args.join(" "));
            out.push('\n');
        }
        Cmd::Print(val) => {
            out.push_str(&pad);
            out.push_str("echo ");
            out.push_str(&emit_val(val));
            out.push('\n');
        }
        Cmd::PrintErr(val) => {
            out.push_str(&pad);
            out.push_str("echo ");
            out.push_str(&emit_val(val));
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
                 let parts: Vec<String> = args.iter().map(emit_val).collect();
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
                    let shell_args: Vec<String> = args.iter().map(emit_val).collect();
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

    }
}
