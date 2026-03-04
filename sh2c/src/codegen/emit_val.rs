use super::helpers::sh_single_quote;
use super::emit_prelude::is_prelude_helper;
use super::TargetShell;
use crate::error::CompileError;
use crate::ir::Val;

pub(super) fn emit_val(v: &Val, target: TargetShell) -> Result<String, CompileError> {
    match v {
        Val::ContainsList { .. } | Val::ContainsSubstring { .. } => {
             let cond = emit_cond(v, target)?;
             Ok(format!("\"$( if {}; then printf true; else printf false; fi )\"", cond))
        }
        Val::Literal(s) => Ok(sh_single_quote(s)),
        Val::Var(s) => Ok(format!("\"${}\"", s)),
        Val::Concat(l, r) => Ok(format!("{}{}", emit_val(l, target)?, emit_val(r, target)?)),
        Val::TryRun(_) => Err(CompileError::unsupported("try_run() must be bound via let (e.g., let r = try_run(...))", target)),
        Val::Which(arg) => {
            match target {
                TargetShell::Bash => Ok(format!("\"$( __sh2_suppress_err_depth=$((${{__sh2_suppress_err_depth:-0}}+1)); __sh2_which {} )\"", emit_word(arg, target)?)),
                _ => Ok(format!("\"$( __sh2_which {} )\"", emit_word(arg, target)?)),
            }
        }
        Val::ReadFile(arg) => {
            let path = emit_word(arg, target)?;
            match target {
                TargetShell::Bash => Ok(format!("\"$( trap '' ERR; __sh2_read_file {} )\"", path)),
                _ => Ok(format!("\"$( __sh2_read_file {} )\"", path)),
            }
        }
        Val::Lines(_) => {
             return Err(CompileError::unsupported(
                "lines() is only valid in 'for' loops or 'let' assignment",
                target,
            ));
        }
        Val::FindFiles { .. } => {
            return Err(CompileError::unsupported(
                "find_files() is only valid in 'for' loops or 'let' assignment",
                target,
            ));
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
        Val::Bool(true) => Ok("true".to_string()),
        Val::Bool(false) => Ok("false".to_string()),
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
        Val::Glob(_) => {
            return Err(CompileError::unsupported(
                "glob() can only be used in 'for' loops or assignments",
                target,
            ));
        }
        Val::Spawn { .. } => {
            return Err(CompileError::unsupported(
                "spawn() can only be used in 'let' assignments",
                target,
            ));
        }
        Val::Wait { .. } => {
            return Err(CompileError::unsupported(
                "wait() can only be used in 'let' assignments",
                target,
            ));
        }
        Val::WaitAll { .. } => {
            return Err(CompileError::unsupported(
                "wait_all() can only be used in 'let' assignments",
                target,
            ));
        }

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
        Val::BoolVar(name) => Ok(format!("\"${}\"", name)),
    }
}

pub(super) fn emit_word(v: &Val, target: TargetShell) -> Result<String, CompileError> {
    if let Val::Args = v {
        return Ok("\"$@\"".to_string());
    }
    emit_val(v, target)
}

pub(super) fn emit_cond(v: &Val, target: TargetShell) -> Result<String, CompileError> {
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
        Val::ContainsList { list, needle } => {
            if target == TargetShell::Posix {
                 return Err(CompileError::unsupported("contains(list, item) is Bash-only. contains(string, substring) is supported.", target));
            }
             match **list {
                 Val::Var(ref name) => {
                     let n = emit_val(needle, target)?;
                     Ok(format!("__sh2_contains \"{}\" {}", name, n))
                 }
                    _ => {
                        // This path should be unreachable because lower_expr guarantees 
                        // that list expressions are materialized into temporary variables (Val::Var)
                        // before reaching codegen. If we see a non-Var list here, it's a compiler bug.
                        return Err(CompileError::internal(
                             "contains(list, item): expected list to be materialized to Val::Var by lowering", 
                             target
                        ));
                    }

             }
        }
        Val::ContainsSubstring { haystack, needle } => {
            // Substring check: printf '%s' "$haystack" | grep -Fq -e "$needle"
            // -F: fixed string (no regex), -q: quiet, -e: explicit pattern (POSIX-compliant)
            Ok(format!("( printf '%s' {} | grep -Fq -e {} )",
                emit_val(haystack, target)?,
                emit_val(needle, target)?
            ))
        }
        Val::Bool(true) => Ok("true".to_string()),
        Val::Bool(false) => Ok("false".to_string()),
        Val::List(_) | Val::Args => {
            Err(CompileError::internal("args/list is not a valid condition; use count(...) > 0", target))
        }
        Val::ContainsLine { file, needle } => {
            // Exact-line match: grep -Fqx -e <needle> <file>
            // -F: fixed string, -q: quiet, -x: exact line
            // -e: explicit pattern (POSIX-compliant, handles needles starting with -)
            Ok(format!("( grep -Fqx -e {} {} )",
                emit_val(needle, target)?,
                emit_val(file, target)?
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
            // Boolean variable: check if equals "true"
            Ok(format!("[ \"${}\" = \"true\" ]", name))
        }
        // "Truthiness" fallback for scalar values: check if non-empty string.
        v => Ok(format!("[ -n {} ]", emit_val(v, target)?)),
    }
}

pub(super) fn emit_index_expr(v: &Val, target: TargetShell) -> Result<String, CompileError> {
    emit_arith_expr(v, target)
}

pub(super) fn emit_cmd_body_raw(args: &[Val], target: TargetShell) -> Result<String, CompileError> {
    let parts: Vec<String> = args.iter().map(|a| emit_word(a, target)).collect::<Result<_, _>>()?;
    Ok(parts.join(" "))
}

fn emit_cmdsub_raw(args: &[Val], target: TargetShell) -> Result<String, CompileError> {
    Ok(format!("$( {} )", emit_cmd_body_raw(args, target)?))
}

pub(super) fn emit_cmd_pipe_body_raw(segments: &[Vec<Val>], target: TargetShell) -> Result<String, CompileError> {
    let seg_strs: Vec<String> = segments
        .iter()
        .map(|seg| emit_cmd_body_raw(seg, target))
        .collect::<Result<_, CompileError>>()?;
    Ok(seg_strs.join(" | "))
}

fn emit_cmdsub_pipe_raw(segments: &[Vec<Val>], target: TargetShell) -> Result<String, CompileError> {
    Ok(format!("$( {} )", emit_cmd_pipe_body_raw(segments, target)?))
}

pub(super) fn emit_arith_expr(v: &Val, target: TargetShell) -> Result<String, CompileError> {
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
