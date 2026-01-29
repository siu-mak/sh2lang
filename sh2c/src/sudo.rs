use crate::ast::{ExprKind, CallOption};
use crate::span::Span;

#[derive(Debug)]
pub struct SudoSpec {
    // (value, span) pairs for precise diagnostics
    pub user: Option<(String, Span)>,
    pub n: Option<(bool, Span)>,
    pub k: Option<(bool, Span)>,
    pub prompt: Option<(String, Span)>,
    pub e: Option<(bool, Span)>,
    pub env_keep: Option<(Vec<String>, Span)>,
    pub allow_fail: Option<(bool, Span)>,
}

impl SudoSpec {
    pub fn new() -> Self {
        Self {
            user: None,
            n: None,
            k: None,
            prompt: None,
            e: None,
            env_keep: None,
            allow_fail: None,
        }
    }

    pub fn from_options(
        options: &[CallOption],
    ) -> Result<Self, (String, Span)> {
        let mut spec = SudoSpec::new();

        for opt in options {
            let name = opt.name.as_str();
            let val = &opt.value;

            match name {
                "user" => {
                    if spec.user.is_some() {
                        return Err(("user specified more than once".to_string(), opt.span));
                    }
                    if let ExprKind::Literal(s) = &val.node {
                        spec.user = Some((s.clone(), val.span));
                    } else {
                        return Err(("user must be a string literal".to_string(), val.span));
                    }
                }
                "prompt" => {
                    if spec.prompt.is_some() {
                        return Err(("prompt specified more than once".to_string(), opt.span));
                    }
                    if let ExprKind::Literal(s) = &val.node {
                        spec.prompt = Some((s.clone(), val.span));
                    } else {
                        return Err(("prompt must be a string literal".to_string(), val.span));
                    }
                }
                "n" => {
                    if spec.n.is_some() {
                        return Err(("n specified more than once".to_string(), opt.span));
                    }
                    if let ExprKind::Bool(b) = val.node {
                        spec.n = Some((b, val.span));
                    } else {
                        return Err(("n must be a boolean literal".to_string(), val.span));
                    }
                }
                "k" => {
                    if spec.k.is_some() {
                        return Err(("k specified more than once".to_string(), opt.span));
                    }
                    if let ExprKind::Bool(b) = val.node {
                        spec.k = Some((b, val.span));
                    } else {
                        return Err(("k must be a boolean literal".to_string(), val.span));
                    }
                }
                "E" => {
                    if spec.e.is_some() {
                        return Err(("E specified more than once".to_string(), opt.span));
                    }
                    if let ExprKind::Bool(b) = val.node {
                        spec.e = Some((b, val.span));
                    } else {
                        return Err(("E must be a boolean literal".to_string(), val.span));
                    }
                }
                "allow_fail" => {
                    if spec.allow_fail.is_some() {
                        return Err(("allow_fail specified more than once".to_string(), opt.span));
                    }
                    if let ExprKind::Bool(b) = val.node {
                        spec.allow_fail = Some((b, val.span));
                    } else {
                        return Err(("allow_fail must be a boolean literal".to_string(), val.span));
                    }
                }
                "env_keep" => {
                    if spec.env_keep.is_some() {
                        return Err(("env_keep specified more than once".to_string(), opt.span));
                    }
                    if let ExprKind::List(items) = &val.node {
                        let mut keep_vars = Vec::new();
                        for item in items {
                            if let ExprKind::Literal(s) = &item.node {
                                keep_vars.push(s.clone());
                            } else {
                                return Err(("env_keep must be a list of string literals".to_string(), item.span));
                            }
                        }
                        spec.env_keep = Some((keep_vars, val.span));
                    } else {
                        return Err(("env_keep must be a list literal".to_string(), val.span));
                    }
                }
                _ => {
                    return Err((format!(
                        "unknown sudo() option '{}'; supported: user, n, k, prompt, E, env_keep, allow_fail",
                        name
                    ), opt.span));
                }
            }
        }

        Ok(spec)
    }

    pub fn to_flags_argv(&self) -> Vec<String> {
        let mut flags = Vec::new();

        // Stable ordering: -u, -n, -k, -p, -E, --preserve-env

        if let Some((user, _)) = &self.user {
            flags.push("-u".to_string());
            flags.push(user.clone());
        }

        if let Some((true, _)) = self.n {
            flags.push("-n".to_string());
        }

        if let Some((true, _)) = self.k {
            flags.push("-k".to_string());
        }

        if let Some((prompt, _)) = &self.prompt {
            flags.push("-p".to_string());
            flags.push(prompt.clone());
        }

        if let Some((true, _)) = self.e {
            flags.push("-E".to_string());
        }

        if let Some((vars, _)) = &self.env_keep {
            if !vars.is_empty() {
                flags.push(format!("--preserve-env={}", vars.join(",")));
            }
        }
        
        // Note: allow_fail is NOT a sudo flag, it is handled by the caller (statement flow logic)

        // Always append separator
        flags.push("--".to_string());

        flags
    }
}
