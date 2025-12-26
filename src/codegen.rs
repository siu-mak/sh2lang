use crate::ir::{Function, Cmd, Val, RedirectTarget};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetShell {
    Bash,
    Posix,
}

pub fn emit(funcs: &[Function]) -> String {
    emit_with_target(funcs, TargetShell::Bash)
}

pub fn emit_with_target(funcs: &[Function], target: TargetShell) -> String {
    let mut out = String::new();
    
    // Existing codegen didn't emit shebang or options, but tests might expect bare functions.
    // Preserving identical output for Bash target.
    
    for (i, f) in funcs.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&format!("{}() {{\n", f.name));
        for (idx, param) in f.params.iter().enumerate() {
            match target {
                TargetShell::Bash => out.push_str(&format!("  local {}=\"${{{}}}\"\n", param, idx + 1)),
                TargetShell::Posix => out.push_str(&format!("  {}=\"${{{}}}\"\n", param, idx + 1)),
            }
        }
        for cmd in &f.commands {
            emit_cmd(cmd, &mut out, 2, target);
        }
        out.push_str("}\n");
    }

    out.push_str("\n__sh2_status=0\nmain \"$@\"\n");
    out
}


// Helper to escape single quotes within a string literal for safe shell quoting
// Replaces ' with '\'' and wraps in '...'
fn sh_single_quote(s: &str) -> String {
    let mut out = String::from("'");
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}


