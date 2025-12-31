pub mod posix_lint;
pub use posix_lint::{PosixLint, PosixLintKind, lint_script, render_lints};

use crate::error::CompileError;
use crate::ir::{Cmd, Function, LogLevel, RedirectTarget, Val};
pub use crate::target::TargetShell;

#[derive(Clone, Debug, Copy)]
pub struct CodegenOptions {
    pub target: TargetShell,
    pub include_diagnostics: bool,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            target: TargetShell::Bash,
            include_diagnostics: true,
        }
    }
}

/// Returns the appropriate shebang line for the target shell
fn shebang(target: TargetShell) -> &'static str {
    match target {
        TargetShell::Bash => "#!/usr/bin/env bash",
        TargetShell::Posix => "#!/bin/sh",
    }
}

#[derive(Debug, Default, Clone)]
pub struct PreludeUsage {
    pub coalesce: bool,
    pub trim: bool,
    pub before: bool,
    pub after: bool,
    pub replace: bool,
    pub split: bool,
    pub matches: bool,
    pub parse_args: bool,
    pub args_flags: bool,
    pub args_positionals: bool,
    pub args_flag_get: bool,
    pub list_get: bool,
    pub load_envfile: bool,
    pub save_envfile: bool,
    pub json_kv: bool,
    pub which: bool,
    pub require: bool,
    pub tmpfile: bool,
    pub read_file: bool,
    pub write_file: bool,
    pub log: bool,
    pub home: bool,
    pub path_join: bool,
    pub loc: bool,
    pub uid: bool,
}

fn scan_usage(funcs: &[Function], include_diagnostics: bool) -> PreludeUsage {
    let mut usage = PreludeUsage::default();
    for f in funcs {
        for cmd in &f.commands {
            visit_cmd(cmd, &mut usage, include_diagnostics);
        }
    }
    usage
}

fn visit_cmd(cmd: &Cmd, usage: &mut PreludeUsage, include_diagnostics: bool) {
    match cmd {
        Cmd::Assign(_, val, loc) => {
            if include_diagnostics && loc.is_some() {
                usage.loc = true;
            }
            visit_val(val, usage);
        }
        Cmd::Exec { args, loc, .. } => {
            if include_diagnostics && loc.is_some() {
                usage.loc = true;
            }
            for a in args {
                visit_val(a, usage)
            }
        }
        Cmd::Print(val) | Cmd::PrintErr(val) => visit_val(val, usage),
        Cmd::If {
            cond,
            then_body,
            elifs,
            else_body,
        } => {
            visit_val(cond, usage);
            for c in then_body {
                visit_cmd(c, usage, include_diagnostics);
            }
            for (v, c) in elifs {
                visit_val(v, usage);
                for i in c {
                    visit_cmd(i, usage, include_diagnostics);
                }
            }
            for c in else_body {
                visit_cmd(c, usage, include_diagnostics);
            }
        }
        Cmd::Pipe(segs, loc) => {
            if include_diagnostics && loc.is_some() {
                usage.loc = true;
            }
            for (args, _) in segs {
                for a in args {
                    visit_val(a, usage)
                }
            }
        }
        Cmd::PipeBlocks(segs, loc) => {
            if include_diagnostics && loc.is_some() {
                usage.loc = true;
            }
            for s in segs {
                for c in s {
                    visit_cmd(c, usage, include_diagnostics)
                }
            }
        }
        Cmd::Case { expr, arms } => {
            visit_val(expr, usage);
            for (_, body) in arms {
                for c in body {
                    visit_cmd(c, usage, include_diagnostics);
                }
            }
        }
        Cmd::For { items, body, .. } => {
            for i in items {
                visit_val(i, usage);
            }
            for c in body {
                visit_cmd(c, usage, include_diagnostics);
            }
        }
        Cmd::ForMap { body, .. } => {
            for c in body {
                visit_cmd(c, usage, include_diagnostics);
            }
        }
        Cmd::While { cond, body } => {
            visit_val(cond, usage);
            for c in body {
                visit_cmd(c, usage, include_diagnostics);
            }
        }
        Cmd::Require(vals) => {
            usage.require = true;
            for v in vals {
                visit_val(v, usage);
            }
        }
        Cmd::Log { msg, .. } => {
            usage.log = true;
            visit_val(msg, usage);
        }
        Cmd::WriteFile { path, content, .. } => {
            usage.write_file = true;
            visit_val(path, usage);
            visit_val(content, usage);
        }
        Cmd::Cd(val) => visit_val(val, usage),
        Cmd::Call { args, .. } => {
            for a in args {
                visit_val(a, usage)
            }
        }
        Cmd::Subshell { body } | Cmd::Group { body } => {
            for c in body {
                visit_cmd(c, usage, include_diagnostics)
            }
        }
        Cmd::WithRedirect {
            stdout,
            stderr,
            stdin,
            body,
        } => {
            if let Some(t) = stdout {
                visit_redirect(t, usage);
            }
            if let Some(t) = stderr {
                visit_redirect(t, usage);
            }
            if let Some(t) = stdin {
                visit_redirect(t, usage);
            }
            for c in body {
                visit_cmd(c, usage, include_diagnostics);
            }
        }
        Cmd::Spawn(inner) => visit_cmd(inner, usage, include_diagnostics),
        Cmd::Wait(opt) => {
            if let Some(v) = opt {
                visit_val(v, usage)
            }
        }
        Cmd::TryCatch {
            try_body,
            catch_body,
        } => {
            for c in try_body {
                visit_cmd(c, usage, include_diagnostics);
            }
            for c in catch_body {
                visit_cmd(c, usage, include_diagnostics);
            }
        }
        Cmd::AndThen { left, right } | Cmd::OrElse { left, right } => {
            for c in left {
                visit_cmd(c, usage, include_diagnostics);
            }
            for c in right {
                visit_cmd(c, usage, include_diagnostics);
            }
        }
        Cmd::Export { value, .. } => {
            if let Some(v) = value {
                visit_val(v, usage)
            }
        }
        Cmd::Source(v) => visit_val(v, usage),
        Cmd::ExecReplace(args, loc) => {
            if include_diagnostics && loc.is_some() {
                usage.loc = true;
            }
            for a in args {
                visit_val(a, usage)
            }
        }
        Cmd::SaveEnvfile { path, env } => {
            usage.save_envfile = true;
            visit_val(path, usage);
            visit_val(env, usage);
        }
        Cmd::WithEnv { bindings, body } => {
            for (_, v) in bindings {
                visit_val(v, usage);
            }
            for c in body {
                visit_cmd(c, usage, include_diagnostics);
            }
        }
        Cmd::WithLog { path, body, .. } => {
            visit_val(path, usage);
            for c in body {
                visit_cmd(c, usage, include_diagnostics);
            }
        }
        Cmd::WithCwd { path, body } => {
            visit_val(path, usage);
            for c in body {
                visit_cmd(c, usage, include_diagnostics);
            }
        }
        Cmd::Break
        | Cmd::Continue
        | Cmd::Return(_)
        | Cmd::Exit(_)
        | Cmd::Unset(_)
        | Cmd::Raw(_) => {
            if let Cmd::Return(Some(v)) = cmd {
                visit_val(v, usage);
            }
            if let Cmd::Exit(Some(v)) = cmd {
                visit_val(v, usage);
            }
        }
    }
}

fn visit_redirect(target: &RedirectTarget, usage: &mut PreludeUsage) {
    match target {
        RedirectTarget::File { path, .. } => visit_val(path, usage),
        _ => {}
    }
}

fn visit_val(val: &Val, usage: &mut PreludeUsage) {
    match val {
        Val::Call { name, args } => {
            if name == "default" {
                usage.coalesce = true;
            }
            match name.as_str() {
                "trim" => usage.trim = true,
                "before" => usage.before = true,
                "after" => usage.after = true,
                "replace" => usage.replace = true,
                "split" => usage.split = true,
                "coalesce" => usage.coalesce = true,
                _ => {}
            }
            for a in args {
                visit_val(a, usage);
            }
        }
        Val::Which(v) => {
            usage.which = true;
            visit_val(v, usage);
        }
        Val::ReadFile(v) => {
            usage.read_file = true;
            visit_val(v, usage);
        }
        Val::Home => {
            usage.home = true;
        }
        Val::PathJoin(args) => {
            usage.path_join = true;
            for a in args {
                visit_val(a, usage);
            }
        }
        Val::Concat(l, r) | Val::And(l, r) | Val::Or(l, r) => {
            visit_val(l, usage);
            visit_val(r, usage);
        }
        Val::Arith { left, right, .. } | Val::Compare { left, right, .. } => {
            visit_val(left, usage);
            visit_val(right, usage);
        }
        Val::Not(v)
        | Val::Exists(v)
        | Val::IsDir(v)
        | Val::IsFile(v)
        | Val::IsSymlink(v)
        | Val::IsExec(v)
        | Val::IsReadable(v)
        | Val::IsWritable(v)
        | Val::IsNonEmpty(v)
        | Val::Len(v)
        | Val::Count(v)
        | Val::BoolStr(v)
        | Val::Input(v)
        | Val::Confirm(v)
        | Val::Env(v)
        | Val::ArgsFlags(v)
        | Val::ArgsPositionals(v)
        | Val::LoadEnvfile(v)
        | Val::JsonKv(v) => {
            visit_val(v, usage);
            if let Val::ArgsFlags(_) = val {
                usage.args_flags = true;
            }
            if let Val::ArgsPositionals(_) = val {
                usage.args_positionals = true;
            }
            if let Val::LoadEnvfile(_) = val {
                usage.load_envfile = true;
            }
            if let Val::JsonKv(_) = val {
                usage.json_kv = true;
            }
        }
        Val::Matches(t, r) => {
            usage.matches = true;
            visit_val(t, usage);
            visit_val(r, usage);
        }
        Val::ParseArgs => usage.parse_args = true,
        Val::Index { list, index } => {
            visit_val(list, usage);
            visit_val(index, usage);
            if let Val::ArgsFlags(_) = **list {
                usage.args_flag_get = true;
            }
            if let Val::ArgsPositionals(_) = **list {
                usage.list_get = true;
            }
        }
        Val::Join { list, sep } => {
            visit_val(list, usage);
            visit_val(sep, usage);
        }
        Val::TryRun(args) => {
            usage.tmpfile = true;
            usage.read_file = true;
            for a in args {
                visit_val(a, usage);
            }
        }
        Val::Uid => {
            usage.uid = true;
        }
        _ => {}
    }
}

