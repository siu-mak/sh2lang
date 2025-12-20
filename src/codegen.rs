use crate::ir::{Function, Cmd};

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

fn emit_cmd(cmd: &Cmd, out: &mut String, indent: usize) {
    let pad = " ".repeat(indent);

    match cmd {
        Cmd::Exec(args) => {
            out.push_str(&pad);
            out.push_str(&args.join(" "));
            out.push('\n');
        }
        Cmd::Print(s) => {
            out.push_str(&pad);
            out.push_str("echo ");
            out.push_str(s);
            out.push('\n');
        }
        Cmd::IfNonEmpty { var, then_body, else_body } => {
            out.push_str(&format!("{pad}if [ -n \"${var}\" ]; then\n"));
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
