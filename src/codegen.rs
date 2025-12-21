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
        Val::Compare { .. } => panic!("Cannot emit boolean value as string"),
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
        Cmd::If { cond, then_body, else_body } => {
            let cond_str = emit_cond(cond);
            out.push_str(&format!("{pad}if {cond_str}; then\n"));
            for c in then_body {
                emit_cmd(c, out, indent + 2);
            }

            if !else_body.is_empty() {
                out.push_str(&format!("{pad}else\n"));
                for c in else_body {
                    emit_cmd(c, out, indent + 2);
                }
            }

            out.push_str(&format!("{pad}fi\n"));
        }

    }
}