pub fn emit(funcs: &[Function]) -> String {
    emit_with_target(funcs, TargetShell::Bash).expect("Failed to emit code")
}

pub fn emit_with_target(funcs: &[Function], target: TargetShell) -> Result<String, CompileError> {
    emit_with_options(
        funcs,
        CodegenOptions {
            target,
            include_diagnostics: true,
        },
    )
}

pub fn emit_with_options(funcs: &[Function], opts: CodegenOptions) -> Result<String, CompileError> {
    let usage = scan_usage(funcs, opts.include_diagnostics);
    let mut out = String::new();

    // Emit shebang as the very first line
    out.push_str(shebang(opts.target));
    out.push('\n');

    // Usage-aware prelude emission
    out.push_str(&emit_prelude(opts.target, &usage));

    for (i, f) in funcs.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&format!("{}() {{\n", f.name));
        if usage.loc && opts.target == TargetShell::Bash {
            out.push_str("  local __sh2_loc=\"\"\n");
        }
        for (idx, param) in f.params.iter().enumerate() {
            match opts.target {
                TargetShell::Bash => {
                    out.push_str(&format!("  local {}=\"${{{}}}\"\n", param, idx + 1))
                }
                TargetShell::Posix => out.push_str(&format!("  {}=\"${{{}}}\"\n", param, idx + 1)),
            }
        }
        for cmd in &f.commands {
            emit_cmd(cmd, &mut out, 2, opts.target, false)?;
        }
        out.push_str("}\n");
    }

    if usage.parse_args {
        out.push_str("\n__sh2_parsed_args=\"$(__sh2_parse_args \"$@\")\"\n");
    }
    out.push_str("__sh2_status=0\nmain \"$@\"\n");
    Ok(out)
}

