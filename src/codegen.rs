use crate::ir::{Function, Cmd};

pub fn emit(funcs: &[Function]) -> String {
    let mut out = String::new();

    for f in funcs {
        out.push_str(&format!("{}() {{\n", f.name));

        for cmd in &f.commands {
            match cmd {
                Cmd::Exec(args) => {
                    out.push_str("  ");
                    out.push_str(&args.join(" "));
                    out.push('\n');
                }
                Cmd::Print(s) => {
                    out.push_str("  echo ");
                    out.push_str(s);
                    out.push('\n');
                }
            }
        }

        out.push_str("}\n");
    }

    out.push_str("\nmain \"$@\"\n");
    out
}