fn emit_val(v: &Val, target: TargetShell) -> String {
    match v {
        Val::Literal(s) => sh_single_quote(s),
        Val::Var(s) => format!("\"${}\"", s),
        Val::Concat(l, r) => format!("{}{}", emit_val(l, target), emit_val(r, target)),
        Val::Command(args) => {
            let parts: Vec<String> = args.iter().map(|a| emit_word(a, target)).collect();
            format!("\"$( {} )\"", parts.join(" "))
        }
        Val::CommandPipe(segments) => {
             let seg_strs: Vec<String> = segments.iter().map(|seg| {
                 let words: Vec<String> = seg.iter().map(|w| emit_word(w, target)).collect();
                 words.join(" ")
             }).collect();
             format!("\"$( {} )\"", seg_strs.join(" | "))
        }
        Val::Len(inner) => {
             format!("\"$( printf \"%s\" {} | awk '{{ print length($0) }}' )\"", emit_val(inner, target))
        }
        Val::Arg(n) => format!("\"${}\"", n),
        Val::Index { list, index } => {
            if target == TargetShell::Posix {
                panic!("List indexing is not supported in POSIX sh target");
            }
            match &**list {
                 Val::Var(name) => format!("\"${{{}[{}]}}\"", name, emit_index_expr(index, target)),
                 Val::List(elems) => {
                     let mut arr_str = String::new();
                     for (i, elem) in elems.iter().enumerate() {
                         if i > 0 { arr_str.push(' '); }
                         arr_str.push_str(&emit_word(elem, target));
                     }
                     // Force evaluation of index
                     format!("\"$( arr=({}); idx=$(( {} )); printf \"%s\" \"${{arr[idx]}}\" )\"", arr_str, emit_index_expr(index, target))
                 }
                 Val::Args => {
                     format!("\"$( arr=(\"$@\"); idx=$(( {} )); printf \"%s\" \"${{arr[idx]}}\" )\"", emit_index_expr(index, target))
                 }
                 _ => panic!("Index implemented only for variables and list literals"),
            }
        }
        Val::Join { list, sep } => {
             if target == TargetShell::Posix {
                 panic!("List join is not supported in POSIX sh target");
             }
             match &**list {
                 Val::Var(name) => {
                     // Arrays: "$( IFS=<sep>; printf "%s" "${name[*]}" )"
                     format!("\"$( IFS={}; printf \"%s\" \"${{{}[*]}}\" )\"", emit_val(sep, target), name)
                 }
                 Val::List(elems) => {
                     let mut arr_str = String::new();
                     for (i, elem) in elems.iter().enumerate() {
                         if i > 0 { arr_str.push(' '); }
                         arr_str.push_str(&emit_word(elem, target));
                     }
                     format!("\"$( arr=({}); IFS={}; printf \"%s\" \"${{arr[*]}}\" )\"", arr_str, emit_val(sep, target))
                 }
                 Val::Args => {
                     format!("\"$( IFS={}; printf \"%s\" \"$*\" )\"", emit_val(sep, target))
                 }
                 _ => panic!("Join implemented only for variables and list literals"),
             }
        }
        Val::Count(inner) => match &**inner {
            Val::List(elems) => match target {
                TargetShell::Bash => format!("\"{}\"", elems.len()),
                TargetShell::Posix => panic!("List literals not supported in POSIX target"),
            },
            Val::Var(name) => match target {
                TargetShell::Bash => format!("\"${{#{}[@]}}\"", name),
                TargetShell::Posix => panic!("Array count not supported in POSIX target"),
            },
            Val::Args => "\"$#\"".to_string(),
            _ => panic!("count(...) supports only list literals, list variables, and args"),
        },
        Val::Bool(_) => panic!("Cannot emit boolean value as string/word; booleans are only valid in conditions"),
        Val::Number(n) => format!("\"{}\"", n),
        Val::Status => "\"$__sh2_status\"".to_string(),
        Val::Pid => "\"$!\"".to_string(),
        Val::Env(inner) => match &**inner {
            Val::Literal(s) => format!("\"${{{}}}\"", s),
            Val::Var(name) => match target {
                TargetShell::Bash => format!("\"${{!{}}}\"", name),
                TargetShell::Posix => panic!("env(var_name) is not supported in POSIX sh target; use env(\"NAME\") or env.NAME"),
            },
            _ => panic!("env(...) requires a string literal name or variable name"),
        },
        Val::EnvDot(name) => match target {
            TargetShell::Bash => format!("\"$( ( typeset +x {0}; printenv {0} ) 2>/dev/null || printenv {0} 2>/dev/null || true )\"", name),
            TargetShell::Posix => format!("\"${{{}-}}\"", name),
        },
        Val::Uid => match target { 
            TargetShell::Bash => "\"$UID\"".to_string(), 
            TargetShell::Posix => panic!("uid() is not supported in POSIX sh target") 
        },
        Val::Ppid => match target { 
            TargetShell::Bash => "\"$PPID\"".to_string(), 
            TargetShell::Posix => panic!("ppid() is not supported in POSIX sh target") 
        },
        Val::Pwd => match target { 
            TargetShell::Bash => "\"$PWD\"".to_string(), 
            TargetShell::Posix => panic!("pwd() is not supported in POSIX sh target") 
        },
        Val::SelfPid => "\"$$\"".to_string(),
        Val::Argv0 => "\"$0\"".to_string(),
        Val::Argc => "\"$#\"".to_string(),
        Val::Arith { .. } => format!("\"$(( {} ))\"", emit_arith_expr(v, target)),
        Val::BoolStr(inner) => {
             format!("\"$( if {}; then printf \"%s\" \"true\"; else printf \"%s\" \"false\"; fi )\"", emit_cond(inner, target))
        },
        Val::Input(prompt) => match target {
             TargetShell::Bash => {
                 let p = emit_val(prompt, target);
                 // $( printf '%s' "Prompt" >&2; IFS= read -r __sh2_in; printf '%s' "$__sh2_in" )
                 format!("\"$( printf '%s' {} >&2; IFS= read -r __sh2_in; printf '%s' \"$__sh2_in\" )\"", p)
             }
             TargetShell::Posix => panic!("input(...) is not supported in POSIX sh target"),
        },
        Val::Args => panic!("args cannot be embedded/concatenated inside a word"),
        Val::Compare { .. } | Val::And(..) | Val::Or(..) | Val::Not(..) | Val::Exists(..) | Val::IsDir(..) | Val::IsFile(..) | Val::IsSymlink(..) | Val::IsExec(..) | Val::IsReadable(..) | Val::IsWritable(..) | Val::IsNonEmpty(..) | Val::List(..) | Val::Confirm(..) => panic!("Cannot emit boolean/list value as string"),
    }
}

fn emit_word(v: &Val, target: TargetShell) -> String {
    if let Val::Args = v {
        return "\"$@\"".to_string();
    }
    emit_val(v, target)
}