/// Emit shell script with POSIX compatibility checking
/// Returns Ok(script) if successful, or Err(lint_message) if POSIX lints fail
pub fn emit_with_options_checked(funcs: &[Function], opts: CodegenOptions) -> Result<String, CompileError> {
    let out = emit_with_options(funcs, opts)?;
    
    // Run POSIX lints if targeting POSIX
    if opts.target == TargetShell::Posix {
        let lints = lint_script(&out);
        if !lints.is_empty() {
            return Err(CompileError {
                message: render_lints(&lints),
                target: Some(opts.target),
                location: None,
            });
        }
    }
    
    Ok(out)
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

fn emit_val(v: &Val, target: TargetShell) -> Result<String, CompileError> {
    match v {
        Val::Literal(s) => Ok(sh_single_quote(s)),
        Val::Var(s) => Ok(format!("\"${}\"", s)),
        Val::Concat(l, r) => Ok(format!("{}{}", emit_val(l, target)?, emit_val(r, target)?)),
        Val::TryRun(_) => Err(CompileError::unsupported("try_run() must be bound via let (e.g., let r = try_run(...))", target)),
        Val::Which(arg) => Ok(format!("\"$( __sh2_which {} )\"", emit_word(arg, target)?)),
        Val::ReadFile(arg) => Ok(format!("\"$( __sh2_read_file {} )\"", emit_word(arg, target)?)),
        Val::Home => Ok("\"$( __sh2_home )\"".to_string()),
        Val::PathJoin(args) => {
            let parts: Vec<String> = args.iter().map(|a| emit_word(a, target)).collect::<Result<_, _>>()?;
            Ok(format!("\"$( __sh2_path_join {} )\"", parts.join(" ")))
        }
        Val::Command(args) => {
            let parts: Vec<String> = args.iter().map(|a| emit_word(a, target)).collect::<Result<_, _>>()?;
            Ok(format!("\"$( {} )\"", parts.join(" ")))
        }
        Val::CommandPipe(segments) => {
            let seg_strs: Vec<String> = segments
                .iter()
                .map(|seg| {
                    let words: Vec<String> = seg.iter().map(|w| emit_word(w, target)).collect::<Result<_, _>>()?;
                    Ok(words.join(" "))
                })
                .collect::<Result<_, CompileError>>()?;
            Ok(format!("\"$( {} )\"", seg_strs.join(" | ")))
        }
        Val::Len(inner) => {
            Ok(format!(
                "\"$( printf \"%s\" {} | awk 'BEGIN{{l=0}} {{l=length($0)}} END{{print l}}' )\"",
                emit_val(inner, target)?
            ))
        }
        Val::Arg(n) => Ok(format!("\"${}\"", n)),
        Val::ParseArgs => Ok("\"${__sh2_parsed_args}\"".to_string()),
        Val::ArgsFlags(inner) => Ok(format!("\"$( __sh2_args_flags {} )\"", emit_val(inner, target)?)),
        Val::ArgsPositionals(inner) => Ok(format!(
            "\"$( __sh2_args_positionals {} )\"",
            emit_val(inner, target)?
        )),
        Val::Index { list, index } => {
            match &**list {
                Val::ArgsFlags(_) => {
                    Ok(format!(
                        "\"$( __sh2_args_flag_get {} {} )\"",
                        emit_val(list, target)?,
                        emit_val(index, target)?
                    ))
                }
                Val::ArgsPositionals(_) => {
                    Ok(format!(
                        "\"$( __sh2_list_get {} $(( {} )) )\"",
                        emit_val(list, target)?,
                        emit_index_expr(index, target)?
                    ))
                }
                _ => {
                    if target == TargetShell::Posix {
                        return Err(CompileError::unsupported("List indexing is not supported in POSIX sh target", target));
                    }
                    match &**list {
                        Val::Var(name) => {
                            Ok(format!("\"${{{}[{}]}}\"", name, emit_index_expr(index, target)?))
                        }
                        Val::List(elems) => {
                            let mut arr_str = String::new();
                            for (i, elem) in elems.iter().enumerate() {
                                if i > 0 {
                                    arr_str.push(' ');
                                }
                                arr_str.push_str(&emit_word(elem, target)?);
                            }
                            Ok(format!(
                                "\"$( arr=({}); idx=$(( {} )); printf \"%s\" \"${{arr[idx]}}\" )\"",
                                arr_str,
                                emit_index_expr(index, target)?
                            ))
                        }
                        Val::Args => {
                            Ok(format!(
                                "\"$( arr=(\"$@\"); idx=$(( {} )); printf \"%s\" \"${{arr[idx]}}\" )\"",
                                emit_index_expr(index, target)?
                            ))
                        }
                        _ => Err(CompileError::internal("Index implemented only for variables and list literals", target)),
                    }
                }
            }
        }
        Val::Join { list, sep } => {
            if target == TargetShell::Posix {
                return Err(CompileError::unsupported("List join is not supported in POSIX sh target", target));
            }
            match &**list {
                Val::Var(name) => {
                    Ok(format!(
                        "\"$( IFS={}; printf \"%s\" \"${{{}[*]}}\" )\"",
                        emit_val(sep, target)?,
                        name
                    ))
                }
                Val::List(elems) => {
                    let mut arr_str = String::new();
                    for (i, elem) in elems.iter().enumerate() {
                        if i > 0 {
                            arr_str.push(' ');
                        }
                        arr_str.push_str(&emit_word(elem, target)?);
                    }
                    Ok(format!(
                        "\"$( arr=({}); IFS={}; printf \"%s\" \"${{arr[*]}}\" )\"",
                        arr_str,
                        emit_val(sep, target)?
                    ))
                }
                Val::Args => {
                    Ok(format!(
                        "\"$( IFS={}; printf \"%s\" \"$*\" )\"",
                        emit_val(sep, target)?
                    ))
                }
                _ => Err(CompileError::internal("Join implemented only for variables and list literals", target)),
            }
        }
        Val::Count(inner) => match &**inner {
            Val::List(elems) => match target {
                TargetShell::Bash => Ok(format!("\"{}\"", elems.len())),
                TargetShell::Posix => Err(CompileError::unsupported("List literals not supported in POSIX target", target)),
            },
            Val::Var(name) => match target {
                TargetShell::Bash => Ok(format!("\"${{#{}[@]}}\"", name)),
                TargetShell::Posix => Err(CompileError::unsupported("Array count not supported in POSIX target", target)),
            },
            Val::Args => Ok("\"$#\"".to_string()),
            _ => Err(CompileError::internal("count(...) supports only list literals, list variables, and args", target)),
        },
        Val::Bool(_) => Err(CompileError::internal(
            "Cannot emit boolean value as string/word; booleans are only valid in conditions",
            target
        )),
        Val::Number(n) => Ok(format!("\"{}\"", n)),
        Val::Status => Ok("\"$__sh2_status\"".to_string()),
        Val::Pid => Ok("\"$!\"".to_string()),
        Val::Env(inner) => match &**inner {
            Val::Literal(s) => Ok(format!("\"${{{}}}\"", s)),
            Val::Var(name) => match target {
                TargetShell::Bash => Ok(format!("\"${{!{}}}\"", name)),
                TargetShell::Posix => Err(CompileError::unsupported(
                    "env(var_name) is not supported in POSIX sh target; use env(\"NAME\") or env.NAME",
                    target
                )),
            },
            _ => Err(CompileError::internal("env(...) requires a string literal name or variable name", target)),
        },
        Val::EnvDot(name) => match target {
            TargetShell::Bash => Ok(format!(
                "\"$( ( typeset +x {0}; printenv {0} ) 2>/dev/null || printenv {0} 2>/dev/null || true )\"",
                name
            )),
            TargetShell::Posix => Ok(format!("\"${{{}-}}\"", name)),
        },
        Val::Uid => Ok("\"$__sh2_uid\"".to_string()),
        Val::Ppid => match target {
            TargetShell::Bash => Ok("\"$PPID\"".to_string()),
            TargetShell::Posix => Err(CompileError::unsupported("ppid() is not supported in POSIX sh target", target)),
        },
        Val::Pwd => match target {
            TargetShell::Bash => Ok("\"$PWD\"".to_string()),
            TargetShell::Posix => Err(CompileError::unsupported("pwd() is not supported in POSIX sh target", target)),
        },
        Val::SelfPid => Ok("\"$$\"".to_string()),
        Val::Argv0 => Ok("\"$0\"".to_string()),
        Val::Argc => Ok("\"$#\"".to_string()),
        Val::Arith { .. } => Ok(format!("\"$(( {} ))\"", emit_arith_expr(v, target)?)),
        Val::BoolStr(inner) => {
            Ok(format!(
                "\"$( if {}; then printf \"%s\" \"true\"; else printf \"%s\" \"false\"; fi )\"",
                emit_cond(inner, target)?
            ))
        }
        Val::Input(prompt) => match target {
            TargetShell::Bash => {
                let p = emit_val(prompt, target)?;
                Ok(format!(
                    "\"$( printf '%s' {} >&2; IFS= read -r __sh2_in; printf '%s' \"$__sh2_in\" )\"",
                    p
                ))
            }
            TargetShell::Posix => Err(CompileError::unsupported("input(...) is not supported in POSIX sh target", target)),
        },
        Val::Args => Err(CompileError::internal("args cannot be embedded/concatenated inside a word", target)),
        Val::Call { name, args } => {
            let (func_name, needs_prefix) = if name == "default" {
                ("coalesce", true)
            } else if is_prelude_helper(name) {
                (name.as_str(), true)
            } else {
                (name.as_str(), false)
            };

            let arg_strs: Vec<String> = args.iter().map(|a| emit_word(a, target)).collect::<Result<_, _>>()?;
            if needs_prefix {
                Ok(format!("\"$( __sh2_{} {} )\"", func_name, arg_strs.join(" ")))
            } else {
                Ok(format!("\"$( {} {} )\"", func_name, arg_strs.join(" ")))
            }
        }
        Val::LoadEnvfile(path) => {
            Ok(format!("\"$( __sh2_load_envfile {} )\"", emit_word(path, target)?))
        }
        Val::JsonKv(blob) => {
            Ok(format!("\"$( __sh2_json_kv {} )\"", emit_word(blob, target)?))
        }
        Val::Matches(..) => {
            Ok(format!(
                "\"$( if {}; then printf \"%s\" \"true\"; else printf \"%s\" \"false\"; fi )\"",
                emit_cond(v, target)?
            ))
        }
        Val::MapIndex { map, key } => {
            if target == TargetShell::Posix {
                return Err(CompileError::unsupported("map/dict is only supported in Bash target", target));
            }
            let escaped_key = sh_single_quote(key);
            Ok(format!("\"${{{}[{}]}}\"", map, escaped_key))
        }
        Val::MapLiteral(_) => Err(CompileError::unsupported("Map literal is only allowed in 'let' assignment", target)),
        Val::Compare { .. }
        | Val::And(..)
        | Val::Or(..)
        | Val::Not(..)
        | Val::Exists(..)
        | Val::IsDir(..)
        | Val::IsFile(..)
        | Val::IsSymlink(..)
        | Val::IsExec(..)
        | Val::IsReadable(..)
        | Val::IsWritable(..)
        | Val::IsNonEmpty(..)
        | Val::List(..)
        | Val::Confirm(..) => Err(CompileError::internal("Cannot emit boolean/list value as string", target)),
    }
}

fn emit_word(v: &Val, target: TargetShell) -> Result<String, CompileError> {
    if let Val::Args = v {
        return Ok("\"$@\"".to_string());
    }
    emit_val(v, target)
}

fn emit_cond(v: &Val, target: TargetShell) -> Result<String, CompileError> {
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
                Ok(format!(
                    "[ {} {} {} ]",
                    emit_val(left, target)?,
                    op_str,
                    emit_val(right, target)?
                ))
            } else {
                Ok(format!(
                    "[ {} {} {} ]",
                    emit_val(left, target)?,
                    op_str,
                    emit_val(right, target)?
                ))
            }
        }
        Val::And(left, right) => {
            let mut l_str = emit_cond(left, target)?;
            let mut r_str = emit_cond(right, target)?;
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
            Ok(format!("{} && {}", l_str, r_str))
        }
        Val::Or(left, right) => {
            let l_str = emit_cond(left, target)?;
            let mut r_str = emit_cond(right, target)?;
            // If right is And, we must wrap it because && > || in sh2c but equal in bash.
            // A || B && C -> A || B && C (bash interprets as (A||B)&&C). We want A || (B&&C).
            if let Val::And(..) = **right {
                r_str = format!("( {} )", r_str);
            }
            Ok(format!("{} || {}", l_str, r_str))
        }
        Val::Not(expr) => {
            let inner = emit_cond(expr, target)?;
            // If inner is binary, wrap it. ! (A && B) -> ! A && B (bash interprets as (!A) && B).
            match **expr {
                Val::And(..) | Val::Or(..) => Ok(format!("! ( {} )", inner)),
                _ => Ok(format!("! {}", inner)),
            }
        }
        Val::Exists(path) => {
            Ok(format!("[ -e {} ]", emit_val(path, target)?))
        }
        Val::IsDir(path) => {
            Ok(format!("[ -d {} ]", emit_val(path, target)?))
        }
        Val::IsFile(path) => {
            Ok(format!("[ -f {} ]", emit_val(path, target)?))
        }
        Val::IsSymlink(path) => {
            Ok(format!("[ -L {} ]", emit_val(path, target)?))
        }
        Val::IsExec(path) => {
            Ok(format!("[ -x {} ]", emit_val(path, target)?))
        }
        Val::IsReadable(path) => {
            Ok(format!("[ -r {} ]", emit_val(path, target)?))
        }
        Val::IsWritable(path) => {
            Ok(format!("[ -w {} ]", emit_val(path, target)?))
        }
        Val::IsNonEmpty(path) => {
            Ok(format!("[ -s {} ]", emit_val(path, target)?))
        }
        Val::Confirm(prompt) => match target {
            TargetShell::Bash => {
                let p = emit_val(prompt, target)?;
                // Subshell that loops until valid input
                // Returns 0 for true, 1 for false
                Ok(format!(
                    "( while true; do printf '%s' {} >&2; if ! IFS= read -r __sh2_ans; then exit 1; fi; case \"${{__sh2_ans,,}}\" in y|yes|true|1) exit 0 ;; n|no|false|0|\"\") exit 1 ;; esac; done )",
                    p
                ))
            }
            TargetShell::Posix => Err(CompileError::unsupported("confirm(...) is not supported in POSIX sh target", target)),
        },
        Val::Bool(true) => Ok("true".to_string()),
        Val::Bool(false) => Ok("false".to_string()),
        Val::List(_) | Val::Args => {
            Err(CompileError::internal("args/list is not a valid condition; use count(...) > 0", target))
        }
        Val::Matches(text, regex) => {
            Ok(format!(
                "__sh2_matches {} {}",
                emit_val(text, target)?,
                emit_val(regex, target)?
            ))
        }
        // "Truthiness" fallback for scalar values: check if non-empty string.
        v => Ok(format!("[ -n {} ]", emit_val(v, target)?)),
    }
}

