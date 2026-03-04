pub mod posix_lint;
pub use posix_lint::{PosixLint, PosixLintKind, lint_script, render_lints};

pub(crate) mod helpers;

use crate::error::CompileError;
use crate::ir::{Cmd, Function, RedirectOutputTarget, RedirectInputTarget, Val};
pub use crate::target::TargetShell;

mod emit_prelude;
use self::emit_prelude::emit_prelude;

mod emit_val;

mod emit_cmd;
use self::emit_cmd::emit_cmd;

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
    pub find_files: bool,
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
    pub sh_probe_args: bool,
    pub confirm: bool,
    pub glob: bool,
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
        Cmd::PipeEachLine { producer, body, .. } => {
            usage.tmpfile = true;
            visit_cmd(producer, usage, include_diagnostics);
            for c in body {
                visit_cmd(c, usage, include_diagnostics);
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
        Cmd::For { iterable, body, .. } => {
            match iterable {
                crate::ir::ForIterable::List(items) => {
                     for i in items {
                         visit_val(i, usage);
                     }
                }
                crate::ir::ForIterable::Range(start, end) => {
                    visit_val(start, usage);
                    visit_val(end, usage);
                }
                crate::ir::ForIterable::StdinLines => {}
                crate::ir::ForIterable::Find0 { dir, name, type_filter, maxdepth } => {
                    visit_val(dir, usage);
                    if let Some(n) = name { visit_val(n, usage); }
                    if let Some(t) = type_filter { visit_val(t, usage); }
                    if let Some(m) = maxdepth { visit_val(m, usage); }
                }
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
        Cmd::Raw { cmd: val, args, loc } => {
             if let Some(_) = args {
                 usage.sh_probe_args = true;
             } else {
                 usage.sh_probe = true;
             }
             if include_diagnostics && loc.is_some() {
                 usage.loc = true;
             }
             visit_val(val, usage);
             if let Some(a) = args {
                 visit_val(a, usage);
             }
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
        Val::Glob(inner) => {
            usage.glob = true;
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
        Val::ContainsList { list, needle } => {
            usage.contains = true;
            visit_val(list, usage);
            visit_val(needle, usage);
        }
        Val::ContainsSubstring { haystack, needle } => {
            visit_val(haystack, usage);
            visit_val(needle, usage);
        }
        Val::ArgDynamic(index) => {
            usage.arg_dynamic = true;
            visit_val(index, usage);
        }
        Val::ContainsLine { file, needle } => {
            visit_val(file, usage);
            visit_val(needle, usage);
        }

        Val::FindFiles { dir, name } => {
            usage.find_files = true;
            visit_val(dir, usage);
            visit_val(name, usage);
        }
        Val::Spawn { args, .. } => {
            for a in args {
                visit_val(a, usage);
            }
        }
        Val::Wait { pid, .. } => {
            visit_val(pid, usage);
        }
        Val::WaitAll { pids, .. } => {
            visit_val(pids, usage);
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

// RF-02b: Status emission helpers for mechanically safe patterns.
// Note: Some occurrences of `__sh2_status=$?` are embedded in inline shell strings 
// (e.g., `"; __sh2_status=$?\n"`) or use custom `"exit"` variants. 
// Those are intentionally left manual to preserve byte-for-byte identical output without over-engineering.

fn emit_status_capture(pad: &str, out: &mut String) {
    out.push_str(pad);
    out.push_str("__sh2_status=$?\n");
}

pub(super) fn emit_status_check(pad: &str, out: &mut String) {
    emit_status_capture(pad, out);
    out.push_str(pad);
    out.push_str("__sh2_check \"$__sh2_status\" \"${__sh2_loc:-}\"\n");
}

pub(super) fn emit_status_check_ctx(pad: &str, out: &mut String, in_cond_ctx: bool) {
    emit_status_capture(pad, out);
    out.push_str(pad);
    if in_cond_ctx {
        out.push_str("__sh2_check \"$__sh2_status\" \"${__sh2_loc:-}\" \"return\"\n");
    } else {
        out.push_str("__sh2_check \"$__sh2_status\" \"${__sh2_loc:-}\"\n");
    }
}

// Used when capture and check are separated (e.g., by a `$!` capture in Spawn)
pub(super) fn emit_status_check_only(pad: &str, out: &mut String) {
    out.push_str(pad);
    out.push_str("__sh2_check \"$__sh2_status\" \"${__sh2_loc:-}\"\n");
}

use std::collections::HashSet;

#[derive(Default)]
pub(super) struct CodegenContext {
    pub(super) known_lists: HashSet<String>,
    uid_counter: usize,
}

impl CodegenContext {
    pub(super) fn next_id(&mut self) -> usize {
        let id = self.uid_counter;
        self.uid_counter += 1;
        id
    }
}