fn emit_cond(v: &Val, target: TargetShell) -> String {
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
                format!("[ {} {} {} ]", emit_val(left, target), op_str, emit_val(right, target))
            } else {
                format!("[ {} {} {} ]", emit_val(left, target), op_str, emit_val(right, target))
            }
        }
        Val::And(left, right) => {
            let mut l_str = emit_cond(left, target);
            let mut r_str = emit_cond(right, target);
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
            let l_str = emit_cond(left, target);
            let mut r_str = emit_cond(right, target);
            // If right is And, we must wrap it because && > || in sh2c but equal in bash.
            // A || B && C -> A || B && C (bash interprets as (A||B)&&C). We want A || (B&&C).
            if let Val::And(..) = **right {
                r_str = format!("( {} )", r_str);
            }
            format!("{} || {}", l_str, r_str)
        }
        Val::Not(expr) => {
            let inner = emit_cond(expr, target);
            // If inner is binary, wrap it. ! (A && B) -> ! A && B (bash interprets as (!A) && B).
            match **expr {
                Val::And(..) | Val::Or(..) => format!("! ( {} )", inner),
                _ => format!("! {}", inner),
            }
        }
        Val::Exists(path) => {
            format!("[ -e {} ]", emit_val(path, target))
        }
        Val::IsDir(path) => {
            format!("[ -d {} ]", emit_val(path, target))
        }
        Val::IsFile(path) => {
            format!("[ -f {} ]", emit_val(path, target))
        }
        Val::IsSymlink(path) => {
            format!("[ -L {} ]", emit_val(path, target))
        }
        Val::IsExec(path) => {
            format!("[ -x {} ]", emit_val(path, target))
        }
        Val::IsReadable(path) => {
            format!("[ -r {} ]", emit_val(path, target))
        }
        Val::IsWritable(path) => {
            format!("[ -w {} ]", emit_val(path, target))
        }
        Val::IsNonEmpty(path) => {
            format!("[ -s {} ]", emit_val(path, target))
        }
        Val::Confirm(prompt) => match target {
            TargetShell::Bash => {
                let p = emit_val(prompt, target);
                // Subshell that loops until valid input
                // Returns 0 for true, 1 for false
                format!(
                    "( while true; do printf '%s' {} >&2; if ! IFS= read -r __sh2_ans; then exit 1; fi; case \"${{__sh2_ans,,}}\" in y|yes|true|1) exit 0 ;; n|no|false|0|\"\") exit 1 ;; esac; done )",
                    p
                )
            }
            TargetShell::Posix => panic!("confirm(...) is not supported in POSIX sh target"),
        },
        Val::Bool(true) => "true".to_string(),
        Val::Bool(false) => "false".to_string(),
        Val::List(_) | Val::Args => panic!("args/list is not a valid condition; use count(...) > 0"),
        // "Truthiness" fallback for scalar values: check if non-empty string.
        v => format!("[ -n {} ]", emit_val(v, target)),
    }
}

fn emit_index_expr(v: &Val, target: TargetShell) -> String {
    emit_arith_expr(v, target)
}

// Helper for command substitution without outer double quotes (for use in arithmetic)
fn emit_cmdsub_raw(args: &[Val], target: TargetShell) -> String {
    let parts: Vec<String> = args.iter().map(|a| emit_word(a, target)).collect();
    format!("$( {} )", parts.join(" "))
}

fn emit_cmdsub_pipe_raw(segments: &[Vec<Val>], target: TargetShell) -> String {
    let seg_strs: Vec<String> = segments.iter().map(|seg| {
        let words: Vec<String> = seg.iter().map(|w| emit_word(w, target)).collect();
        words.join(" ")
    }).collect();
    format!("$( {} )", seg_strs.join(" | "))
}

fn emit_arith_expr(v: &Val, target: TargetShell) -> String {
    match v {
        Val::Literal(s) => s.clone(),
        Val::Number(n) => n.to_string(),
        Val::Var(s) => s.clone(), // Bare variable for arithmetic context
        Val::Arg(n) => format!("${}", n), // $1 etc
        Val::Status => "$__sh2_status".to_string(),
        Val::Pid => "$!".to_string(),
        Val::Uid => match target {
            TargetShell::Bash => "$UID".to_string(),
            TargetShell::Posix => panic!("uid() is not supported in POSIX sh target"),
        },
        Val::Ppid => match target {
            TargetShell::Bash => "$PPID".to_string(),
            TargetShell::Posix => panic!("ppid() is not supported in POSIX sh target"),
        },
        Val::SelfPid => "$$".to_string(),
        Val::Argc => "$#".to_string(),
        Val::Arith { left, op, right } => {
            let op_str = match op {
                crate::ir::ArithOp::Add => "+",
                crate::ir::ArithOp::Sub => "-",
                crate::ir::ArithOp::Mul => "*",
                crate::ir::ArithOp::Div => "/",
                crate::ir::ArithOp::Mod => "%",
            };
            format!("( {} {} {} )", emit_arith_expr(left, target), op_str, emit_arith_expr(right, target))
        }
        Val::Command(args) => emit_cmdsub_raw(args, target),
        Val::CommandPipe(segments) => emit_cmdsub_pipe_raw(segments, target),
        Val::Len(inner) => {
            // Raw command substitution: emits $( ... )
            format!("$( printf \"%s\" {} | awk '{{ print length($0) }}' )", emit_val(inner, target))
        }
        Val::Count(inner) => match &**inner {
            Val::List(elems) => match target {
                TargetShell::Bash => elems.len().to_string(),
                TargetShell::Posix => panic!("List literals not supported in POSIX target"),
            },
            Val::Var(name) => match target {
                TargetShell::Bash => format!("${{#{}[@]}}", name),
                TargetShell::Posix => panic!("Array count not supported in POSIX target"),
            },
            Val::Args => "$#".to_string(),
            _ => panic!("count(...) supports only list literals, list variables, and args"),
        },
        _ => panic!("Unsupported type in arithmetic expression"),
    }
}