fn emit_index_expr(v: &Val, target: TargetShell) -> Result<String, CompileError> {
    emit_arith_expr(v, target)
}

fn emit_cmdsub_raw(args: &[Val], target: TargetShell) -> Result<String, CompileError> {
    let parts: Vec<String> = args.iter().map(|a| emit_word(a, target)).collect::<Result<_, _>>()?;
    Ok(format!("$( {} )", parts.join(" ")))
}

fn emit_cmdsub_pipe_raw(segments: &[Vec<Val>], target: TargetShell) -> Result<String, CompileError> {
    let seg_strs: Vec<String> = segments
        .iter()
        .map(|seg| {
            let words: Vec<String> = seg.iter().map(|w| emit_word(w, target)).collect::<Result<_, _>>()?;
            Ok(words.join(" "))
        })
        .collect::<Result<_, CompileError>>()?;
    Ok(format!("$( {} )", seg_strs.join(" | ")))
}

fn emit_arith_expr(v: &Val, target: TargetShell) -> Result<String, CompileError> {
    match v {
        Val::Literal(s) => Ok(s.clone()),
        Val::Number(n) => Ok(n.to_string()),
        Val::Var(s) => Ok(s.clone()),
        Val::Arg(n) => Ok(format!("${}", n)),
        Val::Status => Ok("$__sh2_status".to_string()),
        Val::Pid => Ok("$!".to_string()),
        Val::Uid => Ok("$__sh2_uid".to_string()),
        Val::Ppid => match target {
            TargetShell::Bash => Ok("$PPID".to_string()),
            TargetShell::Posix => Err(CompileError::unsupported("ppid() is not supported in POSIX sh target", target)),
        },
        Val::SelfPid => Ok("$$".to_string()),
        Val::Argc => Ok("$#".to_string()),
        Val::Arith { left, op, right } => {
            let op_str = match op {
                crate::ir::ArithOp::Add => "+",
                crate::ir::ArithOp::Sub => "-",
                crate::ir::ArithOp::Mul => "*",
                crate::ir::ArithOp::Div => "/",
                crate::ir::ArithOp::Mod => "%",
            };
            Ok(format!(
                "( {} {} {} )",
                emit_arith_expr(left, target)?,
                op_str,
                emit_arith_expr(right, target)?
            ))
        }
        Val::Command(args) => emit_cmdsub_raw(args, target),
        Val::CommandPipe(segments) => emit_cmdsub_pipe_raw(segments, target),
        Val::Len(inner) => {
            // Raw command substitution: emits $( ... )
            Ok(format!(
                "$( printf \"%s\" {} | awk '{{ print length($0) }}' )",
                emit_val(inner, target)?
            ))
        }
        Val::Count(inner) => match &**inner {
            Val::List(elems) => match target {
                TargetShell::Bash => Ok(elems.len().to_string()),
                TargetShell::Posix => Err(CompileError::unsupported("List literals not supported in POSIX target", target)),
            },
            Val::Var(name) => match target {
                TargetShell::Bash => Ok(format!("${{#{}[@]}}", name)),
                TargetShell::Posix => Err(CompileError::unsupported("Array count not supported in POSIX target", target)),
            },
            Val::Args => Ok("$#".to_string()),
            _ => Err(CompileError::internal("count(...) supports only list literals, list variables, and args", target)),
        },
        _ => Err(CompileError::internal("Unsupported type in arithmetic expression", target)),
    }
}

