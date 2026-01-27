pub mod posix_lint;
pub use posix_lint::{PosixLint, PosixLintKind, lint_script, render_lints};

use crate::error::CompileError;
use crate::ir::{Cmd, Function, LogLevel, RedirectOutputTarget, RedirectInputTarget, Val};
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
    pub lines: bool,
    pub contains: bool,
    pub starts_with: bool,
    pub arg_dynamic: bool,
    pub sh_probe: bool,
    pub confirm: bool,
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
            if let Some(targets) = stdout {
                for t in targets {
                    visit_redirect_output(t, usage);
                }
            }
            if let Some(targets) = stderr {
                for t in targets {
                    visit_redirect_output(t, usage);
                }
            }
            if let Some(t) = stdin {
                visit_redirect_input(t, usage);
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
        Cmd::Break | Cmd::Continue | Cmd::Unset(_) => {}
        Cmd::Return(opt) | Cmd::Exit(opt) => {
             if let Some(v) = opt {
                 visit_val(v, usage);
             }
        }
        Cmd::Raw(val, _) => {
             usage.sh_probe = true;
             visit_val(val, usage);
        }
        Cmd::RawLine { .. } => {}
        Cmd::Call { args, name } => {
            if name == "default" {
                usage.coalesce = true;
            }
            visit_val(&Val::Call { name: name.clone(), args: args.clone() }, usage);
        }
    }
}


fn visit_redirect_output(target: &RedirectOutputTarget, usage: &mut PreludeUsage) {
    match target {
        RedirectOutputTarget::File { path, .. } => visit_val(path, usage),
        _ => {}
    }
}

