use crate::ast;
use crate::ir;
use crate::span::SourceMap;
use crate::error::CompileError;
use crate::sudo::SudoSpec;
use super::{LoweringContext, LowerOptions};
use super::expr::lower_expr;

pub(super) fn lower_run_call_args<'a>(
    run_call: &ast::RunCall,
    out: &mut Vec<ir::Cmd>,
    ctx: &mut LoweringContext<'a>,
    sm: &SourceMap,
    file: &str,
    opts: &'a LowerOptions,
) -> Result<(Vec<ir::Val>, bool), CompileError> {
    let lowered_args = run_call
        .args
        .iter()
        .map(|a| lower_expr(a.clone(), out, ctx, sm, file))
        .collect::<Result<Vec<_>, _>>()?;

    let mut allow_fail = false;
    for opt in &run_call.options {
        if opt.name == "shell" {
             return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "shell option is not supported in run(...); use sh(...) for raw shell code", opt.span)));
        } else if opt.name == "allow_fail" {
             if let ast::ExprKind::Bool(b) = opt.value.node {
                 allow_fail = b;
             } else {
                 return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), "allow_fail must be true/false", opt.value.span)));
             }
        } else {
             return Err(CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), &format!("Unknown option {:?}", opt.name), opt.span)));
        }
    }
    
    Ok((lowered_args, allow_fail))
}

pub(super) fn lower_sudo_command<'a>(
    args: Vec<ast::Expr>,
    options: Vec<ast::CallOption>,
    out: &mut Vec<ir::Cmd>,
    ctx: &mut LoweringContext<'a>,
    sm: &SourceMap,
    file: &str,
) -> Result<(Vec<ir::Val>, Option<crate::span::Span>), CompileError> {
    let opts = ctx.opts();

    // Use SudoSpec to parse/validate options
    // Note: SudoSpec::from_options uses (String, Span), we need compatibility
    let spec = SudoSpec::from_options(&options)
        .map_err(|(msg, span)| CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), &msg, span)))?;

    let mut argv = Vec::new();
    argv.push(ir::Val::Literal("sudo".to_string()));

    // Use deterministic flag generation from spec
    let flags = spec.to_flags_argv();
    for flag in flags {
        argv.push(ir::Val::Literal(flag));
    }

    // Mandatory separator before command args
    argv.push(ir::Val::Literal("--".to_string()));

    // Add positional args (command + args), lowered
    for arg in args {
        argv.push(lower_expr(arg, out, ctx, sm, file)?);
    }
    
    // Extract allow_fail check from spec (it handles the boolean logic)
    // Note spec maps (bool, span).
    // We return the span if allow_fail is present (true or false doesn't matter for the error check in Expr, 
    // but usually user=true is the trigger. Wait, spec says "is_some() -> Err".
    // We traverse options to find the name span for correct highlighting
    let allow_fail_name_span = options.iter()
        .find(|o| o.name == "allow_fail")
        .map(|o| o.span);

    Ok((argv, allow_fail_name_span))
}

pub(super) fn lower_sudo_call_args<'a>(
    run_call: &ast::RunCall,
    out: &mut Vec<ir::Cmd>,
    ctx: &mut LoweringContext<'a>,
    sm: &SourceMap,
    file: &str,
    opts: &'a LowerOptions,
) -> Result<(Vec<ir::Val>, bool), CompileError> {
    // Validate options via SudoSpec (validation only, duplicates parser check but safe)
    // Note: parser already validated, but we need to re-derive the flags deterministicly.
    // Parser constructed the AST as raw args/options.
    
    let spec = SudoSpec::from_options(&run_call.options)
        .map_err(|(msg, span)| CompileError::new(sm.format_diagnostic(file, opts.diag_base_dir.as_deref(), &msg, span)))?;

    let mut argv = Vec::new();
    argv.push(ir::Val::Literal("sudo".to_string()));

    // Deterministic flags
    for flag in spec.to_flags_argv() {
        argv.push(ir::Val::Literal(flag));
    }

    // Positional args
    for arg in &run_call.args {
        argv.push(lower_expr(arg.clone(), out, ctx, sm, file)?);
    }
    
    // Extract allow_fail boolean
    let allow_fail = spec.allow_fail.map(|(b, _)| b).unwrap_or(false);
    
    Ok((argv, allow_fail))
}