fn emit_cmd(
    cmd: &Cmd,
    out: &mut String,
    indent: usize,
    target: TargetShell,
    in_cond_ctx: bool,
) -> Result<(), CompileError> {
    let pad = " ".repeat(indent);

    match cmd {
        Cmd::Assign(name, val, loc) => {
            if let Some(l) = loc {
                out.push_str(&format!("{}__sh2_loc=\"{}\"\n", pad, l));
            }
            if target == TargetShell::Posix {
                if matches!(val, Val::MapLiteral(_)) {
                    return Err(CompileError::unsupported("map/dict is only supported in Bash target", target));
                }
                if matches!(val, Val::List(_) | Val::Args) {
                    return Err(CompileError::unsupported("Array assignment is not supported in POSIX sh target", target));
                }
            } else if let Val::MapLiteral(entries) = val {
                // Bash Map Assignment
                out.push_str(&pad);

                // 1. Emit associative array
                // local -A map=( ['k']="v" ... )
                out.push_str("local -A ");
                out.push_str(name);
                out.push_str("=(");

                // Track usage for keys array
                let mut keys_seen = std::collections::HashSet::new();
                let mut ordered_keys = Vec::new();

                for (key, value) in entries {
                    out.push(' ');
                    out.push_str(&format!("[{}]=", sh_single_quote(key)));
                    out.push_str(&emit_word(value, target)?);

                    if !keys_seen.contains(key) {
                        keys_seen.insert(key.clone());
                        ordered_keys.push(key);
                    }
                }
                out.push_str(" )\n");

                // 2. Emit keys array for deterministic iteration
                out.push_str(&pad);
                out.push_str(&format!("local -a __sh2_keys_{}=(", name));
                for key in ordered_keys {
                    out.push(' ');
                    out.push_str(&sh_single_quote(key));
                }
                out.push_str(" )\n");
                return Ok(());
            }

            // Normal assignment
            out.push_str(&pad);
            if let Val::List(elems) = val {
                out.push_str(name);
                out.push_str("=(");
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        out.push(' ');
                    }
                    out.push_str(&emit_word(elem, target)?);
                }
                out.push_str(")\n");
            } else if let Val::TryRun(args) = val {
                let cmd = args
                    .iter()
                    .map(|a| emit_word(a, target))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(" ");
                // Mktemp
                out.push_str(&format!(
                    "{pad}{name}__tmp_out=\"$(__sh2_tmpfile)\"\n",
                    pad = pad,
                    name = name
                ));
                out.push_str(&format!(
                    "{pad}{name}__tmp_err=\"$(__sh2_tmpfile)\"\n",
                    pad = pad,
                    name = name
                ));

                // Run (allow fail semantics)
                // Use 'if' to suppress ERR trap and capture status
                out.push_str(&format!(
                    "{pad}if {} >\"${{{}__tmp_out}}\" 2>\"${{{}__tmp_err}}\"; then {}__status=0; else {}__status=$?; fi;\n",
                    cmd, name, name, name, name
                ));

                // Read output
                out.push_str(&format!(
                    "{pad}{name}__stdout=\"$(__sh2_read_file \"${name}__tmp_out\")\"\n",
                    pad = pad,
                    name = name
                ));
                out.push_str(&format!(
                    "{pad}{name}__stderr=\"$(__sh2_read_file \"${name}__tmp_err\")\"\n",
                    pad = pad,
                    name = name
                ));

                // Cleanup
                out.push_str(&format!(
                    "{pad}rm -f \"${name}__tmp_out\" \"${name}__tmp_err\"\n",
                    pad = pad,
                    name = name
                ));

                // Propagate status purely for __sh2_status tracking, though try_run succeeded as a statement.
                out.push_str(&format!(
                    "{pad}__sh2_status=\"${name}__status\"\n",
                    pad = pad,
                    name = name
                ));
            } else if let Val::Args = val {
                out.push_str(name);
                out.push_str("=(\"$@\")\n");
            } else {
                out.push_str(name);
                out.push('=');
                out.push_str(&emit_val(val, target)?);
                out.push('\n');
            }
        }
        Cmd::Exec {
            args,
            allow_fail,
            loc,
        } => {
            if let Some(l) = loc {
                // In condition context, suppress error location reporting to avoid noise before catch
                if !in_cond_ctx {
                    out.push_str(&format!("{}__sh2_loc=\"{}\"\n", pad, l));
                }
            }
            out.push_str(&pad);
            let shell_cmd = args
                .iter()
                .map(|a| emit_word(a, target))
                .collect::<Result<Vec<_>, _>>()?
                .join(" ");

            if *allow_fail {
                // allow_fail: suppresses script failure (returns 0), captures real status in __sh2_status
                // We use || to suppress 'set -e' and 'trap ERR' for the command.
                out.push_str(&format!(
                    "__sh2_status=0; {} || __sh2_status=$?; :\n",
                    shell_cmd
                ));
            } else {
                // Normal: capture status in __sh2_status, then restore $? so try/set-e works
                out.push_str(&shell_cmd);
                out.push_str("; __sh2_status=$?; ");
                match target {
                    TargetShell::Bash => {
                        out.push_str("(exit $__sh2_status)\n");
                    }
                    TargetShell::Posix => {
                        if in_cond_ctx {
                            // In condition context (e.g. try block), we must NOT exit the script.
                            // We use (exit $s) to set $? and trigger errexit if active (which catch handles).
                            out.push_str("(exit $__sh2_status)\n");
                        } else {
                            // For POSIX, like bash: just set $? for try/catch contexts
                            out.push_str("(exit $__sh2_status)\n");
                        }
                    }
                }
            }
        }

        Cmd::ExecReplace(args, loc) => {
             if let Some(l) = loc {
                 out.push_str(&format!("{}__sh2_loc=\"{}\"\n", pad, l));
             }
             out.push_str(&pad);
             out.push_str("exec ");
             let shell_args: Vec<String> = args.iter().map(|a| emit_word(a, target)).collect::<Result<Vec<_>, _>>()?;
             out.push_str(&shell_args.join(" "));
             out.push('\n');
        }
        Cmd::Print(val) => {
            out.push_str(&pad);
            out.push_str("printf '%s\\n' ");
            match val {
                Val::Args => out.push_str("\"$*\""),
                _ => out.push_str(&emit_val(val, target)?),
            }
            out.push('\n');
        }
        Cmd::PrintErr(val) => {
            out.push_str(&pad);
            out.push_str("printf '%s\\n' ");
            match val {
                Val::Args => out.push_str("\"$*\""),
                _ => out.push_str(&emit_val(val, target)?),
            }
            out.push_str(" >&2\n");
        }
        Cmd::If {
            cond,
            then_body,
            elifs,
            else_body,
        } => {
            let cond_str = emit_cond(cond, target)?;
            out.push_str(&format!("{pad}if {cond_str}; then\n"));
            for c in then_body {
                emit_cmd(c, out, indent + 2, target, in_cond_ctx)?;
            }

            for (cond, body) in elifs {
                let cond_str = emit_cond(cond, target)?;
                out.push_str(&format!("{pad}elif {cond_str}; then\n"));
                for c in body {
                    emit_cmd(c, out, indent + 2, target, in_cond_ctx)?;
                }
            }

            if !else_body.is_empty() {
                out.push_str(&format!("{pad}else\n"));
                for c in else_body {
                    emit_cmd(c, out, indent + 2, target, in_cond_ctx)?;
                }
            }

            out.push_str(&format!("{pad}fi\n"));
        }
        Cmd::Pipe(segments, loc) => {
            if let Some(l) = loc {
                out.push_str(&format!("{}__sh2_loc=\"{}\"\n", pad, l));
            }
            // Bash: use subshell with pipefail for robust status capture
            // POSIX: manual pipeline with FIFOs via helper

            let last_idx = segments.len() - 1;
            let allow_fail_last = segments[last_idx].1;

            match target {
                TargetShell::Bash => {
                    out.push_str(&pad);

                    let mut pipe_str = String::new();
                    for (i, (args, allow_fail)) in segments.iter().enumerate() {
                        if i > 0 {
                            pipe_str.push_str(" | ");
                        }

                        let cmd_str = args
                            .iter()
                            .map(|a| emit_word(a, target))
                            .collect::<Result<Vec<_>, _>>()?
                            .join(" ");

                        if *allow_fail {
                            if i < last_idx {
                                // Non-final segment: suppress failure
                                pipe_str.push_str(&format!("if {}; then :; else true; fi", cmd_str));
                            } else {
                                // Final segment: suppress ERR trap via 'if' context, preserve exit status
                                pipe_str.push_str(&format!("if {}; then :; else exit $?; fi", cmd_str));
                            }
                        } else {
                            pipe_str.push_str(&cmd_str);
                        }
                    }

                     // Wrap in subshell to isolate set -o pipefail and set +e
                     // Use 'if' to capture status while suppressing ERR trap for the pipeline itself.
                     out.push_str(&format!(
                          "if ( set -o pipefail; set +e; {} ); then __sh2_status=0; else __sh2_status=$?; fi; ",
                          pipe_str
                      ));

                    // Return
                    if allow_fail_last {
                        out.push_str(":\n");
                    } else {
                        out.push_str("(exit $__sh2_status)\n");
                    }
                }
                TargetShell::Posix => {
                    let stages: Vec<String> = segments
                        .iter()
                        .map(|(args, _)| {
                            let parts = args.iter()
                                .map(|a| emit_word(a, target))
                                .collect::<Result<Vec<_>, _>>()?;
                            Ok(parts.join(" "))
                        })
                        .collect::<Result<_, _>>()?;
                    let allow_fails: Vec<bool> = segments.iter().map(|(_, af)| *af).collect();

                    emit_posix_pipeline(
                        out,
                        &pad,
                        target,
                        &stages,
                        &allow_fails,
                        allow_fail_last,
                        loc.is_some(),
                    );
                }
            }
        }
        Cmd::PipeBlocks(segments, loc) => {
            if let Some(l) = loc {
                out.push_str(&format!("{}__sh2_loc=\"{}\"\n", pad, l));
            }
            match target {
                TargetShell::Bash => {
                    out.push_str(&pad);
                    let mut pipe_str = String::new();
                    for (i, seg) in segments.iter().enumerate() {
                        if i > 0 {
                            pipe_str.push_str(" | ");
                        }
                        pipe_str.push_str("{\n");
                        for cmd in seg {
                            emit_cmd(cmd, &mut pipe_str, indent + 2, target, false)?;
                        }
                        pipe_str.push_str(&format!("{pad}}}"));
                    }

                    out.push_str(&format!(
                        "if ( set -o pipefail; set +e; {} ); then __sh2_status=0; else __sh2_status=$?; fi; ",
                        pipe_str
                    ));

                    out.push_str("(exit $__sh2_status)\n");
                }
                TargetShell::Posix => {
                    let stages: Vec<String> = segments
                        .iter()
                        .map(|seg| {
                            let mut s = String::new();
                            s.push_str("{\n");
                            for cmd in seg {
                                emit_cmd(cmd, &mut s, indent + 2, target, false)?;
                            }
                            s.push_str(&format!("{pad}}}"));
                            Ok(s)
                        })
                        .collect::<Result<_, _>>()?;
                    let allow_fails = vec![false; segments.len()];
                    emit_posix_pipeline(
                        out,
                        &pad,
                        target,
                        &stages,
                        &allow_fails,
                        false,
                        loc.is_some(),
                    );
                }
            }
        }
        Cmd::Case { expr, arms } => {
            out.push_str(&format!("{}case {} in\n", pad, emit_val(expr, target)?));
            for (patterns, body) in arms {
                out.push_str(&pad);
                out.push_str("  ");
                let pat_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| match p {
                        crate::ir::Pattern::Literal(s) => sh_single_quote(s),
                        crate::ir::Pattern::Glob(s) => emit_case_glob_pattern(s),
                        crate::ir::Pattern::Wildcard => "*".to_string(),
                    })
                    .collect();
                out.push_str(&pat_strs.join("|"));
                out.push_str(")\n");

                for cmd in body {
                    emit_cmd(cmd, out, indent + 4, target, in_cond_ctx)?;
                }
                out.push_str(&format!("{}  ;;\n", pad));
            }
            out.push_str(&format!("{}esac\n", pad));
        }

        Cmd::While { cond, body } => {
            let cond_str = emit_cond(cond, target)?;
            out.push_str(&format!("{pad}while {cond_str}; do\n"));
            for c in body {
                emit_cmd(c, out, indent + 2, target, in_cond_ctx)?;
            }
            out.push_str(&format!("{pad}done\n"));
        }
        Cmd::Require(cmds) => {
            out.push_str(&pad);
            out.push_str("__sh2_require");
            for cmd in cmds {
                out.push(' ');
                out.push_str(&emit_word(cmd, target)?);
            }
            out.push('\n');
            out.push_str(&format!("{}__sh2_status=$?; (exit $__sh2_status)\n", pad));
        }
        Cmd::Log {
            level,
            msg,
            timestamp,
        } => {
            out.push_str(&pad);
            out.push_str("__sh2_log ");
            match level {
                LogLevel::Info => out.push_str("'INFO' "),
                LogLevel::Warn => out.push_str("'WARN' "),
                LogLevel::Error => out.push_str("'ERROR' "),
            }
            out.push_str(&emit_word(msg, target)?);
            out.push(' ');
            if *timestamp {
                out.push_str("'true'\n");
            } else {
                out.push_str("'false'\n");
            }
        }
        Cmd::WriteFile {
            path,
            content,
            append,
        } => {
            out.push_str(&pad);
            out.push_str("__sh2_write_file ");
            out.push_str(&emit_word(path, target)?);
            out.push(' ');
            out.push_str(&emit_word(content, target)?);
            out.push(' ');
            if *append {
                out.push_str("'true'");
            } else {
                out.push_str("'false'");
            }
            out.push('\n');
        }
        Cmd::For { var, items, body } => {
            out.push_str(&format!("{}for {} in", pad, var));
            for item in items {
                match item {
                    Val::List(elems) => {
                        for elem in elems {
                            out.push(' ');
                            out.push_str(&emit_val(elem, target)?);
                        }
                    }
                    Val::Args => {
                        out.push_str(" \"$@\"");
                    }
                    Val::Var(name) => {
                        if target == TargetShell::Posix {
                            return Err(CompileError::unsupported("Iterating over array variable not supported in POSIX", target));
                        }
                        out.push_str(&format!(" \"${{{}[@]}}\"", name));
                    }
                    _ => {
                        out.push(' ');
                        out.push_str(&emit_val(item, target)?);
                    }
                }
            }
            out.push_str("; do\n");
            for c in body {
                emit_cmd(c, out, indent + 2, target, in_cond_ctx)?;
            }
            out.push_str(&format!("{}done\n", pad));
        }
        Cmd::ForMap {
            key_var,
            val_var,
            map,
            body,
        } => {
            if target == TargetShell::Posix {
                return Err(CompileError::unsupported("map/dict is only supported in Bash target", target));
            }
            // Iterate over keys array for deterministic order
            // for __sh2_k in "${__sh2_keys_map[@]}"; do
            //   local key="$__sh2_k"
            //   local val="${map[$__sh2_k]}"
            //   ...
            out.push_str(&format!(
                "{pad}for __sh2_k in \"${{__sh2_keys_{}[@]}}\"; do\n",
                map
            ));
            out.push_str(&format!("{pad}  local {}=\"$__sh2_k\"\n", key_var));
            out.push_str(&format!(
                "{pad}  local {}=\"${{{}[$__sh2_k]}}\"\n",
                val_var, map
            ));

            for c in body {
                emit_cmd(c, out, indent + 2, target, in_cond_ctx)?;
            }

            out.push_str(&format!("{pad}done\n"));
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
                    let cond_str = emit_cond(v, target)?;
                    out.push_str(&format!(
                        "{pad}if {}; then return 0; else return 1; fi\n",
                        cond_str
                    ));
                } else {
                    out.push_str(&format!("{pad}return {}\n", emit_val(v, target)?));
                }
            } else {
                out.push_str(&format!("{pad}return\n"));
            }
        }
        Cmd::Exit(val) => {
            if let Some(v) = val {
                if is_boolean_expr(v) {
                    let cond_str = emit_cond(v, target)?;
                    out.push_str(&format!(
                        "{pad}if {}; then exit 0; else exit 1; fi\n",
                        cond_str
                    ));
                } else {
                    out.push_str(&format!("{pad}exit {}\n", emit_val(v, target)?));
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
                        out.push_str(&format!("{}={} ", k, emit_val(v, target)?));
                    }
                    let shell_args: Vec<String> =
                        args.iter().map(|a| emit_word(a, target)).collect::<Result<Vec<_>, _>>()?;
                    out.push_str(&shell_args.join(" "));
                    out.push('\n');
                    return Ok(());
                }
            }

            // General case: Subshell
            out.push_str(&format!("{pad}(\n"));
            for (k, v) in bindings {
                out.push_str(&format!("{}  export {}={}\n", pad, k, emit_val(v, target)?));
            }
            for cmd in body {
                emit_cmd(cmd, out, indent + 2, target, in_cond_ctx)?;
            }
            out.push_str(&format!("{pad})\n"));
        }
        Cmd::WithLog { path, append, body } => {
            if target == TargetShell::Posix {
                return Err(CompileError::unsupported("with log(...) is not supported in POSIX sh target", target));
            }

            // Bash implementation using process substitution
            let path_val = emit_val(path, target)?;

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
            out.push_str(&format!(
                "{pad}  exec 2> >(tee -a \"$__sh2_log_path\" >&2)\n"
            ));

            for cmd in body {
                emit_cmd(cmd, out, indent + 2, target, in_cond_ctx)?;
            }
            out.push_str(&format!("{pad})\n"));
        }
        Cmd::WithCwd { path, body } => {
            out.push_str(&format!("{pad}(\n"));
            out.push_str(&format!("{pad}  cd {}\n", emit_val(path, target)?));
            for cmd in body {
                emit_cmd(cmd, out, indent + 2, target, in_cond_ctx)?;
            }
            out.push_str(&format!("{pad})\n"));
        }
        Cmd::Cd(path) => {
            out.push_str(&pad);
            out.push_str("cd ");
            out.push_str(&emit_val(path, target)?);
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
                out.push_str(&emit_word(arg, target)?);
            }
            out.push_str("; __sh2_status=$?; (exit $__sh2_status)\n");
        }
        Cmd::Subshell { body } => {
            out.push_str(&format!("{pad}(\n"));
            for cmd in body {
                emit_cmd(cmd, out, indent + 2, target, in_cond_ctx)?;
            }
            out.push_str(&format!("{pad})\n"));
        }
        Cmd::Group { body } => {
            out.push_str(&format!("{pad}{{\n"));
            for cmd in body {
                emit_cmd(cmd, out, indent + 2, target, in_cond_ctx)?;
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        Cmd::WithRedirect {
            stdout,
            stderr,
            stdin,
            body,
        } => {
            out.push_str(&format!("{pad}{{\n"));
            for cmd in body {
                emit_cmd(cmd, out, indent + 2, target, in_cond_ctx)?;
            }
            out.push_str(&format!("{pad}}}")); // No newline yet, redirections follow

            // Handle regular stdin file redirection (before others, or order doesn't matter much for inputs vs outputs)
            // But preserving existing behavior: currently it emits stdin first.
            // Plan said: "Keep existing stdin handling position, but since heredoc is not a < redirection, emit heredoc operator after stdout/stderr redirections"

            let mut heredoc_content = None;

            if let Some(target_redir) = stdin {
                match target_redir {
                    RedirectTarget::File { path, .. } => {
                        out.push_str(&format!(" < {}", emit_val(path, target)?));
                    }
                    RedirectTarget::HereDoc { content } => {
                        heredoc_content = Some(content);
                    }
                    _ => return Err(CompileError::unsupported("stdin redirected to something invalid", target)),
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
                        return Err(CompileError::unsupported(
                            "Cyclic redirection: stdout to stderr AND stderr to stdout is not supported"
                        , target));
                    }
                }
            }

            let emit_stdout = |out: &mut String| -> Result<(), CompileError> {
                if let Some(target_redir) = &stdout {
                    match target_redir {
                        RedirectTarget::File { path, append } => {
                            let op = if *append { ">>" } else { ">" };
                            out.push_str(&format!(" {} {}", op, emit_val(path, target)?));
                        }
                        RedirectTarget::Stderr => {
                            out.push_str(" 1>&2");
                        }
                        RedirectTarget::Stdout => {
                            // no-op
                        }
                        RedirectTarget::HereDoc { .. } => return Err(CompileError::unsupported("Heredoc not valid for stdout", target)),
                    }
                }
                Ok(())
            };

            let emit_stderr = |out: &mut String| -> Result<(), CompileError> {
                if let Some(target_redir) = &stderr {
                    match target_redir {
                        RedirectTarget::File { path, append } => {
                            let op = if *append { ">>" } else { ">" };
                            out.push_str(&format!(" 2{} {}", op, emit_val(path, target)?));
                        }
                        RedirectTarget::Stdout => {
                            out.push_str(" 2>&1");
                        }
                        RedirectTarget::Stderr => {
                            // no-op
                        }
                        RedirectTarget::HereDoc { .. } => return Err(CompileError::unsupported("Heredoc not valid for stderr", target)),
                    }
                }
                Ok(())
            };

            if emit_stdout_first {
                emit_stdout(out)?;
                emit_stderr(out)?;
            } else {
                emit_stderr(out)?;
                emit_stdout(out)?;
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
            // Wrap the entire spawned command in a subshell so & applies to the whole unit.
            // This ensures $! refers to the subshell running the actual work.
            // For Cmd::Exec, emit just the raw command without status tracking.
            out.push_str(&pad);
            out.push_str("( ");

            match cmd.as_ref() {
                Cmd::Exec {
                    args,
                    allow_fail: _,
                    loc,
                } => {
                    // Simple command: emit inline
                    if let Some(l) = loc {
                        out.push_str(&format!("__sh2_loc=\"{}\"; ", l));
                    }
                    let shell_cmd = args
                        .iter()
                        .map(|a| emit_word(a, target))
                        .collect::<Result<Vec<_>, _>>()?
                        .join(" ");
                    out.push_str(&shell_cmd);
                    out.push_str(" ) &\n");
                }
                _ => {
                    // Complex command (block, group, etc): emit with increased indent
                    out.push('\n');
                    emit_cmd(cmd, out, indent + 2, target, false)?;
                    out.push_str(&pad);
                    out.push_str(") &\n");
                }
            }
        }
        Cmd::Wait(opt) => {
            // Wait must update __sh2_status to the exit status of the waited process
            match opt {
                Some(val) => match val {
                    crate::ir::Val::List(elems) => {
                        out.push_str(&format!("{pad}wait"));
                        for elem in elems {
                            out.push(' ');
                            out.push_str(&emit_word(elem, target)?);
                        }
                        out.push_str("; __sh2_status=$?; (exit $__sh2_status)\n");
                    }
                    _ => {
                        out.push_str(&format!(
                            "{pad}wait {}; __sh2_status=$?; (exit $__sh2_status)\n",
                            emit_word(val, target)?
                        ));
                    }
                },
                None => out.push_str(&format!("{pad}wait; __sh2_status=$?\n")),
            }
        }
        Cmd::TryCatch {
            try_body,
            catch_body,
        } => {
            // Setup: Save errexit state and disable it.
            // Bash: also save ERR trap.
            match target {
                TargetShell::Bash => {
                    out.push_str(&format!(
                        "{pad}local __sh2_e=0; case $- in *e*) __sh2_e=1;; *) __sh2_e=0;; esac; set +e\n"
                    ));
                    out.push_str(&format!(
                        "{pad}local __sh2_err=$(trap -p ERR || true); trap - ERR\n"
                    ));
                }
                TargetShell::Posix => {
                    out.push_str(&format!(
                        "{pad}case $- in *e*) __sh2_e=1;; *) __sh2_e=0;; esac; set +e\n"
                    ));
                }
            }

            // Try body
            out.push_str(&format!("{pad}if {{\n"));
            if try_body.is_empty() {
                out.push_str(&format!("{pad}  :\n"));
            } else {
                for (i, cmd) in try_body.iter().enumerate() {
                    if i > 0 {
                        out.push_str(&format!("{pad}  }} && {{\n"));
                    } else {
                        out.push_str(&format!("{pad}  {{\n"));
                    }
                    // Emit command directly with increased indent and in condition context
                    emit_cmd(cmd, out, indent + 4, target, true)?;
                }
                out.push_str(&format!("{pad}  }}\n"));
            }
            out.push_str(&format!("{pad}}}; then\n"));

            // Helper to emit restoration logic
            let emit_restore = |out: &mut String| match target {
                TargetShell::Bash => {
                    out.push_str(&format!(
                        "{pad}  if [ -n \"$__sh2_err\" ]; then eval \"$__sh2_err\"; fi; if [ \"$__sh2_e\" = 1 ]; then set -e; fi\n"
                    ));
                }
                TargetShell::Posix => {
                    // Safe reference to __sh2_e using :-0 to prevent nounset errors if somehow undefined
                    out.push_str(&format!(
                        "{pad}  if [ \"${{__sh2_e:-0}}\" = 1 ]; then set -e; fi\n"
                    ));
                }
            };

            emit_restore(out);
            out.push_str(&format!("{pad}else\n"));
            emit_restore(out);

            // Catch body
            if catch_body.is_empty() {
                out.push_str(&format!("{pad}  :\n"));
            }
            for cmd in catch_body {
                emit_cmd(cmd, out, indent + 2, target, in_cond_ctx)?;
            }
            out.push_str(&format!("{pad}fi\n"));
        }
        Cmd::AndThen { left, right } => {
            out.push_str(&format!("{pad}{{\n"));
            for cmd in left {
                emit_cmd(cmd, out, indent + 2, target, in_cond_ctx)?;
            }
            out.push_str(&format!("{pad}}} && {{\n"));
            for cmd in right {
                emit_cmd(cmd, out, indent + 2, target, in_cond_ctx)?;
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        Cmd::OrElse { left, right } => {
            out.push_str(&format!("{pad}{{\n"));
            for cmd in left {
                emit_cmd(cmd, out, indent + 2, target, in_cond_ctx)?;
            }
            out.push_str(&format!("{pad}}} || {{\n"));
            for cmd in right {
                emit_cmd(cmd, out, indent + 2, target, in_cond_ctx)?;
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        Cmd::Export { name, value } => {
            out.push_str(&pad);
            out.push_str("export ");
            out.push_str(name);
            if let Some(v) = value {
                out.push('=');
                out.push_str(&emit_val(v, target)?);
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
            out.push_str(&emit_word(path, target)?);
            out.push('\n');
        }
        Cmd::SaveEnvfile { path, env } => {
            out.push_str(&pad);
            out.push_str("__sh2_save_envfile ");
            out.push_str(&emit_word(path, target)?);
            out.push(' ');
            out.push_str(&emit_val(env, target)?);
            out.push_str("; __sh2_status=$?; (exit $__sh2_status)\n");
        }
    }

    Ok(())}

fn is_boolean_expr(v: &Val) -> bool {
    matches!(
        v,
        Val::Compare { .. }
            | Val::And(..)
            | Val::Or(..)
            | Val::Not(..)
            | Val::Exists(..)
            | Val::IsDir(..)
            | Val::IsFile(..)
            | Val::IsSymlink(..)
            | Val::IsExec(..)
            | Val::IsReadable(..)
            | Val::IsWritable(..)
            | Val::IsNonEmpty(..)
            | Val::Bool(..)
            | Val::Matches(..)
    )
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
    target: TargetShell,
    stages: &[String],
    allow_fails: &[bool],
    allow_fail_last: bool,
    _has_loc: bool,
) {
    // POSIX sh manual pipeline using FIFOs to simulate pipefail without deadlocks.
    // This implementation:
    // - Is errexit-safe: saves/restores set -e state
    // - Saves/restores user traps using POSIX-safe $(trap)
    // - Cleans up FIFOs on success or failure
    //
    // Note: We use $(trap) to capture all current traps as a shell-re-evaluable string.
    // This is POSIX-compliant (unlike trap -p which is not).

    out.push_str(pad);
    out.push_str("{\n");
    let indent_pad = format!("{}  ", pad);

    // Save errexit state and user traps, then disable errexit
    out.push_str(&format!(
        "{}case $- in *e*) __sh2_e=1;; *) __sh2_e=0;; esac; set +e;\n",
        indent_pad
    ));
    out.push_str(&format!("{}__sh2_saved_traps=$(trap);\n", indent_pad));

    let num_fifos = stages.len() - 1;
    out.push_str(&format!(
        "{}__sh2_base=\"${{TMPDIR:-/tmp}}/sh2_fifo_$$\";\n",
        indent_pad
    ));
    out.push_str(&format!("{}rm -f \"${{__sh2_base}}_\"*;\n", indent_pad));

    for i in 0..num_fifos {
        out.push_str(&format!(
            "{}mkfifo \"${{__sh2_base}}_{}\";\n",
            indent_pad, i
        ));
    }

    // Set cleanup traps
    out.push_str(&format!(
        "{}trap 'rm -f \"${{__sh2_base}}_\"*' EXIT;\n",
        indent_pad
    ));
    out.push_str(&format!(
        "{}trap 'rm -f \"${{__sh2_base}}_\"*; exit 1' INT TERM QUIT;\n",
        indent_pad
    ));

    // Open keepalive FDs (fd 3+)
    out.push_str(&format!("{}__sh2_fd=3;\n", indent_pad));
    out.push_str(&format!("{}__sh2_fds=\"\";\n", indent_pad));
    for i in 0..num_fifos {
        out.push_str(&format!(
            "{}eval \"exec ${{__sh2_fd}}<>\\\"${{__sh2_base}}_{}\\\"\";\n",
            indent_pad, i
        ));
        out.push_str(&format!(
            "{}__sh2_fds=\"$__sh2_fds $__sh2_fd\";\n",
            indent_pad
        ));
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
        out.push_str(&format!(
            "{}( for fd in $__sh2_fds; do eval \"exec $fd>&-\"; done; {} ) {} & __sh2_p{}=$!;\n",
            indent_pad, cmd, redir, i
        ));
    }

    // Close keepalive FDs in parent
    out.push_str(&format!(
        "{}for fd in $__sh2_fds; do eval \"exec $fd>&-\"; done;\n",
        indent_pad
    ));

    // Wait and collect statuses
    for i in 0..stages.len() {
        out.push_str(&format!(
            "{}wait \"$__sh2_p{}\"; __sh2_s{}=$?;\n",
            indent_pad, i, i
        ));
    }

    // Compute effective status (rightmost non-zero wins, ignoring allow_fail stages)
    out.push_str(&format!("{}__sh2_status=0;\n", indent_pad));
    for i in 0..stages.len() {
        if !allow_fails[i] || i == stages.len() - 1 {
            out.push_str(&format!(
                "{}if [ \"$__sh2_s{}\" -ne 0 ]; then __sh2_status=\"$__sh2_s{}\"; fi;\n",
                indent_pad, i, i
            ));
        }
    }

    // Cleanup FIFOs and reset traps to default
    out.push_str(&format!("{}trap - EXIT INT TERM QUIT;\n", indent_pad));
    out.push_str(&format!("{}rm -f \"${{__sh2_base}}_\"*;\n", indent_pad));

    // Restore user traps (if any were saved)
    out.push_str(&format!(
        "{}if [ -n \"$__sh2_saved_traps\" ]; then eval \"$__sh2_saved_traps\"; fi;\n",
        indent_pad
    ));

    // Restore errexit if it was set
    out.push_str(&format!(
        "{}if [ \"$__sh2_e\" = 1 ]; then set -e; fi;\n",
        indent_pad
    ));

    // Return status
    if allow_fail_last {
        out.push_str(&format!("{}:\n", indent_pad));
    } else {
        match target {
            TargetShell::Bash => out.push_str(&format!("{}(exit $__sh2_status)\n", indent_pad)),
            TargetShell::Posix => {
                // Do not explicitly exit (which kills script if no subshell) or print (because we lack trap ERR equivalent)
                // Just relying on (exit) to set status and trigger errexit if enabled.
                out.push_str(&format!("{}(exit $__sh2_status)\n", indent_pad));
            }
        }
    }

    out.push_str(&format!("{}}}\n", pad));
}

fn emit_prelude(target: TargetShell, usage: &PreludeUsage) -> String {
    let mut s = String::new();

    if usage.coalesce {
        s.push_str("__sh2_coalesce() { if [ -n \"$1\" ]; then printf '%s' \"$1\"; else printf '%s' \"$2\"; fi; }\n");
    }
    if usage.trim {
        s.push_str(r#"__sh2_trim() { awk -v s="$1" 'BEGIN { sub(/^[[:space:]]+/, "", s); sub(/[[:space:]]+$/, "", s); printf "%s", s }'; }
"#);
    }
    if usage.before {
        s.push_str(r#"__sh2_before() { awk -v s="$1" -v sep="$2" 'BEGIN { n=index(s, sep); if(n==0) printf "%s", s; else printf "%s", substr(s, 1, n-1) }'; }
"#);
    }
    if usage.after {
        s.push_str(r#"__sh2_after() { awk -v s="$1" -v sep="$2" 'BEGIN { n=index(s, sep); if(n==0) printf ""; else printf "%s", substr(s, n+length(sep)) }'; }
"#);
    }
    if usage.replace {
        s.push_str(r#"__sh2_replace() { awk -v s="$1" -v old="$2" -v new="$3" 'BEGIN { if(old=="") { printf "%s", s; exit } len=length(old); while(i=index(s, old)) { printf "%s%s", substr(s, 1, i-1), new; s=substr(s, i+len) } printf "%s", s }'; }
"#);
    }
    if usage.split {
        s.push_str(r#"__sh2_split() { awk -v s="$1" -v sep="$2" 'BEGIN { if(sep=="") { printf "%s", s; exit } len=length(sep); while(i=index(s, sep)) { printf "%s\n", substr(s, 1, i-1); s=substr(s, i+len) } printf "%s", s }'; }
"#);
    }

    match target {
        TargetShell::Bash => {
            if usage.loc {
                s.push_str("__sh2_err_handler() { local s=$?; if [[ \"${BASH_COMMAND}\" == *\"(exit \"* ]]; then return $s; fi; printf \"Error in %s\\n\" \"${__sh2_loc:-unknown}\" >&2; return $s; }\n");
                s.push_str("set -o errtrace\n");
                s.push_str("trap '__sh2_err_handler' ERR\n");
            }
            if usage.matches {
                s.push_str("__sh2_matches() { [[ \"$1\" =~ $2 ]]; }\n");
            }
            if usage.parse_args {
                s.push_str(r#"__sh2_parse_args() {
  local out="" key val
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --) shift; while [ "$#" -gt 0 ]; do out="${out}P	${1}
"; shift; done; break ;;
      --*=*) key="${1%%=*}"; val="${1#*=}"; out="${out}F	${key}	${val}
" ;;
      --*) key="$1"; if [ "$#" -gt 1 ] && [ "${2}" != "--" ] && [[ ! "$2" =~ ^-- ]]; then val="$2"; shift; else val="true"; fi; out="${out}F	${key}	${val}
" ;;
      *) out="${out}P	${1}
" ;;
    esac
    shift
  done
  printf '%s' "$out"
}
"#);
            }
        }
        TargetShell::Posix => {
            if usage.matches {
                s.push_str(
                    r#"__sh2_matches() { printf '%s\n' "$1" | grep -Eq -- "$2"; }
"#,
                );
            }
            if usage.parse_args {
                s.push_str(r#"__sh2_parse_args() {
  __out="" 
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --) shift; while [ "$#" -gt 0 ]; do __out="${__out}P	${1}
"; shift; done; break ;;
      --*=*) __key="${1%%=*}"; __val="${1#*=}"; __out="${__out}F	${__key}	${__val}
" ;;
      --*) __key="$1"; __f=0; case "$2" in --*) __f=1;; esac
           if [ "$#" -gt 1 ] && [ "${2}" != "--" ] && [ "$__f" = 0 ]; then __val="$2"; shift; else __val="true"; fi
           __out="${__out}F	${__key}	${__val}
" ;;
      *) __out="${__out}P	${1}
" ;;
    esac
    shift
  done
  printf '%s' "$__out"
}
"#);
            }
        }
    }

    if usage.args_flags {
        s.push_str(
            r#"__sh2_args_flags() { printf '%s' "$1" | awk '/^F\t/ { sub(/^F\t/, ""); print }'; }
"#,
        );
    }
    if usage.args_positionals {
        s.push_str(r#"__sh2_args_positionals() { printf '%s' "$1" | awk '/^P\t/ { sub(/^P\t/, ""); print }'; }
"#);
    }
    if usage.args_flag_get {
        s.push_str(r#"__sh2_args_flag_get() { printf '%s' "$1" | awk -v k="$2" -F '\t' '{ if (sub(/^F\t/, "")) { if ($1==k) v=$2 } else if ($1==k) v=$2 } END { printf "%s", v }'; }
"#);
    }
    if usage.list_get {
        s.push_str(r#"__sh2_list_get() { printf '%s' "$1" | awk -v i="$2" 'NR==i+1 { printf "%s", $0; exit }'; }
"#);
    }
    if usage.load_envfile {
        s.push_str(r##"__sh2_load_envfile() { if [ -r "$1" ]; then awk '{ sub(/^[[:space:]]+/, ""); sub(/[[:space:]]+$/, ""); if($0=="" || substr($0,1,1)=="#") next; if(substr($0,1,7)=="export ") sub(/^export[[:space:]]+/, ""); idx=index($0,"="); if(idx==0) next; k=substr($0,1,idx-1); v=substr($0,idx+1); sub(/^[[:space:]]+/, "", k); sub(/[[:space:]]+$/, "", k); sub(/^[[:space:]]+/, "", v); sub(/[[:space:]]+$/, "", v); len=length(v); if(len>=2){ f=substr(v,1,1); l=substr(v,len,1); if((f=="\047" && l=="\047") || (f=="\"" && l=="\"")){ v=substr(v,2,len-2) } } printf "%s\t%s\n", k, v }' "$1" 2>/dev/null || true; fi; }
"##);
    }
    if usage.save_envfile {
        s.push_str(r#"__sh2_save_envfile() { printf '%s' "$2" | awk -F '\t' 'NF>=1{ print $1 "=" $2 }' > "$1"; }
"#);
    }
    if usage.json_kv {
        s.push_str(r#"__sh2_json_kv() { printf '%s' "$1" | awk -F '\t' 'function esc(s) { gsub(/\\/, "\\\\", s); gsub(/"/, "\\\"", s); gsub(/\t/, "\\t", s); gsub(/\r/, "\\r", s); gsub(/\n/, "\\n", s); return s; } { k=$1; v=$2; if (k == "") next; if (!(k in seen)) { ord[++n] = k; seen[k] = 1; } val[k] = v; } END { printf "{"; for (i=1; i<=n; i++) { k = ord[i]; v = val[k]; printf "%s\"%s\":\"%s\"", (i==1?"":","), esc(k), esc(v); } printf "}"; }'; }
"#);
    }
    if usage.which {
        s.push_str(
            r#"__sh2_which() { command -v -- "$1" 2>/dev/null || true; }
"#,
        );
    }
    if usage.require {
        s.push_str(r#"__sh2_require() { for c in "$@"; do if ! command -v -- "$c" >/dev/null 2>&1; then printf '%s\n' "missing required command: $c" >&2; exit 127; fi; done; }
"#);
    }
    if usage.tmpfile {
        s.push_str(r#"__sh2_tmpfile() { if command -v mktemp >/dev/null 2>&1; then mktemp; else printf "%s/sh2_tmp_%s_%s" "${TMPDIR:-/tmp}" "$$" "$(awk 'BEGIN{srand();print int(rand()*1000000)}')"; fi; }
"#);
    }
    if usage.read_file {
        s.push_str(
            r#"__sh2_read_file() { cat "$1" 2>/dev/null || true; }
"#,
        );
    }
    if usage.write_file {
        s.push_str(r#"__sh2_write_file() { if [ "$3" = "true" ]; then printf '%s' "$2" >> "$1"; else printf '%s' "$2" > "$1"; fi; }
"#);
    }
    if usage.log {
        s.push_str(r#"__sh2_log_now() { if [ -n "${SH2_LOG_TS:-}" ]; then printf '%s' "$SH2_LOG_TS"; return 0; fi; date '+%Y-%m-%dT%H:%M:%S%z' 2>/dev/null || date 2>/dev/null || printf '%s' 'unknown-time'; }
"#);
        s.push_str(r#"__sh2_log() { if [ "$3" = "true" ]; then printf '%s\t%s\t%s\n' "$(__sh2_log_now)" "$1" "$2" >&2; else printf '%s\t%s\n' "$1" "$2" >&2; fi; }
"#);
    }
    if usage.home {
        s.push_str(
            r#"__sh2_home() { printf '%s' "${HOME-}"; }
"#,
        );
    }
    if usage.path_join {
        s.push_str(r#"__sh2_path_join() { out=''; for p in "$@"; do [ -z "$p" ] && continue; case "$p" in /*) out="$p";; *) if [ -z "$out" ]; then out="$p"; else while [ "${out%/}" != "$out" ]; do out="${out%/}"; done; while [ "${p#/}" != "$p" ]; do p="${p#/}"; done; out="${out}/${p}"; fi;; esac; done; printf '%s' "$out"; }
"#);
    }

    if usage.uid {
        s.push_str("__sh2_uid=\"$(id -u 2>/dev/null || printf '%s' 0)\"\n");
    }
    s
}

fn is_prelude_helper(name: &str) -> bool {
    matches!(
        name,
        "trim" | "before" | "after" | "replace" | "split" | "coalesce"
    )
}
