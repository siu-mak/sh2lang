use crate::ir::*;

pub fn emit(funcs: &[Function]) -> String {
    let mut out = String::new();

    for f in funcs {
        out.push_str(&format!("{}() {{\n", f.name));
        for cmd in &f.commands {
            out.push_str("  ");
            out.push_str(&cmd.join(" "));
            out.push('\n');
        }
        out.push_str("}\n");
    }
    out.push_str("\nmain \"$@\"\n");

    out
}
