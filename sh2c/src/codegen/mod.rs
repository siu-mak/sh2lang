pub mod posix_lint;
pub use posix_lint::{PosixLint, PosixLintKind, lint_script, render_lints};

pub(crate) mod helpers;

use crate::error::CompileError;
use crate::ir::Function;
pub use crate::target::TargetShell;

mod emit_prelude;
use self::emit_prelude::emit_prelude;

mod emit_val;

mod emit_cmd;
use self::emit_cmd::emit_cmd;

mod scan_usage;
use self::scan_usage::scan_usage;
use self::scan_usage::PreludeUsage;

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