fn emit_cmd(cmd: &Cmd, out: &mut String, indent: usize, target: TargetShell) {
    let pad = " ".repeat(indent);

    match cmd {
        Cmd::Assign(name, val) => {
            if target == TargetShell::Posix && matches!(val, Val::List(_) | Val::Args) {
                panic!("Array assignment is not supported in POSIX sh target");
            }
            out.push_str(&pad);
            if let Val::List(elems) = val {
                out.push_str(name);
                out.push_str("=(");
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 { out.push(' '); }
                    out.push_str(&emit_word(elem, target));
                }
                out.push_str(")\n");
            } else if let Val::Args = val {
                out.push_str(name);
                out.push_str("=(\"$@\")\n");
            } else {
                out.push_str(name);
                out.push('=');
                out.push_str(&emit_val(val, target));
                out.push('\n');
            }
        }
        Cmd::Exec { args, allow_fail } => {
            out.push_str(&pad);
            let shell_cmd = args.iter().map(|a| emit_word(a, target)).collect::<Vec<_>>().join(" ");
            
            if *allow_fail {
                // allow_fail: suppresses script failure (returns 0), captures real status in __sh2_status
                out.push_str(&format!(
                    "case $- in *e*) __e=1;; *) __e=0;; esac; set +e; {} ; __sh2_status=$?; if [ \"$__e\" = 1 ]; then set -e; fi; :\n",
                    shell_cmd
                ));
            } else {
                // Normal: capture status in __sh2_status, then restore $? so try/set-e works
                out.push_str(&shell_cmd);
                out.push_str("; __sh2_status=$?; (exit $__sh2_status)\n");
            }
        }
        Cmd::ExecReplace(args) => {
            out.push_str(&pad);
            out.push_str("exec ");
            let shell_args: Vec<String> = args.iter().map(|a| emit_word(a, target)).collect();
            out.push_str(&shell_args.join(" "));
            out.push('\n');
        }
        Cmd::Print(val) => {
            out.push_str(&pad);
            out.push_str("printf '%s\\n' ");
            match val {
                Val::Args => out.push_str("\"$*\""),
                _ => out.push_str(&emit_val(val, target)),
            }
            out.push('\n');
        }
        Cmd::PrintErr(val) => {
            out.push_str(&pad);
            out.push_str("printf '%s\\n' ");
            match val {
                Val::Args => out.push_str("\"$*\""),
                _ => out.push_str(&emit_val(val, target)),
            }
            out.push_str(" >&2\n");
        }
        Cmd::If { cond, then_body, elifs, else_body } => {
            let cond_str = emit_cond(cond, target);
            out.push_str(&format!("{pad}if {cond_str}; then\n"));
            for c in then_body {
                emit_cmd(c, out, indent + 2, target);
            }

            for (cond, body) in elifs {
                let cond_str = emit_cond(cond, target);
                out.push_str(&format!("{pad}elif {cond_str}; then\n"));
                for c in body {
                    emit_cmd(c, out, indent + 2, target);
                }
            }

            if !else_body.is_empty() {
                out.push_str(&format!("{pad}else\n"));
                for c in else_body {
                    emit_cmd(c, out, indent + 2, target);
                }
            }

            out.push_str(&format!("{pad}fi\n"));
        }
        Cmd::Pipe(segments) => {
             // Bash: normal pipeline with PIPESTATUS
             // POSIX: manual pipeline with FIFOs via helper
             
             let last_idx = segments.len() - 1;
             let allow_fail_last = segments[last_idx].1;

             match target {
                 TargetShell::Bash => {
                     out.push_str(&pad);
                     
                     let mut pipe_str = String::new();
                     for (i, (args, allow_fail)) in segments.iter().enumerate() {
                         if i > 0 { pipe_str.push_str(" | "); }
                         
                         let cmd_str = args.iter().map(|a| emit_word(a, target)).collect::<Vec<_>>().join(" ");
                         
                         if *allow_fail && i < last_idx {
                             // Non-final segment: suppress failure so it doesn't abort early
                             pipe_str.push_str(&format!("{{ {{ {}; }} || true; }}", cmd_str));
                         } else {
                             pipe_str.push_str(&cmd_str);
                         }
                     }

                     out.push_str(&format!(
                         "if [[ -o pipefail ]]; then __p=1; else __p=0; fi; set -o pipefail; case $- in *e*) __e=1;; *) __e=0;; esac; set +e; {} ; __sh2_status=$?; if [ \"$__e\" = 1 ]; then set -e; fi; if [ \"$__p\" = 0 ]; then set +o pipefail; fi; ",
                         pipe_str
                     ));

                     // Return
                     // If last stage is allowed to fail, the whole statement succeeds.
                     if allow_fail_last {
                         out.push_str(":\n"); 
                     } else {
                         out.push_str("(exit $__sh2_status)\n");
                     }
                 }
                 TargetShell::Posix => {
                     let stages: Vec<String> = segments.iter().map(|(args, _)| {
                          args.iter().map(|a| emit_word(a, target)).collect::<Vec<_>>().join(" ")
                     }).collect();
                     let allow_fails: Vec<bool> = segments.iter().map(|(_, af)| *af).collect();
                     
                     emit_posix_pipeline(out, &pad, target, &stages, &allow_fails, allow_fail_last);
                 }
             }
        }
        Cmd::PipeBlocks(segments) => {
            match target {
                TargetShell::Bash => {
                    out.push_str(&pad);
                    let mut pipe_str = String::new();
                    for (i, seg) in segments.iter().enumerate() {
                        if i > 0 { pipe_str.push_str(" | "); }
                        pipe_str.push_str("{\n");
                        for cmd in seg {
                            emit_cmd(cmd, &mut pipe_str, indent + 2, target);
                        }
                        pipe_str.push_str(&format!("{pad}}}"));
                    }
                    
                    out.push_str(&format!(
                        "if [[ -o pipefail ]]; then __p=1; else __p=0; fi; set -o pipefail; case $- in *e*) __e=1;; *) __e=0;; esac; set +e; {} ; __sh2_status=$?; if [ \"$__e\" = 1 ]; then set -e; fi; if [ \"$__p\" = 0 ]; then set +o pipefail; fi; ",
                        pipe_str
                    ));
                    out.push_str("(exit $__sh2_status)\n");
                }
                TargetShell::Posix => {
                     let stages: Vec<String> = segments.iter().map(|seg| {
                         let mut s = String::new();
                         s.push_str("{\n");
                         for cmd in seg { emit_cmd(cmd, &mut s, indent + 2, target); }
                         s.push_str(&format!("{pad}}}"));
                         s
                     }).collect();
                     let allow_fails = vec![false; segments.len()];
                     emit_posix_pipeline(out, &pad, target, &stages, &allow_fails, false);
                }
            }
        }
        Cmd::Case { expr, arms } => {
            out.push_str(&format!("{}case {} in\n", pad, emit_val(expr, target)));
            for (patterns, body) in arms {
                out.push_str(&pad);
                out.push_str("  ");
                let pat_strs: Vec<String> = patterns.iter().map(|p| match p {
                    crate::ir::Pattern::Literal(s) => sh_single_quote(s),
                    crate::ir::Pattern::Glob(s) => emit_case_glob_pattern(s),
                    crate::ir::Pattern::Wildcard => "*".to_string(),
                }).collect();
                out.push_str(&pat_strs.join("|"));
                out.push_str(")\n");
                
                for cmd in body {
                    emit_cmd(cmd, out, indent + 4, target);
                }
                out.push_str(&format!("{}  ;;\n", pad));
            }
            out.push_str(&format!("{}esac\n", pad));
        }

        Cmd::While { cond, body } => {
            let cond_str = emit_cond(cond, target);
            out.push_str(&format!("{pad}while {cond_str}; do\n"));
            for c in body {
                emit_cmd(c, out, indent + 2, target);
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
                            out.push_str(&emit_val(elem, target));
                        }
                    }
                    Val::Args => {
                         out.push_str(" \"$@\"");
                    }
                    Val::Var(name) => {
                        if target == TargetShell::Posix {
                            panic!("Iterating over array variable not supported in POSIX");
                        }
                        out.push_str(&format!(" \"${{{}[@]}}\"", name));
                    }
                    _ => {
                        out.push(' ');
                        out.push_str(&emit_val(item, target));
                    }
                }
             }
             out.push_str("; do\n");
             for c in body {
                 emit_cmd(c, out, indent + 2, target);
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
                 if is_boolean_expr(v) {
                     let cond_str = emit_cond(v, target);
                     out.push_str(&format!("{pad}if {}; then return 0; else return 1; fi\n", cond_str));
                 } else {
                     out.push_str(&format!("{pad}return {}\n", emit_val(v, target)));
                 }
             } else {
                 out.push_str(&format!("{pad}return\n"));
             }
        }
        Cmd::Exit(val) => {
             if let Some(v) = val {
                 if is_boolean_expr(v) {
                     let cond_str = emit_cond(v, target);
                     out.push_str(&format!("{pad}if {}; then exit 0; else exit 1; fi\n", cond_str));
                 } else {
                     out.push_str(&format!("{pad}exit {}\n", emit_val(v, target)));
                 }
             } else {
                 out.push_str(&format!("{pad}exit\n"));
             }
        }
        Cmd::WithEnv { bindings, body } => {
            // Check for single Exec optimization
            if body.len() == 1 {
                if let Cmd::Exec { args, .. } = &body[0] {
                    out.push_str(&pad);
                    for (k, v) in bindings {
                        out.push_str(&format!("{}={} ", k, emit_val(v, target)));
                    }
                    let shell_args: Vec<String> = args.iter().map(|a| emit_word(a, target)).collect();
                    out.push_str(&shell_args.join(" "));
                    out.push('\n');
                    return;
                }
            }

            // General case: Subshell
            out.push_str(&format!("{pad}(\n"));
            for (k, v) in bindings {
                out.push_str(&format!("{}  export {}={}\n", pad, k, emit_val(v, target)));
            }
            for cmd in body {
                emit_cmd(cmd, out, indent + 2, target);
            }
            out.push_str(&format!("{pad})\n"));
        }
        Cmd::WithLog { path, append, body } => {
            if target == TargetShell::Posix {
                panic!("with log(...) is not supported in POSIX sh target");
            }
            
            // Bash implementation using process substitution
            let path_val = emit_val(path, target);
            
            out.push_str(&format!("{pad}(\n"));
            out.push_str(&format!("{pad}  __sh2_log_path={}\n", path_val));
            
            if !append {
                // Truncate file once
                out.push_str(&format!("{pad}  : > \"$__sh2_log_path\"\n"));
            }

            // Always use append for tee to avoid race conditions between stdout/stderr tees 
            // overwriting each other (they have separate fds/offsets otherwise).
            // Users want interleaved output.
            
            // Ensure we wait for tee to finish even if the block exits early
            out.push_str(&format!("{pad}  trap 'exec >&-; exec 2>&-; wait' EXIT\n"));

            out.push_str(&format!("{pad}  exec > >(tee -a \"$__sh2_log_path\")\n"));
            out.push_str(&format!("{pad}  exec 2> >(tee -a \"$__sh2_log_path\" >&2)\n"));
            
            for cmd in body {
                emit_cmd(cmd, out, indent + 2, target);
            }
            out.push_str(&format!("{pad})\n"));
        }
        Cmd::WithCwd { path, body } => {
            out.push_str(&format!("{pad}(\n"));
            out.push_str(&format!("{pad}  cd {}\n", emit_val(path, target)));
            for cmd in body {
                emit_cmd(cmd, out, indent + 2, target);
            }
            out.push_str(&format!("{pad})\n"));
        }
        Cmd::Cd(path) => {
            out.push_str(&pad);
            out.push_str("cd ");
            out.push_str(&emit_val(path, target));
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
                out.push_str(&emit_word(arg, target));
            }
            out.push('\n');
        }
        Cmd::Subshell { body } => {
            out.push_str(&format!("{pad}(\n"));
            for cmd in body {
                emit_cmd(cmd, out, indent + 2, target);
            }
            out.push_str(&format!("{pad})\n"));
        }
        Cmd::Group { body } => {
            out.push_str(&format!("{pad}{{\n"));
            for cmd in body {
                emit_cmd(cmd, out, indent + 2, target);
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        Cmd::WithRedirect { stdout, stderr, stdin, body } => {
            out.push_str(&format!("{pad}{{\n"));
            for cmd in body {
                 emit_cmd(cmd, out, indent + 2, target);
            }
            out.push_str(&format!("{pad}}}")); // No newline yet, redirections follow
            
            // Handle regular stdin file redirection (before others, or order doesn't matter much for inputs vs outputs)
            // But preserving existing behavior: currently it emits stdin first.
            // Plan said: "Keep existing stdin handling position, but since heredoc is not a < redirection, emit heredoc operator after stdout/stderr redirections"
            
            let mut heredoc_content = None;

            if let Some(target_redir) = stdin {
                match target_redir {
                    RedirectTarget::File { path, .. } => {
                        out.push_str(&format!(" < {}", emit_val(path, target)));
                    }
                    RedirectTarget::HereDoc { content } => {
                        heredoc_content = Some(content);
                    }
                    _ => panic!("stdin redirected to something invalid"),
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
                if let Some(target_redir) = &stdout {
                    match target_redir {
                        RedirectTarget::File { path, append } => {
                            let op = if *append { ">>" } else { ">" };
                            out.push_str(&format!(" {} {}", op, emit_val(path, target)));
                        }
                        RedirectTarget::Stderr => {
                            out.push_str(" 1>&2");
                        }
                        RedirectTarget::Stdout => {
                            // no-op
                        }
                        RedirectTarget::HereDoc { .. } => panic!("Heredoc not valid for stdout"),
                    }
                }
            };

            let emit_stderr = |out: &mut String| {
                if let Some(target_redir) = &stderr {
                    match target_redir {
                        RedirectTarget::File { path, append } => {
                             let op = if *append { ">>" } else { ">" };
                             out.push_str(&format!(" 2{} {}", op, emit_val(path, target)));
                        }
                        RedirectTarget::Stdout => {
                             out.push_str(" 2>&1");
                        }
                        RedirectTarget::Stderr => {
                            // no-op
                        }
                        RedirectTarget::HereDoc { .. } => panic!("Heredoc not valid for stderr"),
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

            // Emit heredoc operator and content if present
            if let Some(content) = heredoc_content {
                // Find safe delimiter
                let mut delim = "__SH2_EOF__".to_string();
                let mut counter = 1;
                while content.contains(&delim) {
                    delim = format!("__SH2_EOF__{}__", counter);
                    counter += 1;
                }

                out.push_str(&format!(" <<'{}'\n", delim));
                out.push_str(content);
                if !content.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str(&delim);
            }
            
            out.push('\n');
        }
        Cmd::Spawn(cmd) => {
             // Emit inner command to a temp buffer to handle trailing newline
             let mut inner_out = String::new();
             emit_cmd(cmd, &mut inner_out, indent, target);
             
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
                                 out.push_str(&emit_word(elem, target));
                             }
                             out.push('\n');
                          }
                         _ => {
                             out.push_str(&format!("{pad}wait {}\n", emit_word(val, target)));
                         }
                     }
                 }
                 None => out.push_str(&format!("{pad}wait\n")),
             }
        }
        Cmd::TryCatch { try_body, catch_body } => {
            // Use group { ...; } instead of subshell ( ... ) so variables (like __sh2_status) propagate
            // Use && chaining for try_body to ensure we abort on failure (mimicking set -e which doesn't work in if)
            out.push_str(&format!("{pad}if ! {{\n"));
            if try_body.is_empty() {
                out.push_str(&format!("{pad}  :\n"));
            } else {
                for (i, cmd) in try_body.iter().enumerate() {
                    let mut cmd_buf = String::new();
                    emit_cmd(cmd, &mut cmd_buf, indent + 2, target);
                    let cmd_str = cmd_buf.trim_end();
                    
                    if i > 0 {
                         out.push_str(" &&\n");
                    }
                    out.push_str(cmd_str);
                }
                out.push('\n');
            }
            out.push_str(&format!("{pad}}}; then\n"));
            
            if catch_body.is_empty() {
                out.push_str(&format!("{pad}  :\n"));
            }
            for cmd in catch_body {
                emit_cmd(cmd, out, indent + 2, target);
            }
            out.push_str(&format!("{pad}fi\n"));
        }
        Cmd::AndThen { left, right } => {
            out.push_str(&format!("{pad}{{\n"));
            for cmd in left {
                emit_cmd(cmd, out, indent + 2, target);
            }
            out.push_str(&format!("{pad}}} && {{\n"));
            for cmd in right {
                emit_cmd(cmd, out, indent + 2, target);
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        Cmd::OrElse { left, right } => {
            out.push_str(&format!("{pad}{{\n"));
            for cmd in left {
                emit_cmd(cmd, out, indent + 2, target);
            }
            out.push_str(&format!("{pad}}} || {{\n"));
            for cmd in right {
                emit_cmd(cmd, out, indent + 2, target);
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        Cmd::Export { name, value } => {
            out.push_str(&pad);
            out.push_str("export ");
            out.push_str(name);
            if let Some(v) = value {
                out.push('=');
                out.push_str(&emit_val(v, target));
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
            out.push_str(&emit_word(path, target));
            out.push('\n');
        }
    }
}

fn is_boolean_expr(v: &Val) -> bool {
    matches!(v, Val::Compare { .. } | Val::And(..) | Val::Or(..) | Val::Not(..) | Val::Exists(..) | Val::IsDir(..) | Val::IsFile(..) | Val::IsSymlink(..) | Val::IsExec(..) | Val::IsReadable(..) | Val::IsWritable(..) | Val::IsNonEmpty(..) | Val::Bool(..))
}

fn emit_case_glob_pattern(glob: &str) -> String {
    let mut out = String::new();
    let mut literal_buf = String::new();

    for c in glob.chars() {
        if c == '*' || c == '?' {
            if !literal_buf.is_empty() {
                out.push_str(&sh_single_quote(&literal_buf));
                literal_buf.clear();
            }
            out.push(c);
        } else {
            literal_buf.push(c);
        }
    }
    if !literal_buf.is_empty() {
        out.push_str(&sh_single_quote(&literal_buf));
    }

    if out.is_empty() {
        return "''".to_string();
    }
    out
}

fn emit_posix_pipeline(
    out: &mut String,
    pad: &str,
    _target: TargetShell,
    stages: &[String],
    allow_fails: &[bool],
    allow_fail_last: bool,
) {
    // POSIX sh manual pipeline using FIFOs to simulate pipefail without deadlocks.
    // This implementation is errexit-safe: it saves/restores set -e state and ensures
    // all waits and cleanup run even when the surrounding script has set -e enabled.
    //
    // Algorithm:
    // 1. Save errexit state and disable it.
    // 2. Create FIFOs and set up traps.
    // 3. Open keepalive FDs (read+write) in parent to avoid open() deadlocks.
    // 4. Launch background processes (each closes keepalive FDs before running).
    // 5. Close keepalive FDs in parent.
    // 6. Wait and collect statuses (with set +e to avoid abort on non-zero).
    // 7. Compute effective status.
    // 8. Cleanup FIFOs and reset traps.
    // 9. Restore errexit state.
    // 10. Return status.
    
    out.push_str(pad);
    out.push_str("{\n");
    let indent_pad = format!("{}  ", pad);
    
    // Save and disable errexit
    out.push_str(&format!("{}case $- in *e*) __sh2_e=1;; *) __sh2_e=0;; esac; set +e;\n", indent_pad));
    
    let num_fifos = stages.len() - 1;
    out.push_str(&format!("{}__sh2_base=\"${{TMPDIR:-/tmp}}/sh2_fifo_$$\";\n", indent_pad));
    out.push_str(&format!("{}rm -f \"${{__sh2_base}}_\"*;\n", indent_pad));

    for i in 0..num_fifos {
        out.push_str(&format!("{}mkfifo \"${{__sh2_base}}_{}\";\n", indent_pad, i));
    }
    
    // Traps: EXIT only cleans up (no exit 1), INT/TERM/QUIT clean up and exit 1
    out.push_str(&format!("{}trap 'rm -f \"${{__sh2_base}}_\"*' EXIT;\n", indent_pad));
    out.push_str(&format!("{}trap 'rm -f \"${{__sh2_base}}_\"*; exit 1' INT TERM QUIT;\n", indent_pad));
    
    // Open keepalive FDs (fd 3+)
    out.push_str(&format!("{}__sh2_fd=3;\n", indent_pad));
    out.push_str(&format!("{}__sh2_fds=\"\";\n", indent_pad));
    for i in 0..num_fifos {
        out.push_str(&format!("{}eval \"exec ${{__sh2_fd}}<>\\\"${{__sh2_base}}_{}\\\"\";\n", indent_pad, i));
        out.push_str(&format!("{}__sh2_fds=\"$__sh2_fds $__sh2_fd\";\n", indent_pad));
        out.push_str(&format!("{}__sh2_fd=$((__sh2_fd + 1));\n", indent_pad));
    }

    // Launch stages
    for (i, cmd) in stages.iter().enumerate() {
        let mut redir = String::new();
        if i > 0 {
            redir.push_str(&format!(" < \"${{__sh2_base}}_{}\"", i - 1));
        }
        if i < stages.len() - 1 {
            redir.push_str(&format!(" > \"${{__sh2_base}}_{}\"", i));
        }
        
        // Child closes keepalive FDs before running command
        out.push_str(&format!("{}( for fd in $__sh2_fds; do eval \"exec $fd>&-\"; done; {} ) {} & __sh2_p{}=$!;\n", indent_pad, cmd, redir, i));
    }
    
    // Close keepalive FDs in parent
    out.push_str(&format!("{}for fd in $__sh2_fds; do eval \"exec $fd>&-\"; done;\n", indent_pad));
    
    // Wait and collect statuses (set +e already active, so non-zero won't abort)
    for i in 0..stages.len() {
        out.push_str(&format!("{}wait \"$__sh2_p{}\"; __sh2_s{}=$?;\n", indent_pad, i, i));
    }
    
    // Compute effective status (rightmost non-zero wins, ignoring allow_fail stages)
    out.push_str(&format!("{}__sh2_status=0;\n", indent_pad));
    for i in 0..stages.len() {
        if !allow_fails[i] {
            out.push_str(&format!("{}if [ \"$__sh2_s{}\" -ne 0 ]; then __sh2_status=\"$__sh2_s{}\"; fi;\n", indent_pad, i, i));
        }
    }
    
    // Cleanup: reset traps and remove FIFOs
    out.push_str(&format!("{}trap - EXIT INT TERM QUIT;\n", indent_pad));
    out.push_str(&format!("{}rm -f \"${{__sh2_base}}_\"*;\n", indent_pad));
    
    // Restore errexit if it was set
    out.push_str(&format!("{}if [ \"$__sh2_e\" = 1 ]; then set -e; fi;\n", indent_pad));
    
    // Return status
    if allow_fail_last {
        out.push_str(&format!("{}:\n", indent_pad));
    } else {
        out.push_str(&format!("{}(exit $__sh2_status)\n", indent_pad));
    }
    
    out.push_str(&format!("{}}}\n", pad));
}