fn visit_redirect_input(target: &RedirectInputTarget, usage: &mut PreludeUsage) {
    match target {
        RedirectInputTarget::File { path } => visit_val(path, usage),
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
        Val::Lines(inner) => {
            usage.lines = true;
            visit_val(inner, usage);
        }
        Val::Split { s, delim } => {
            usage.split = true;
            visit_val(s, usage);
            visit_val(delim, usage);
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
        Val::Confirm { prompt, .. } => {
            visit_val(prompt, usage);
            usage.confirm = true;
        }
        Val::Matches(t, r) => {
            usage.matches = true;
            visit_val(t, usage);
            visit_val(r, usage);
        }
        Val::StartsWith { text, prefix } => {
            usage.starts_with = true;
            visit_val(text, usage);
            visit_val(prefix, usage);
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
        Val::Capture { value, allow_fail } => {
            if *allow_fail {
                usage.tmpfile = true;
            }
            visit_val(value, usage);
        }
        Val::Uid => {
            usage.uid = true;
        }
        Val::Contains { list, needle } => {
            usage.contains = true;
            visit_val(list, usage);
            visit_val(needle, usage);
        }
        Val::ArgDynamic(index) => {
            usage.arg_dynamic = true;
            visit_val(index, usage);
        }
        Val::ContainsLine { text, needle } => {
            visit_val(text, usage);
            visit_val(needle, usage);
        }
        _ => {}
    }
}


pub fn emit_with_options(funcs: &[Function], opts: CodegenOptions) -> Result<String, CompileError> {
    let usage = scan_usage(funcs, opts.include_diagnostics);
    let mut out = String::new();

    // Emit shebang as the very first line
    out.push_str(shebang(opts.target));
    out.push('\n');

    // Usage-aware prelude emission
    out.push_str(&emit_prelude(opts.target, &usage));
    let mut ctx = CodegenContext::default();

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
            emit_cmd(cmd, &mut out, 2, opts, false, &mut ctx)?;
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


use std::collections::HashSet;

/// Check if a Val represents a boolean expression (comparison, logical op, predicate, etc.)
/// These require special handling when assigned to variables.
fn is_boolean_val(v: &Val) -> bool {
    matches!(
        v,
        Val::Bool(_)
            | Val::Compare { .. }
            | Val::And(_, _)
            | Val::Or(_, _)
            | Val::Not(_)
            | Val::Exists(_)
            | Val::IsDir(_)
            | Val::IsFile(_)
            | Val::IsSymlink(_)
            | Val::IsExec(_)
            | Val::IsReadable(_)
            | Val::IsWritable(_)
            | Val::IsNonEmpty(_)
            | Val::Matches(_, _)
            | Val::Contains { .. }
            | Val::StartsWith { .. }
            | Val::ContainsLine { .. }
            | Val::Confirm { .. }
    )
}

#[derive(Default)]
struct CodegenContext {
    known_lists: HashSet<String>,
    uid_counter: usize,
}

impl CodegenContext {
    fn next_id(&mut self) -> usize {
        let id = self.uid_counter;
        self.uid_counter += 1;
        id
    }
}

fn emit_val(v: &Val, target: TargetShell) -> Result<String, CompileError> {
    match v {
        Val::Contains { .. } => {
             let cond = emit_cond(v, target)?;
             Ok(format!("\"$( if {}; then printf true; else printf false; fi )\"", cond))
        }
        Val::Literal(s) => Ok(sh_single_quote(s)),
        Val::Var(s) => Ok(format!("\"${}\"", s)),
        Val::Concat(l, r) => Ok(format!("{}{}", emit_val(l, target)?, emit_val(r, target)?)),
        Val::TryRun(_) => Err(CompileError::unsupported("try_run() must be bound via let (e.g., let r = try_run(...))", target)),
        Val::Which(arg) => Ok(format!("\"$( __sh2_which {} )\"", emit_word(arg, target)?)),
        Val::ReadFile(arg) => {
            let path = emit_word(arg, target)?;
            match target {
                TargetShell::Bash => Ok(format!("\"$( trap '' ERR; __sh2_read_file {} )\"", path)),
                _ => Ok(format!("\"$( __sh2_read_file {} )\"", path)),
            }
        }




        Val::Home => Ok("\"$( __sh2_home )\"".to_string()),
        Val::PathJoin(args) => {
            let parts: Vec<String> = args.iter().map(|a| emit_word(a, target)).collect::<Result<_, _>>()?;
            Ok(format!("\"$( __sh2_path_join {} )\"", parts.join(" ")))
        }
        Val::Command(args) => {
            let parts: Vec<String> = args.iter().map(|a| emit_word(a, target)).collect::<Result<_, _>>()?;
            Ok(format!("\"$( {} )\"", parts.join(" ")))
        }
        Val::Capture { value, allow_fail } => {
            if *allow_fail {
                return Err(CompileError::new("capture(..., allow_fail=true) is only allowed in 'let' assignment (e.g. let res = capture(...))").with_target(target));
            }
            emit_val(value, target)
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
        Val::ArgDynamic(index) => {
            let idx_str = emit_arg_index_word(index, target)?;
            // idx_str is already quoted (e.g. "$i" or "$((...))"), so we don't quote it here
            Ok(format!("\"$( __sh2_arg_by_index {} \"$@\" )\"", idx_str))
        }
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
        Val::Bool(_) => Err(CompileError::new(
            "Cannot emit boolean value as string/word; booleans are only valid in conditions",
        ).with_target(target)),
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
                "\"$( ( unset {0}; printenv {0} ) 2>/dev/null || printenv {0} 2>/dev/null || true )\"",
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
        Val::Lines(_) => Err(CompileError::unsupported(
            "lines() is only valid in 'for' loops or 'let' assignment",
            target,
        )),
        Val::JsonKv(blob) => {
            Ok(format!("\"$( __sh2_json_kv {} )\"", emit_word(blob, target)?))
        }
        Val::Matches(..) | Val::StartsWith { .. } => {
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
        | Val::Split { .. }
        | Val::ContainsLine { .. }
        | Val::Confirm { .. } => Err(CompileError::new("Cannot emit boolean/list value as string").with_target(target)),
        Val::BoolVar(_) => Err(CompileError::new(
            "bool is not a string; bool→string conversion is not supported yet"
        ).with_target(target)),
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
        Val::Confirm { prompt, default } => {
            let p = emit_val(prompt, target)?;
            let d = if *default { "1" } else { "0" };
            Ok(format!("[ \"$( __sh2_confirm {} {} \"$@\" )\" = \"1\" ]", p, d))
        }
        Val::Contains { list, needle } => {
            if target == TargetShell::Posix {
                 return Err(CompileError::unsupported("contains() is bash-only", target));
            }
             match **list {
                 Val::Var(ref name) => {
                     let n = emit_val(needle, target)?;
                     Ok(format!("__sh2_contains \"{}\" {}", name, n))
                 }
                 _ => {
                      let n = emit_val(needle, target)?;
                      let setup = match **list {
                           Val::Lines(ref inner) => {
                               format!("__sh2_lines {} __sh2_tmp_arr", emit_val(inner, target)?)
                           }
                           Val::List(ref elems) => {
                               let mut s = String::from("__sh2_tmp_arr=(");
                               for e in elems {
                                   s.push_str(&emit_word(e, target)?);
                                   s.push(' ');
                               }
                               s.push(')');
                               s
                           }
                           Val::Split { ref s, ref delim } => {
                               format!("__sh2_split __sh2_tmp_arr {} {}", emit_val(s, target)?, emit_val(delim, target)?)
                           }
                           _ => {
                               format!("__sh2_tmp_arr=({})", emit_val(list, target)?)
                           }
                      };
                      
                      Ok(format!("( {}; __sh2_contains __sh2_tmp_arr {} )", setup, n))
                 }
             }
        }
        Val::Bool(true) => Ok("true".to_string()),
        Val::Bool(false) => Ok("false".to_string()),
        Val::List(_) | Val::Args => {
            Err(CompileError::internal("args/list is not a valid condition; use count(...) > 0", target))
        }
        Val::ContainsLine { text, needle } => {
            // ( printf '%s' <text> | grep -Fxq -- <needle> )
            Ok(format!("( printf '%s' {} | grep -Fxq -- {} )",
                emit_val(text, target)?,
                emit_val(needle, target)?
            ))
        }
        Val::Matches(text, regex) => {
            Ok(format!(
                "__sh2_matches {} {}",
                emit_val(text, target)?,
                emit_val(regex, target)?
            ))
        }
        Val::StartsWith { text, prefix } => {
            Ok(format!(
                "__sh2_starts_with {} {}",
                emit_val(text, target)?,
                emit_val(prefix, target)?
            ))
        }
        Val::BoolVar(name) => {
            // Boolean variable: check if equals "1"
            Ok(format!("[ \"${}\" = \"1\" ]", name))
        }
        // "Truthiness" fallback for scalar values: check if non-empty string.
        v => Ok(format!("[ -n {} ]", emit_val(v, target)?)),
    }
}

fn emit_index_expr(v: &Val, target: TargetShell) -> Result<String, CompileError> {
    emit_arith_expr(v, target)
}

fn emit_cmd_body_raw(args: &[Val], target: TargetShell) -> Result<String, CompileError> {
    let parts: Vec<String> = args.iter().map(|a| emit_word(a, target)).collect::<Result<_, _>>()?;
    Ok(parts.join(" "))
}

fn emit_cmdsub_raw(args: &[Val], target: TargetShell) -> Result<String, CompileError> {
    Ok(format!("$( {} )", emit_cmd_body_raw(args, target)?))
}

fn emit_cmd_pipe_body_raw(segments: &[Vec<Val>], target: TargetShell) -> Result<String, CompileError> {
    let seg_strs: Vec<String> = segments
        .iter()
        .map(|seg| emit_cmd_body_raw(seg, target))
        .collect::<Result<_, CompileError>>()?;
    Ok(seg_strs.join(" | "))
}

fn emit_cmdsub_pipe_raw(segments: &[Vec<Val>], target: TargetShell) -> Result<String, CompileError> {
    Ok(format!("$( {} )", emit_cmd_pipe_body_raw(segments, target)?))
}

fn emit_arith_expr(v: &Val, target: TargetShell) -> Result<String, CompileError> {
    match v {
        Val::Literal(s) => Ok(s.clone()),
        Val::Number(n) => Ok(n.to_string()),
        Val::Var(s) => Ok(s.clone()),
        Val::Arg(n) => Ok(format!("${}", n)),
        Val::ArgDynamic(index) => {
            let idx_str = emit_arg_index_word(index, target)?;
            // idx_str is already quoted
            // Purity: emit purely as command substitution, unquoted, for arithmetic compatibility
            Ok(format!("$( __sh2_arg_by_index {} \"$@\" )", idx_str))
        }
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



fn emit_arg_index_word(v: &Val, target: TargetShell) -> Result<String, CompileError> {
    // Both Bash and POSIX support "$i" and "$((...))"
    match v {
        Val::Var(name) => Ok(format!("\"${}\"", name)),
        Val::Number(n) => Ok(format!("\"{}\"", n)),
        Val::Literal(s) => {
             // Strictness: lowering should prevent this.
             Err(CompileError::internal(format!("String literal index should have been rejected by lowering: {:?}", s), target))
        },
        Val::ArgDynamic(_) => {
             // Recursion guard: lowering should prevent nested arg(arg(...)).
             Err(CompileError::internal("Nested dynamic argument index (recursion) should have been rejected by lowering", target))
        },
        _ => {
            // Fallback: try arithmetic emission for other valid types (Arith, Argc, etc.)
            let expr = emit_arith_expr(v, target)?;
            Ok(format!("\"$(( {} ))\"", expr))
        }
    }
}

fn emit_cmd(
    cmd: &Cmd,
    out: &mut String,
    indent: usize,
    opts: CodegenOptions,
    in_cond_ctx: bool,
    ctx: &mut CodegenContext,
) -> Result<(), CompileError> {
    let pad = " ".repeat(indent);
    let target = opts.target;

    match cmd {
        Cmd::Assign(name, val, loc) => {
            if let Some(l) = loc {
                out.push_str(&format!("{}__sh2_loc=\"{}\"\n", pad, l));
            }
            if let Val::Lines(inner) = val {
                if target == TargetShell::Posix {
                    return Err(CompileError::unsupported("lines() not supported in POSIX", target));
                }
                out.push_str(&pad);
                out.push_str(&format!("__sh2_lines {} {}\n", emit_val(inner, target)?, name));
                out.push_str(&format!("{}__sh2_status=$?\n", pad));

                out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\"\n", pad));
                return Ok(());
            }
            if let Val::Split { s, delim } = val {
                match target {
                    TargetShell::Bash => {
                         out.push_str(&pad);
                         out.push_str(&format!("__sh2_split {} {} {}\n", name, emit_val(s, target)?, emit_val(delim, target)?));
                         out.push_str(&format!("{}__sh2_status=$?\n", pad));

                         out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\"\n", pad));
                         return Ok(());
                    }
                    TargetShell::Posix => {
                         out.push_str(&pad);
                         out.push_str(&format!("{}=\"$(__sh2_tmpfile)\"\n", name));
                         out.push_str(&pad);
                         out.push_str(&format!("__sh2_split {} {} > \"${}\"\n", emit_val(s, target)?, emit_val(delim, target)?, name));
                         out.push_str(&format!("{}__sh2_status=$?\n", pad));

                         out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\"\n", pad));
                         ctx.known_lists.insert(name.to_string());
                         return Ok(());
                    }
                }
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
                out.push_str(&format!("{}__sh2_status=$?\n", pad));

                out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\"\n", pad));
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
            } else if is_boolean_val(val) {
                // Boolean assignment: emit as "1"/"0" string
                // Format: var="$( if <cond>; then printf 1; else printf 0; fi )"
                // The condition result is captured as 1/0, not via $?.
                // Boolean assignment always "succeeds" (status=0) since it's just
                // evaluating a condition and storing the result.
                out.push_str(name);
                out.push_str("=\"$( if ");
                out.push_str(&emit_cond(val, target)?);
                out.push_str("; then printf 1; else printf 0; fi )\"");
                out.push('\n');
                // Boolean assignment always succeeds - the condition result is stored
                // as 1/0, not reflected in exit status.
                out.push_str(&format!("{}__sh2_status=0\n", pad));
            } else if let Val::Capture { value, allow_fail: true } = val {
                // capture(..., allow_fail=true) logic
                // Generates:
                //   name__stdout_tmp=$(__sh2_tmpfile)
                //   name__stderr_tmp=$(__sh2_tmpfile)
                //   ( cmd ) >"$name__stdout_tmp" 2>"$name__stderr_tmp"
                //   name__status=$?
                //   name__stdout=$(__sh2_read_file "$name__stdout_tmp")
                //   name__stderr=$(__sh2_read_file "$name__stderr_tmp")
                //   name="$name__stdout"
                //   rm -f "$name__stdout_tmp" "$name__stderr_tmp"
                //   __sh2_status=0 (since allowed fail)

                // Note: We avoid 'local' to support top-level usage and POSIX sh.

                out.push_str(&format!("{}{}__stdout_tmp=$(__sh2_tmpfile)\n", pad, name));
                out.push_str(&format!("{}{}__stderr_tmp=$(__sh2_tmpfile)\n", pad, name));
                
                let cmd_str = match &**value {
                    Val::Command(args) => emit_cmd_body_raw(args, target)?,
                    Val::CommandPipe(segments) => emit_cmd_pipe_body_raw(segments, target)?,
                    _ => return Err(CompileError::internal("Capture expects Command or CommandPipe", target)),
                };

                // Use a unique status variable for this capture to avoid conflicts
                out.push_str(&format!("{}{}__cs=0\n", pad, name));
                out.push_str(&format!(
                    "{}( {} ) >\"${{{}__stdout_tmp}}\" 2>\"${{{}__stderr_tmp}}\" || {}__cs=$?\n",
                    pad, cmd_str, name, name, name
                ));
                
                out.push_str(&format!("{}{}__status=\"${{{}__cs}}\"\n", pad, name, name));
                out.push_str(&format!("{}__sh2_status=\"${{{}__cs}}\"\n", pad, name));
                // Use safe read_file helper or cat? cat is standard. read_file might have trap logic.
                 // The original code used cat. 
                 // But wait, existing code used $(cat ...).
                 // Safe read file for bash handles ERR trap.
                 // Let's use cat for simplicity as in original, or __sh2_read_file if available.
                 // __sh2_read_file is generated in prelude.
                 // Original used `cat`. Let's stick to `cat` to minimize diff risk, or upgrade to `__sh2_read_file` if safe.
                 // `emit_val` for `Val::ReadFile` uses `__sh2_read_file`.
                 // Let's use `cat` as it was verified to work.
                out.push_str(&format!("{}{}__stdout=$(cat \"${{{}__stdout_tmp}}\")\n", pad, name, name));
                out.push_str(&format!("{}{}__stderr=$(cat \"${{{}__stderr_tmp}}\")\n", pad, name, name));
                out.push_str(&format!("{}{}=\"${{{}__stdout}}\"\n", pad, name, name));
                out.push_str(&format!("{}rm -f \"${{{}__stdout_tmp}}\" \"${{{}__stderr_tmp}}\"\n", pad, name, name));
                
                // Status captured above


            } else if let Val::Args = val {
                out.push_str(name);
                out.push_str("=(\"$@\")\n");
                out.push_str(&format!("{}__sh2_status=$?\n", pad));

                if in_cond_ctx {
                    out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\" \"return\"\n", pad));
                } else {
                    out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\"\n", pad));
                }
            } else {
                out.push_str(name);
                out.push('=');
                out.push_str(&emit_val(val, target)?);
                out.push('\n');
                out.push_str(&format!("{}__sh2_status=$?\n", pad));

                if in_cond_ctx {
                    out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\" \"return\"\n", pad));
                } else {
                    out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\"\n", pad));
                }
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
                // Normal: capture status in __sh2_status, then check for failure
                out.push_str(&shell_cmd);
                out.push_str("; __sh2_status=$?
");
                if in_cond_ctx {
                    // In condition context (e.g. try block), we must NOT exit the script.
                    // We use (exit $s) to set $? and trigger errexit if active (which catch handles).
                    out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\" \"return\"
", pad));
                } else {
                    // Normal context: use __sh2_check to fail-fast with diagnostics
                    out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\" \"exit\"
", pad));
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
                emit_cmd(c, out, indent + 2, opts, in_cond_ctx, ctx)?;
            }

            for (cond, body) in elifs {
                let cond_str = emit_cond(cond, target)?;
                out.push_str(&format!("{pad}elif {cond_str}; then\n"));
                for c in body {
                    emit_cmd(c, out, indent + 2, opts, in_cond_ctx, ctx)?;
                }
            }

            if !else_body.is_empty() {
                out.push_str(&format!("{pad}else\n"));
                for c in else_body {
                    emit_cmd(c, out, indent + 2, opts, in_cond_ctx, ctx)?;
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
                            emit_cmd(cmd, &mut pipe_str, indent + 2, opts, false, ctx)?;
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
                                emit_cmd(cmd, &mut s, indent + 2, opts, false, ctx)?;
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
                    emit_cmd(cmd, out, indent + 4, opts, in_cond_ctx, ctx)?;
                }
                out.push_str(&format!("{}  ;;\n", pad));
            }
            out.push_str(&format!("{}esac\n", pad));
        }

        Cmd::While { cond, body } => {
            let cond_str = emit_cond(cond, target)?;
            out.push_str(&format!("{pad}while {cond_str}; do\n"));
            for c in body {
                emit_cmd(c, out, indent + 2, opts, in_cond_ctx, ctx)?;
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
            out.push_str(&format!("{}__sh2_status=$?\n", pad));

            out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\"\n", pad));
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

        Cmd::For { var, items, body } => {
            // Check if we need POSIX list iteration (file-based)
            let is_posix_list_mode = target == TargetShell::Posix && items.iter().any(|i|
                matches!(i, Val::Split { .. } | Val::Lines(_)) ||
                (if let Val::Var(n) = i { ctx.known_lists.contains(n) } else { false })
            );

            if is_posix_list_mode {
                 // POSIX List Iteration: Generate stream to temp file, then while-read it to preserve body semantics
                 out.push_str(&pad);
                 out.push_str(&format!("__sh2_for_tmp_{}=$(__sh2_tmpfile)\n", var));
                 out.push_str(&pad);
                 out.push_str("{\n");
                 for item in items {
                     match item {
                         Val::Split { s, delim } => {
                             out.push_str(&format!("{}  __sh2_split {} {}\n", pad, emit_val(s, target)?, emit_val(delim, target)?));
                         }
                         Val::Lines(_inner) => {
                             return Err(CompileError::unsupported("lines() iteration not supported in POSIX", target));
                         }
                         Val::Var(n) if ctx.known_lists.contains(n) => {
                             out.push_str(&format!("{}  cat \"${}\"\n", pad, n));
                         }
                         _ => {
                             // Treat other items (literals, unknown string vars) as single lines
                             out.push_str(&format!("{}  echo {}\n", pad, emit_word(item, target)?));
                         }
                     }
                 }
                 out.push_str(&format!("{}}} > \"$__sh2_for_tmp_{}\"\n", pad, var));

                 out.push_str(&format!("{}while IFS= read -r {} || [ -n \"${}\" ]; do\n", pad, var, var));
                 for c in body {
                     emit_cmd(c, out, indent + 2, opts, in_cond_ctx, ctx)?;
                 }
                 out.push_str(&format!("{}done < \"$__sh2_for_tmp_{}\"\n", pad, var));
                 out.push_str(&format!("{}rm -f \"$__sh2_for_tmp_{}\"\n", pad, var)); // Early cleanup
                 return Ok(());
            }

            // Pre-process any Lines() / Split items for Bash (Arrays)
            for (idx, item) in items.iter().enumerate() {
                if let Val::Lines(inner) = item {
                    if target == TargetShell::Posix {
                        return Err(CompileError::unsupported("lines() iteration not supported in POSIX", target));
                    }
                    out.push_str(&pad);
                    out.push_str(&format!("__sh2_lines {} __sh2_for_lines_{}\n", emit_val(inner, target)?, idx));
                }
                if let Val::Split { s, delim } = item {
                    if target == TargetShell::Bash {
                        out.push_str(&pad);
                        out.push_str(&format!("__sh2_split __sh2_for_split_{} {} {}\n", idx, emit_val(s, target)?, emit_val(delim, target)?));
                    }
                }
            }

            out.push_str(&format!("{}for {} in", pad, var));
            for (idx, item) in items.iter().enumerate() {
                match item {
                    Val::Lines(_) => {
                         out.push_str(&format!(" \"${{__sh2_for_lines_{}[@]}}\"", idx));
                    }
                    Val::Split { .. } => {
                        // Bash array pre-calc handled above
                        out.push_str(&format!(" \"${{__sh2_for_split_{}[@]}}\"", idx));
                    }
                    Val::List(elems) => {
                        for elem in elems {
                            out.push(' ');
                            out.push_str(&emit_word(elem, target)?);
                        }
                    }
                    Val::Var(name) => {
                        if target == TargetShell::Posix {
                             return Err(CompileError::unsupported("Iterating over array variable not supported in POSIX", target));
                        }
                        out.push_str(&format!(" \"${{{}[@]}}\"", name));
                    }
                    _ => {
                        out.push(' ');
                        out.push_str(&emit_word(item, target)?);
                    }
                }
            }
            out.push_str("; do\n");
            for c in body {
                emit_cmd(c, out, indent + 2, opts, in_cond_ctx, ctx)?;
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
                emit_cmd(c, out, indent + 2, opts, in_cond_ctx, ctx)?;
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
                        "{pad}if {}; then printf '%s' 1; fi\n",
                        cond_str
                    ));
                    out.push_str(&format!("{pad}return 0\n"));
                } else {
                    out.push_str(&format!("{pad}printf '%s' {}\n", emit_val(v, target)?));
                    out.push_str(&format!("{pad}return 0\n"));
                }
            } else {
                out.push_str(&format!("{pad}return 0\n"));
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
        Cmd::WriteFile { path, content, append } => {
            let op = if *append { ">>" } else { ">" };
            out.push_str(&pad);
            out.push_str(&format!(
                "printf '%s' {} {} {}",
                emit_val(content, target)?,
                op,
                emit_val(path, target)?
            ));
            out.push('\n');
            out.push_str(&format!("{}__sh2_status=$?\n", pad));

            if in_cond_ctx {
                out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\" \"return\"\n", pad));
            } else {
                out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\"\n", pad));
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
                emit_cmd(cmd, out, indent + 2, opts, in_cond_ctx, ctx)?;
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
                emit_cmd(cmd, out, indent + 2, opts, in_cond_ctx, ctx)?;
            }
            out.push_str(&format!("{pad})\n"));
        }
        Cmd::WithCwd { path, body } => {
            out.push_str(&format!("{pad}(\n"));
            let path_str = emit_val(path, target)?;
            
            // Ticket 9/11: Runtime hint for tilde literal path.
            // ONLY emit if the path is a literal starting with '~'.
            let is_tilde_literal = if let Val::Literal(s) = path {
                 s.starts_with('~') 
            } else { 
                 false 
            };

            if is_tilde_literal {
                // Wrap cd in failure check with hint (use exit as this is a subshell)
                out.push_str(&format!("{pad}  cd {} || {{ __sh2_err=$?; printf '%s\\n' \"hint: '~' is not expanded; use env.HOME & \\\"/path\\\" or an absolute path.\" >&2; exit $__sh2_err; }}\n", path_str));
            } else {
                out.push_str(&format!("{pad}  cd {}\n", path_str));
            }

            for cmd in body {
                emit_cmd(cmd, out, indent + 2, opts, in_cond_ctx, ctx)?;
            }
            out.push_str(&format!("{pad})\n"));
        }
        Cmd::Cd(path) => {
            out.push_str(&pad);
            out.push_str("cd ");
            out.push_str(&emit_val(path, target)?);
            out.push('\n');
        }
        Cmd::RawLine { line, loc } => {
            if let Some(l) = loc {
                if !in_cond_ctx {
                     out.push_str(&format!("{}__sh2_loc=\"{}\"\n", pad, l));
                }
            }
            out.push_str(&pad);
            out.push_str(line);
            out.push('\n');
            out.push_str(&format!("{}__sh2_status=$?\n", pad));

            if in_cond_ctx {
                out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\" \"return\"\n", pad));
            } else {
                out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\" \"exit\"\n", pad));
            }
        }
        Cmd::Raw(val, loc) => {
             // sh(expr) -> execute expression as a shell command in a subshell (bash -c or sh -c)
             // This is a probe, so it sets __sh2_status but does not fail-fast.
             
             if let Some(l) = loc {
                 out.push_str(&format!("{}__sh2_loc=\"{}\"\n", pad, l));
             }
             match val {
                 Val::Literal(s) => {
                     let cmd_escaped = sh_single_quote(s);
                     out.push_str(&format!("{}__sh2_sh_probe {}\n", pad, cmd_escaped));
                 }
                 _ => {
                     out.push_str(&format!("{}__sh2_cmd=", pad));
                     out.push_str(&emit_val(val, target)?);
                     out.push('\n');
                     out.push_str(&format!("{}__sh2_sh_probe \"$__sh2_cmd\"\n", pad));
                 }
             }
        }
        Cmd::Call { name, args } => {
            out.push_str(&pad);
            out.push_str(name);
            for arg in args {
                out.push(' ');
                out.push_str(&emit_word(arg, target)?);
            }
            out.push_str("; __sh2_status=$?\n");

            if in_cond_ctx {
                out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\" \"return\"\n", pad));
            } else {
                out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\"\n", pad));
            }
        }
        Cmd::Subshell { body } => {
            out.push_str(&format!("{pad}(\n"));
            for cmd in body {
                emit_cmd(cmd, out, indent + 2, opts, in_cond_ctx, ctx)?;
            }
            out.push_str(&format!("{pad})\n"));
        }
        Cmd::Group { body } => {
            out.push_str(&format!("{pad}{{\n"));
            for cmd in body {
                emit_cmd(cmd, out, indent + 2, opts, in_cond_ctx, ctx)?;
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        Cmd::WithRedirect {
            stdout,
            stderr,
            stdin,
            body,
        } => {
            // TICKET 6: Multi-Sink Redirect Logic
            
            // Analyze stdout targets
            let mut stdout_files = Vec::new();
            let mut stdout_inherit = false;
            let mut stdout_cross = None;
            if let Some(ts) = stdout.as_ref() {
                for t in ts {
                    match t {
                        RedirectOutputTarget::File { path, append } => stdout_files.push((path, *append)),
                        RedirectOutputTarget::InheritStdout => stdout_inherit = true,
                        t @ RedirectOutputTarget::ToStdout | t @ RedirectOutputTarget::ToStderr => stdout_cross = Some(t),
                        _ => {}, // InheritStderr and other cases are ignored for stdout
                    }
                }
            }

            // Analyze stderr targets
            let mut stderr_files = Vec::new();
            let mut stderr_inherit = false;
            // stderr cross ignored in lists by parser
            if let Some(ts) = stderr.as_ref() {
                for t in ts {
                    match t {
                         RedirectOutputTarget::File { path, append } => stderr_files.push((path, *append)),
                         RedirectOutputTarget::InheritStderr => stderr_inherit = true,
                         _ => {},
                    }
                }
            }

            // Mixed-append validation: reject lists with both append=true and append=false
            if stdout_files.len() > 1 {
                let has_append = stdout_files.iter().any(|(_, append)| *append);
                let has_non_append = stdout_files.iter().any(|(_, append)| !*append);
                if has_append && has_non_append {
                    return Err(CompileError::unsupported(
                        "redirect list cannot mix append and overwrite modes; all file() targets must use the same append setting",
                        target,
                    ));
                }
            }
            if stderr_files.len() > 1 {
                let has_append = stderr_files.iter().any(|(_, append)| *append);
                let has_non_append = stderr_files.iter().any(|(_, append)| !*append);
                if has_append && has_non_append {
                    return Err(CompileError::unsupported(
                        "redirect list cannot mix append and overwrite modes; all file() targets must use the same append setting",
                        target,
                    ));
                }
            }


            // "Tee needed" if:
            // 1. More than 1 file target
            // 2. 1 file target AND inherit (fan-out to file + terminal)
            // Note: cross-stream checks are handled by parser/simple logic usually,
            // but parser rejects cross-stream in list. So if we have cross-stream, it's a single target.
            
            let tee_stdout = stdout_files.len() > 1 || (stdout_files.len() == 1 && stdout_inherit);
            let tee_stderr = stderr_files.len() > 1 || (stderr_files.len() == 1 && stderr_inherit);

            // POSIX Check: Reject if tee is needed
            // Also reject if list > 1 but parser allowed it (e.g. cross-stream which shouldn't happen in list, but checking len is safe)
            if target == TargetShell::Posix {
                if tee_stdout || (stdout.as_ref().map_or(false, |v| v.len() > 1)) {
                     return Err(CompileError::unsupported(
                        "multi-sink redirect is not supported for POSIX target; use a single redirect target or switch to --target bash",
                        target,
                    ));
                }
                if tee_stderr || (stderr.as_ref().map_or(false, |v| v.len() > 1)) {
                     return Err(CompileError::unsupported(
                        "multi-sink redirect is not supported for POSIX target; use a single redirect target or switch to --target bash",
                        target,
                    ));
                }
            }

            if tee_stdout || tee_stderr {
                // Bash Multi-Sink Implementation
                // We wrap the body in an outer block that sets up FDs/tees
                if target != TargetShell::Bash {
                     return Err(CompileError::unsupported("multi-sink redirect requires bash", target));
                }

                // Generate unique ID for this redirect block
                let uid = ctx.next_id();

                out.push_str(&format!("{pad}{{\n")); // Wrapper block start

                // 1. Setup FIFO + Tee Processes
                let mut stdout_fifo_opt = None;
                let mut stdout_pid_opt = None;
                let mut stderr_fifo_opt = None;
                let mut stderr_pid_opt = None;
                
                // Declare FIFO variables first (needed for consolidated trap)
                if tee_stdout {
                    let fifo_var = format!("__sh2_fifo_out_{}", uid);
                    stdout_fifo_opt = Some(fifo_var.clone());
                    out.push_str(&format!("{pad}  {}=\"/tmp/.${{USER:-user}}.sh2.fifo.out.{}.$$\"\n", fifo_var, uid));
                }
                if tee_stderr {
                    let fifo_var = format!("__sh2_fifo_err_{}", uid);
                    stderr_fifo_opt = Some(fifo_var.clone());
                    out.push_str(&format!("{pad}  {}=\"/tmp/.${{USER:-user}}.sh2.fifo.err.{}.$$\"\n", fifo_var, uid));
                }
                
                // Install single cleanup trap for both FIFOs (if any)
                if tee_stdout || tee_stderr {
                    let mut trap_cmd = String::from("rm -f");
                    if let Some(ref fifo) = stdout_fifo_opt {
                        trap_cmd.push_str(&format!(" \"${}\"", fifo));
                    }
                    if let Some(ref fifo) = stderr_fifo_opt {
                        trap_cmd.push_str(&format!(" \"${}\"", fifo));
                    }
                    out.push_str(&format!("{pad}  trap '{}' RETURN\n", trap_cmd));
                }
                
                // Create FIFOs and start tee processes
                if tee_stdout {
                    let fifo_var = stdout_fifo_opt.as_ref().unwrap();
                    let pid_var = format!("__sh2_pid_out_{}", uid);
                    stdout_pid_opt = Some(pid_var.clone());

                    // Create FIFO
                    out.push_str(&format!("{pad}  mkfifo \"${}\" || {{ __sh2_status=1; return 1; }}\n", fifo_var));
                    
                    // Start tee in background reading from FIFO
                    // Determine append mode once (all files must share same mode)
                    let append_mode = stdout_files.first().map(|(_, a)| *a).unwrap_or(false);
                    out.push_str(&format!("{pad}  ( tee"));
                    if append_mode {
                        out.push_str(" -a");
                    }
                    for (path, _append) in &stdout_files {
                        out.push_str(&format!(" {}", emit_val(path, target)?));
                    }
                    if stdout_inherit {
                         // Inherit means keep printing to current stdout (default)
                    } else {
                         // No inherit: suppress tee's stdout
                         out.push_str(" >/dev/null");
                    }
                    out.push_str(&format!(" < \"${}\" ) &\n", fifo_var));
                    out.push_str(&format!("{pad}  {}=$!\n", pid_var));
                }

                if tee_stderr {
                    let fifo_var = stderr_fifo_opt.as_ref().unwrap();
                    let pid_var = format!("__sh2_pid_err_{}", uid);
                    stderr_pid_opt = Some(pid_var.clone());

                    // Create FIFO
                    out.push_str(&format!("{pad}  mkfifo \"${}\" || {{ __sh2_status=1; return 1; }}\n", fifo_var));
                     
                    // Start tee in background reading from FIFO
                    // Determine append mode once (all files must share same mode)
                    let append_mode = stderr_files.first().map(|(_, a)| *a).unwrap_or(false);
                    out.push_str(&format!("{pad}  ( tee"));
                    if append_mode {
                        out.push_str(" -a");
                    }
                    for (path, _append) in &stderr_files {
                         out.push_str(&format!(" {}", emit_val(path, target)?));
                    }
                    if stderr_inherit {
                         out.push_str(" >&2"); // Write to current stderr
                    } else {
                         out.push_str(" >/dev/null");
                    }
                    out.push_str(&format!(" < \"${}\" ) &\n", fifo_var));
                    out.push_str(&format!("{pad}  {}=$!\n", pid_var));
                }

                // 2. Emit Inner Body
                out.push_str(&format!("{pad}  {{\n"));
                for cmd in body {
                    emit_cmd(cmd, out, indent + 4, opts, in_cond_ctx, ctx)?;
                }
            out.push_str(&format!("{pad}  }}")); // Close inner body

                // 3. Apply Redirects to Inner Body
                
                // Stdout application
                if let Some(ref fifo) = stdout_fifo_opt {
                    out.push_str(&format!(" >\"${}\"", fifo));
                } else if let Some(first) = stdout_files.first() {
                    let (path, append) = first;
                    let op = if *append { ">>" } else { ">" };
                    out.push_str(&format!(" {} {}", op, emit_val(path, target)?));
                } else if let Some(cross) = stdout_cross {
                     match cross {
                         RedirectOutputTarget::ToStderr => out.push_str(" 1>&2"),
                         _ => {}
                     }
                }

                // Stderr application
                if let Some(ref fifo) = stderr_fifo_opt {
                    out.push_str(&format!(" 2>\"${}\"", fifo));
                } else {
                    if let Some((path, append)) = stderr_files.first() {
                        let op = if *append { ">>" } else { ">" };
                        out.push_str(&format!(" 2{} {}", op, emit_val(path, target)?));
                    } else if let Some(types) = stderr.as_ref().and_then(|v| v.first()) {
                         if matches!(types, RedirectOutputTarget::ToStdout) {
                             out.push_str(" 2>&1");
                         }
                    }
                }

                // Stdin application + Heredoc content preparation
                let mut heredoc_to_emit = None;
                if let Some(target_redir) = stdin {
                    match target_redir {
                        RedirectInputTarget::File { path } => {
                            out.push_str(&format!(" < {}", emit_val(path, target)?));
                        }
                        RedirectInputTarget::HereDoc { content } => {
                           // Use unique delimiter with collision avoidance
                            let mut delim = format!("__SH2_EOF_{}__", uid);
                            let mut counter = 1;
                            while content.contains(&delim) {
                                delim = format!("__SH2_EOF_{}_{}__", uid, counter);
                                counter += 1;
                            }
                            out.push_str(&format!(" <<'{}'", delim));
                            heredoc_to_emit = Some((content, delim));
                        }
                    }
                }
                
                // End redirect line
                out.push_str("\n");

                // Emit Heredoc content inside the wrapper
                if let Some((content, delim)) = heredoc_to_emit {
                    out.push_str(content);
                    if !content.ends_with('\n') { out.push('\n'); }
                    out.push_str(&format!("{}\n", delim));
                }
                
                // 4. Capture Status & Cleanup (Wait for tee completion)
                let cmd_status_var = format!("__sh2_cs_{}", uid);
                out.push_str(&format!("{pad}  {}=$?\n", cmd_status_var));
                
                // Wait for tee processes to complete and capture their statuses
                let mut has_stdout_tee = false;
                let mut has_stderr_tee = false;
                
                if let Some(_fifo) = &stdout_fifo_opt {
                     out.push_str(&format!("{pad}  wait \"${}\"\n", stdout_pid_opt.as_ref().unwrap())); 
                     let tee_status_var = format!("__sh2_ts_out_{}", uid);
                     out.push_str(&format!("{pad}  {}=$?\n", tee_status_var));
                     has_stdout_tee = true;
                }
                if let Some(_fifo) = &stderr_fifo_opt {
                     out.push_str(&format!("{pad}  wait \"${}\"\n", stderr_pid_opt.as_ref().unwrap()));
                     let tee_status_var = format!("__sh2_ts_err_{}", uid);
                     out.push_str(&format!("{pad}  {}=$?\n", tee_status_var));
                     has_stderr_tee = true;
                }

                // Compute final status with deterministic precedence: cmd < stdout_tee < stderr_tee
                let final_status_var = format!("__sh2_final_{}", uid);
                out.push_str(&format!("{pad}  {}=${{{}:-0}}\n", final_status_var, cmd_status_var));
                if has_stdout_tee {
                    let tee_status_var = format!("__sh2_ts_out_{}", uid);
                    out.push_str(&format!("{pad}  if [ \"${{{}:-0}}\" -ne 0 ]; then {}=${}; fi\n", tee_status_var, final_status_var, tee_status_var));
                }
                if has_stderr_tee {
                    let tee_status_var = format!("__sh2_ts_err_{}", uid);
                    out.push_str(&format!("{pad}  if [ \"${{{}:-0}}\" -ne 0 ]; then {}=${}; fi\n", tee_status_var, final_status_var, tee_status_var));
                }

                // Propagate to global status variable
                out.push_str(&format!("{pad}  __sh2_status=${}\n", final_status_var));
                out.push_str(&format!("{pad}}}"));// Close wrapper block
                out.push('\n');

            } else {
                 // Simple / Single Target Case (POSIX compatible usually, or Bash single)
                 // Existing logic preserved for compatibility and simplicity where tee is not needed.

                out.push_str(&format!("{pad}{{\n"));
                for cmd in body {
                    emit_cmd(cmd, out, indent + 2, opts, in_cond_ctx, ctx)?;
                }
                out.push_str(&format!("{pad}}}")); 

                // Handle stdin redirection
                let mut heredoc_content = None;

                if let Some(target_redir) = stdin {
                    match target_redir {
                        RedirectInputTarget::File { path } => {
                            out.push_str(&format!(" < {}", emit_val(path, target)?));
                        }
                        RedirectInputTarget::HereDoc { content } => {
                            heredoc_content = Some(content);
                            // Standard heredoc marker
                            // Use safe delimiter check
                            let mut delim = "__SH2_EOF__".to_string();
                            let mut counter = 1;
                            while content.contains(&delim) {
                                delim = format!("__SH2_EOF__{}__", counter);
                                counter += 1;
                            }
                            out.push_str(&format!(" <<'{}'", delim));
                        }
                    }
                }

                // Single target handling
                let stdout_single = stdout.as_ref().and_then(|v| v.first());
                let stderr_single = stderr.as_ref().and_then(|v| v.first());

                // Emission order logic (same as legacy)
                let mut emit_stdout_first = true;
                if let Some(stdout_target) = stdout_single {
                   if let Some(stderr_target) = stderr_single {
                       if matches!(stdout_target, RedirectOutputTarget::ToStderr)
                           && matches!(stderr_target, RedirectOutputTarget::File { .. }) {
                           emit_stdout_first = false;
                       }
                        if matches!(stdout_target, RedirectOutputTarget::ToStderr)
                           && matches!(stderr_target, RedirectOutputTarget::ToStdout) {
                            return Err(CompileError::unsupported(
                                "Cyclic redirection: stdout to stderr AND stderr to stdout is not supported"
                            , target));
                       }
                   }
                }
                
                let emit_stdout = |out: &mut String, target_redir: &RedirectOutputTarget| -> Result<(), CompileError> {
                        match target_redir {
                            RedirectOutputTarget::File { path, append } => {
                                let op = if *append { ">>" } else { ">" };
                                out.push_str(&format!(" {} {}", op, emit_val(path, target)?));
                            }
                            RedirectOutputTarget::ToStderr => out.push_str(" 1>&2"),
                            RedirectOutputTarget::ToStdout => {}, 
                            RedirectOutputTarget::InheritStdout | RedirectOutputTarget::InheritStderr => {}
                        }
                        Ok(())
                };
                let emit_stderr = |out: &mut String, target_redir: &RedirectOutputTarget| -> Result<(), CompileError> {
                        match target_redir {
                            RedirectOutputTarget::File { path, append } => {
                                let op = if *append { ">>" } else { ">" };
                                out.push_str(&format!(" 2{} {}", op, emit_val(path, target)?));
                            }
                            RedirectOutputTarget::ToStdout => out.push_str(" 2>&1"),
                            RedirectOutputTarget::ToStderr => {},
                            RedirectOutputTarget::InheritStdout | RedirectOutputTarget::InheritStderr => {} 
                        }
                        Ok(())
                };

                if emit_stdout_first {
                    if let Some(t) = stdout_single { emit_stdout(out, t)?; }
                    if let Some(t) = stderr_single { emit_stderr(out, t)?; }
                } else {
                    if let Some(t) = stderr_single { emit_stderr(out, t)?; }
                    if let Some(t) = stdout_single { emit_stdout(out, t)?; }
                }

                // Heredoc body
                if let Some(content) = heredoc_content {
                    let mut delim = "__SH2_EOF__".to_string();
                    let mut counter = 1;
                    while content.contains(&delim) {
                        delim = format!("__SH2_EOF__{}__", counter);
                        counter += 1;
                    }
                    out.push('\n');
                    out.push_str(content);
                    if !content.ends_with('\n') { out.push('\n'); }
                    out.push_str(&delim);
                }
                out.push('\n');
            }



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
                    emit_cmd(cmd, out, indent + 2, opts, false, ctx)?;
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
                        out.push_str("; __sh2_status=$?\n");

                        if in_cond_ctx {
                            out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\" \"return\"\n", pad));
                        } else {
                            out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\"\n", pad));
                        }
                    }
                    _ => {
                        out.push_str(&format!(
                            "{pad}wait {}; __sh2_status=$?\n",
                            emit_word(val, target)?
                        ));
                        if in_cond_ctx {
                            out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\" \"return\"\n", pad));
                        } else {
                            out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\"\n", pad));
                        }
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
                    emit_cmd(cmd, out, indent + 4, opts, true, ctx)?;
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
                emit_cmd(cmd, out, indent + 2, opts, in_cond_ctx, ctx)?;
            }
            out.push_str(&format!("{pad}fi\n"));
        }
        Cmd::AndThen { left, right } => {
            out.push_str(&format!("{pad}{{\n"));
            for cmd in left {
                emit_cmd(cmd, out, indent + 2, opts, true, ctx)?;
            }
            out.push_str(&format!("{pad}}} && {{\n"));
            for cmd in right {
                emit_cmd(cmd, out, indent + 2, opts, true, ctx)?;
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        Cmd::OrElse { left, right } => {
            out.push_str(&format!("{pad}{{\n"));
            for cmd in left {
                emit_cmd(cmd, out, indent + 2, opts, true, ctx)?;
            }
            out.push_str(&format!("{pad}}} || {{\n"));
            for cmd in right {
                emit_cmd(cmd, out, indent + 2, opts, true, ctx)?;
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
            out.push_str("; __sh2_status=$?\n");

            if in_cond_ctx {
                out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\" \"return\"\n", pad));
            } else {
                out.push_str(&format!("{}__sh2_check \"$__sh2_status\" \"${{__sh2_loc:-}}\"\n", pad));
            }
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

    // Always emit __sh2_check for fail-fast behavior
    match target {
        TargetShell::Bash => {
            s.push_str("__sh2_check() { local s=\"$1\"; local loc=\"$2\"; local mode=\"$3\"; if (( s != 0 )); then if [[ \"$mode\" == \"return\" ]]; then return \"$s\"; else if [[ -n \"$loc\" ]]; then printf 'Error in %s\\n' \"$loc\" >&2; fi; exit \"$s\"; fi; fi; }\n");
        }
        TargetShell::Posix => {
            s.push_str("__sh2_check() { __sh2_s=\"$1\"; __sh2_l=\"$2\"; __sh2_m=\"$3\"; if [ \"$__sh2_s\" -ne 0 ]; then if [ \"$__sh2_m\" = \"return\" ]; then return \"$__sh2_s\"; fi; if [ -n \"$__sh2_l\" ]; then printf 'Error in %s\\n' \"$__sh2_l\" >&2; fi; exit \"$__sh2_s\"; fi; }\n");
        }
    }


    if usage.sh_probe {
        match target {
            TargetShell::Bash => {
                s.push_str("__sh2_sh_probe() { local cmd=\"$1\"; if bash -c \"$cmd\"; then __sh2_status=0; else __sh2_status=$?; fi; return 0; }\n");
            }
            TargetShell::Posix => {
                s.push_str("__sh2_sh_probe() { cmd=\"$1\"; if sh -c \"$cmd\"; then __sh2_status=0; else __sh2_status=$?; fi; return 0; }\n");
            }
        }
    }

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
        match target {
            TargetShell::Bash => {
                s.push_str(r#"__sh2_split() {
  local -n __o=$1
  if [[ -z "$3" ]]; then eval "$1=(\"$2\")"; return; fi
  mapfile -t __o < <(awk -v s="$2" -v sep="$3" 'BEGIN {
     len=length(sep);
     while(i=index(s, sep)) { print substr(s, 1, i-1); s=substr(s, i+len) }
     print s
  }')
}
"#);
            }
            TargetShell::Posix => {
                s.push_str(r#"__sh2_tmpfiles=""
__sh2_tmpfile() {
    t=$(mktemp) || exit 1
    __sh2_tmpfiles="$__sh2_tmpfiles $t"
    echo "$t"
}
__sh2_cleanup_tmpfiles() {
    for t in $__sh2_tmpfiles; do rm -f "$t"; done
}
trap __sh2_cleanup_tmpfiles EXIT
__sh2_split() {
  awk -v s="$1" -v sep="$2" 'BEGIN {
     if(sep=="") { print s; exit }
     len=length(sep);
     while(i=index(s, sep)) { print substr(s, 1, i-1); s=substr(s, i+len) }
     print s
  }'
}
"#);
            }
        }
    }

    match target {
        TargetShell::Bash => {
            if usage.loc {
                s.push_str(r#"__sh2_err_handler() {
  local s=$?
  local loc="${__sh2_loc:-}"
  if [[ "${BASH_COMMAND}" == *"(exit "* ]]; then return $s; fi
  if [[ -z "$loc" ]]; then return $s; fi
  if [[ "$loc" == "${__sh2_last_err_loc:-}" && "$s" == "${__sh2_last_err_status:-}" ]]; then return $s; fi
  __sh2_last_err_loc="$loc"
  __sh2_last_err_status="$s"
  printf "Error in %s\n" "$loc" >&2
  return $s
}
"#);

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
            r#"__sh2_read_file() { cat "$1"; }
"#,
        );
    }
    if usage.write_file {
        s.push_str(r#"__sh2_write_file() { if [ "$3" = "true" ]; then printf '%s' "$2" >> "$1"; else printf '%s' "$2" > "$1"; fi; }
"#);
    }
    if usage.lines {
        match target {
            TargetShell::Bash => {
                s.push_str(r#"__sh2_lines() { mapfile -t "$2" <<< "$1"; if [[ -z "$1" ]]; then eval "$2=()"; elif [[ "$1" == *$'\n' ]]; then eval "unset '$2[\${#$2[@]}-1]'"; fi; }
"#);
            }
            TargetShell::Posix => {
                // Not supported in POSIX sh
            }
        }
    }

    if usage.arg_dynamic {
        match target {
            TargetShell::Bash => {
                s.push_str("__sh2_arg_by_index() { local __sh2_idx=\"$1\"; shift; ");
                s.push_str("case \"$__sh2_idx\" in (''|*[!0-9]*) printf ''; return 0;; esac; ");
                s.push_str("[ \"$__sh2_idx\" -ge 1 ] 2>/dev/null || { printf ''; return 0; }; ");
                s.push_str("[ \"$__sh2_idx\" -le \"$#\" ] 2>/dev/null || { printf ''; return 0; }; ");
                s.push_str("eval \"printf '%s' \\\"\\${$__sh2_idx}\\\"\"; ");
                s.push_str("}\n");
            }
            TargetShell::Posix => {
                s.push_str("__sh2_arg_by_index() { __sh2_idx=\"$1\"; shift; ");
                s.push_str("case \"$__sh2_idx\" in (''|*[!0-9]*) printf ''; return 0;; esac; ");
                s.push_str("[ \"$__sh2_idx\" -ge 1 ] 2>/dev/null || { printf ''; return 0; }; ");
                s.push_str("[ \"$__sh2_idx\" -le \"$#\" ] 2>/dev/null || { printf ''; return 0; }; ");
                s.push_str("eval \"printf '%s' \\\"\\${$__sh2_idx}\\\"\"; ");
                s.push_str("}\n");
            }
        }
    }

    if usage.contains {
        if target == TargetShell::Bash {
            s.push_str(r#"__sh2_contains() { local -n __arr=$1; local __val=$2; for __e in "${__arr[@]}"; do if [[ "$__e" == "$__val" ]]; then return 0; fi; done; return 1; }
"#);
        }
    }
    if usage.starts_with {
         match target {
            TargetShell::Bash => {
                s.push_str(r#"__sh2_starts_with() { [[ "$1" == "$2"* ]]; return $?; }
"#);
            }
             TargetShell::Posix => {
                 s.push_str(r#"__sh2_starts_with() { case "$1" in "$2"*) return 0;; *) return 1;; esac; }
"#);
             }
         }
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

    if usage.confirm {
        // __sh2_confirm prompt default "$@"
        // Prints "1" or "0" to stdout. Prompt goes to stderr.
        // Precedence: SH2_NO/SH2_YES > --no/--yes > CI/non-tty default > interactive prompt
        s.push_str(r#"__sh2_confirm() { __sh2_prompt="$1"; __sh2_default="$2"; shift 2; "#);
        // Env overrides (highest precedence)
        s.push_str(r#"if [ "${SH2_NO:-}" = "1" ]; then printf '%s' '0'; return 0; fi; "#);
        s.push_str(r#"if [ "${SH2_YES:-}" = "1" ]; then printf '%s' '1'; return 0; fi; "#);
        // Arg overrides
        s.push_str(r#"for __a in "$@"; do case "$__a" in --yes) printf '%s' '1'; return 0;; --no) printf '%s' '0'; return 0;; esac; done; "#);
        // Non-interactive: CI=true or stdin not a TTY
        s.push_str(r#"if [ "${CI:-}" = "true" ] || ! [ -t 0 ]; then printf '%s' "$__sh2_default"; return 0; fi; "#);
        // Interactive prompt loop
        s.push_str(r#"while true; do "#);
        s.push_str(r#"if [ "$__sh2_default" = "1" ]; then printf '%s [Y/n] ' "$__sh2_prompt" >&2; else printf '%s [y/N] ' "$__sh2_prompt" >&2; fi; "#);
        s.push_str(r#"if ! IFS= read -r __sh2_ans; then printf '%s' "$__sh2_default"; return 0; fi; "#);
        s.push_str(r#"__sh2_ans_lc="$(printf '%s' "$__sh2_ans" | tr '[:upper:]' '[:lower:]')"; "#);
        s.push_str(r#"case "$__sh2_ans_lc" in y|yes) printf '%s' '1'; return 0;; n|no) printf '%s' '0'; return 0;; '') printf '%s' "$__sh2_default"; return 0;; esac; "#);
        s.push_str("done; }\n");
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
